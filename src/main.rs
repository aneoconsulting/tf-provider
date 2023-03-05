use plugin::{
    grpc_broker_server::GrpcBrokerServer, grpc_controller_server::GrpcControllerServer,
    grpc_stdio_server::GrpcStdioServer,
};
use plugin::{GrpcBroker, GrpcController, GrpcIo, GrpcStdio};
use provider::tf::provider_server::ProviderServer;
use provider::CmdProvider;

use std::{
    env,
    fs::File,
    io::SeekFrom,
    net::{IpAddr, SocketAddr},
    sync::Mutex,
};

use time::ext::NumericalDuration;

use anyhow::{anyhow, Result};
use futures::{try_join, TryFutureExt};
use rcgen::{BasicConstraints, IsCa};
use rustls::{
    internal::pemfile, ClientCertVerified, HandshakeSignatureValid, ProtocolVersion, TLSError,
};
use tokio::io::AsyncSeekExt;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tonic::transport::{server::ServerTlsConfig, Server};
use tower_http::trace::TraceLayer;

use rustls::internal::msgs::handshake::DigitallySignedStruct;

mod plugin;
mod provider;
mod value;

const CORE_PROTOCOL_VERSION: u8 = 1;

struct CertVerifier {
    pub cert: Vec<u8>,
    pub root_store: rustls::RootCertStore,
}

impl rustls::ClientCertVerifier for CertVerifier {
    fn client_auth_root_subjects(
        &self,
        _sni: Option<&webpki::DNSName>,
    ) -> Option<rustls::DistinguishedNames> {
        Some(self.root_store.get_subjects())
    }

    fn verify_client_cert(
        &self,
        presented_certs: &[rustls::Certificate],
        _sni: Option<&webpki::DNSName>,
    ) -> Result<rustls::ClientCertVerified, TLSError> {
        if presented_certs.len() != 1 {
            return Err(TLSError::General(format!(
                "server sent {} certificates, expected one",
                presented_certs.len()
            )));
        }
        if presented_certs[0].0 != self.cert {
            return Err(TLSError::General(
                "server certificates doesn't match ours".to_string(),
            ));
        }
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TLSError> {
        // It's a SHA-512 ECDSA, which webpki doesn't support. We assume by default that if the client cert
        // someone handed us equals the one in the environment variables that this is probably ok.
        //
        // FIXME: Blocked by upstream https://github.com/briansmith/ring/issues/824

        Ok(HandshakeSignatureValid::assertion())
    }
}

#[derive(Copy, Clone)]
pub struct SockAddrIter {
    pub ip: IpAddr,
    pub port: u16,
    pub port_max: u16,
}

impl Iterator for SockAddrIter {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<Self::Item> {
        if self.port_max == 0 {
            return None;
        }
        let port = self.port;
        if self.port == self.port_max {
            self.port_max = 0;
        } else {
            self.port += 1;
        }
        Some(SocketAddr::new(self.ip, port))
    }
}

impl std::net::ToSocketAddrs for SockAddrIter {
    type Iter = Self;
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        Ok(*self)
    }
}

#[derive(Debug, Default)]
struct ServerConfig {
    pub tls_config: Option<ServerTlsConfig>,
    pub cert: Vec<u8>,
    pub port_min: u16,
    pub port_max: u16,
}

impl ServerConfig {
    pub async fn new() -> Result<ServerConfig, Box<dyn std::error::Error>> {
        let mut port_min = 1025_u16;
        let mut port_max = 65535_u16;

        if let Ok(port) = env::var("PLUGIN_MIN_PORT") {
            port_min = port.parse()?;
        }
        if let Ok(port) = env::var("PLUGIN_MAX_PORT") {
            port_max = port.parse()?;
        }

        port_min = std::cmp::max(port_min, 1);
        port_max = std::cmp::min(port_max, 65535);

        if port_max < port_min {
            std::mem::swap(&mut port_min, &mut port_max);
        }

        if let Ok(env_cert) = env::var("PLUGIN_CLIENT_CERT") {
            let mut client_root_cert_store = rustls::RootCertStore::empty();
            let mut pem_buffer = std::io::Cursor::new(env_cert.clone());
            client_root_cert_store
                .add_pem_file(&mut pem_buffer)
                .unwrap();
            let mut cp = rcgen::CertificateParams::new(vec!["localhost".to_string()]);
            cp.alg = &rcgen::PKCS_ECDSA_P384_SHA384;
            cp.not_before = time::OffsetDateTime::now_utc().saturating_sub(30.seconds());
            cp.not_after = time::OffsetDateTime::now_utc().saturating_add((30 * 365).days());
            let mut dn = rcgen::DistinguishedName::new();
            dn.push(rcgen::DnType::OrganizationName, "Hashicorp");
            dn.push(
                rcgen::DnType::CommonName,
                rcgen::DnValue::PrintableString("localhost".to_string()),
            );
            cp.distinguished_name = dn;
            cp.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
            cp.key_usages = vec![
                rcgen::KeyUsagePurpose::DigitalSignature,
                rcgen::KeyUsagePurpose::KeyEncipherment,
                rcgen::KeyUsagePurpose::KeyAgreement,
                rcgen::KeyUsagePurpose::KeyCertSign,
            ];
            cp.extended_key_usages = vec![
                rcgen::ExtendedKeyUsagePurpose::ClientAuth,
                rcgen::ExtendedKeyUsagePurpose::ServerAuth,
            ];
            cp.key_identifier_method = rcgen::KeyIdMethod::Sha512;
            let server_cert = rcgen::Certificate::from_params(cp)?;

            let mut key_buffer = std::io::Cursor::new(server_cert.serialize_private_key_pem());
            let mut key = pemfile::pkcs8_private_keys(&mut key_buffer).unwrap();

            let server_cert_der = server_cert.serialize_der_with_signer(&server_cert)?;
            let p = pem::Pem {
                tag: "CERTIFICATE".to_string(),
                contents: server_cert_der.clone(),
            };
            let server_cert_pem = pem::encode(&p);

            let mut cert_buffer = std::io::Cursor::new(server_cert_pem);
            let tls_cert = pemfile::certs(&mut cert_buffer).unwrap();

            cert_buffer.seek(SeekFrom::Start(0)).await?;

            let raw_cert = env_cert.as_bytes();
            let x509_cert = x509_parser::pem::parse_x509_pem(raw_cert)
                .unwrap()
                .1
                .clone();
            let mut server_config = rustls::ServerConfig::new(std::sync::Arc::new(CertVerifier {
                cert: x509_cert.contents,
                root_store: client_root_cert_store,
            }));
            server_config.set_single_cert(tls_cert, key.pop().unwrap())?;
            server_config.versions = vec![ProtocolVersion::TLSv1_2];
            let mut tls_config = ServerTlsConfig::new();
            tls_config.rustls_server_config(server_config);

            Ok(ServerConfig {
                tls_config: Some(tls_config),
                cert: server_cert_der,
                port_min,
                port_max,
            })
        } else {
            Ok(ServerConfig {
                tls_config: None,
                cert: Vec::<u8>::default(),
                port_min,
                port_max,
            })
        }
    }

    pub fn builder(&self) -> Result<Server, Box<dyn std::error::Error>> {
        if let Some(tls_config) = &self.tls_config {
            Ok(Server::builder().tls_config(tls_config.clone())?)
        } else {
            Ok(Server::builder())
        }
    }

    pub fn listen(&self, ip: IpAddr) -> std::io::Result<std::net::TcpListener> {
        let addrs = SockAddrIter {
            ip,
            port: self.port_min,
            port_max: self.port_max,
        };
        let listener = std::net::TcpListener::bind(addrs)?;
        listener.set_nonblocking(true)?;
        Ok(listener)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = File::create("cmd-trace.log")?;

    let server_config = ServerConfig::new().await?;
    let cancellation_token = CancellationToken::new();

    let ip = "127.0.0.1".parse()?;
    let (tx, _) = broadcast::channel(10);
    let grpc_io = GrpcIo { tx: tx.clone() };

    let grpc_broker = GrpcBroker {
        io: grpc_io.clone(),
        cancellation_token: cancellation_token.clone(),
    };
    let grpc_controller = GrpcController {
        io: grpc_io,
        cancellation_token: cancellation_token.clone(),
    };
    let grpc_stdio = GrpcStdio {
        tx: tx,
        cancellation_token: cancellation_token.clone(),
    };
    let provider = CmdProvider {};

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(Mutex::new(log_file))
        .init();

    let listener = server_config.listen(ip)?;
    let endpoint = listener.local_addr()?.to_string();
    let listener = tokio::net::TcpListener::from_std(listener)?;
    let listener_stream = tokio_stream::wrappers::TcpListenerStream::new(listener);
    let serve = server_config
        .builder()?
        .layer(TraceLayer::new_for_grpc())
        .add_service(GrpcBrokerServer::new(grpc_broker))
        .add_service(GrpcControllerServer::new(grpc_controller))
        .add_service(GrpcStdioServer::new(grpc_stdio))
        .add_service(ProviderServer::new(provider))
        .serve_with_incoming_shutdown(listener_stream, cancellation_token.cancelled());

    async fn info(endpoint: &str, der: &[u8]) -> Result<()> {
        println!(
            "{}|6|tcp|{}|grpc|{}",
            CORE_PROTOCOL_VERSION,
            endpoint,
            base64::encode_config(der, base64::STANDARD_NO_PAD)
        );
        Ok(())
    }

    try_join!(
        serve.map_err(|e| anyhow!(e)),
        info(&endpoint, server_config.cert.as_slice()),
    )?;

    Ok(())
}
