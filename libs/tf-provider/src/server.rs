use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Seek;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use base64::Engine;
use futures::TryFutureExt;
use rcgen::{BasicConstraints, IsCa};
use rustls::internal::msgs::handshake::DigitallySignedStruct;
use rustls::internal::pemfile;
use rustls::{ClientCertVerified, HandshakeSignatureValid, ProtocolVersion, TLSError};
use time::ext::NumericalDuration;
use tokio::try_join;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::ServerTlsConfig;
use tower_http::trace::TraceLayer;

use crate::plugin::grpc_broker_server::GrpcBrokerServer;
use crate::plugin::grpc_controller_server::GrpcControllerServer;
use crate::plugin::grpc_stdio_server::GrpcStdioServer;
use crate::tfplugin6::provider_server::ProviderServer;

use crate::data_source::DynamicDataSource;
use crate::plugin::GrpcIo;
use crate::provider::DynamicProvider;
use crate::resource::DynamicResource;
use crate::{Diagnostics, Schema};

const CORE_PROTOCOL_VERSION: u8 = 1;

pub struct Server {
    pub(crate) provider: Box<dyn DynamicProvider>,
    pub(crate) io: GrpcIo,
    pub(crate) cancellation_token: CancellationToken,

    pub(crate) init_diags: Diagnostics,
    pub(crate) schema: Option<Schema>,
    pub(crate) meta_schema: Option<Schema>,
    pub(crate) resources: HashMap<String, (Box<dyn DynamicResource>, Schema)>,
    pub(crate) data_sources: HashMap<String, (Box<dyn DynamicDataSource>, Schema)>,
}

impl Server {
    pub fn new(provider_name: String, provider: Box<dyn DynamicProvider>) -> Self {
        let mut diags = Default::default();
        let mut has_errors = false;
        let mut set_error = || {
            has_errors = true;
            None
        };
        let schema = provider.schema(&mut diags).or_else(&mut set_error);
        let meta_schema = provider.meta_schema(&mut diags).or_else(&mut set_error);
        let resources = provider
            .get_resources(&mut diags)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(name, resource)| {
                let schema = resource.schema(&mut diags).or_else(&mut set_error)?;
                Some((format!("{}_{}", provider_name, name), (resource, schema)))
            })
            .collect();
        let data_sources = provider
            .get_data_sources(&mut diags)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(name, data_source)| {
                let schema = data_source.schema(&mut diags).or_else(&mut set_error)?;
                Some((format!("{}_{}", provider_name, name), (data_source, schema)))
            })
            .collect();

        if has_errors {
            diags.internal_error()
        }

        Self {
            provider,
            io: Default::default(),
            cancellation_token: Default::default(),
            init_diags: diags,
            schema,
            meta_schema,
            resources,
            data_sources,
        }
    }

    pub(crate) fn get_resource<'a>(
        &'a self,
        diags: &mut Diagnostics,
        name: &str,
    ) -> Option<&'a dyn DynamicResource> {
        if let Some(resource) = self.resources.get(name) {
            Some(resource.0.as_ref())
        } else {
            diags.root_error_short(format!("Could not find resource `{}` in provider", name));
            None
        }
    }
    pub(crate) fn get_data_source<'a>(
        &'a self,
        diags: &mut Diagnostics,
        name: &str,
    ) -> Option<&'a dyn DynamicDataSource> {
        if let Some(data_source) = self.data_sources.get(name) {
            Some(data_source.0.as_ref())
        } else {
            diags.root_error_short(format!("Could not find data source `{}` in provider", name));
            None
        }
    }
}

pub async fn serve<U: ToString, V: DynamicProvider>(name: U, provider: V) -> Result<()> {
    serve_dynamic(name.to_string(), Box::new(provider)).await
}

pub async fn serve_dynamic(name: String, provider: Box<dyn DynamicProvider>) -> Result<()> {
    let server = Arc::new(Server::new(name, provider));
    let log_file = File::create("cmd-trace.log")?;
    let addrs = SockAddrIter::new()?;
    let (tcp_stream, endpoint) = listen(addrs)?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(Mutex::new(log_file))
        .init();

    let tls_config = TlsConfig::new()?;

    let serve = tls_config
        .builder()?
        .layer(TraceLayer::new_for_grpc())
        .add_service(GrpcBrokerServer::new(server.clone()))
        .add_service(GrpcControllerServer::new(server.clone()))
        .add_service(GrpcStdioServer::new(server.clone()))
        .add_service(ProviderServer::new(server.clone()))
        .serve_with_incoming_shutdown(tcp_stream, server.cancellation_token.cancelled());

    async fn info(endpoint: &str, der: &[u8]) -> Result<()> {
        println!(
            "{}|6|tcp|{}|grpc|{}",
            CORE_PROTOCOL_VERSION,
            endpoint,
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(der),
        );
        Ok(())
    }

    try_join!(
        serve.map_err(|e| anyhow!(e)),
        info(&endpoint, tls_config.cert.as_slice()),
    )?;

    Ok(())
}

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
struct SockAddrIter {
    ip: IpAddr,
    port: u16,
    port_max: u16,
}

impl SockAddrIter {
    fn new_with_ip(ip: IpAddr) -> Result<Self> {
        let port_min = if let Ok(port) = env::var("PLUGIN_MIN_PORT") {
            port.parse()?
        } else {
            1025_u16
        };
        let port_max = if let Ok(port) = env::var("PLUGIN_MAX_PORT") {
            port.parse()?
        } else {
            65535_u16
        };

        Ok(Self {
            ip,
            port: std::cmp::min(port_min, port_max),
            port_max: std::cmp::max(port_min, port_max),
        })
    }
    fn new() -> Result<Self> {
        Self::new_with_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
    }
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

fn listen(addrs: SockAddrIter) -> Result<(TcpListenerStream, String)> {
    let listener = std::net::TcpListener::bind(addrs)?;
    listener.set_nonblocking(true)?;
    let endpoint = listener.local_addr()?.to_string();
    let listener = tokio::net::TcpListener::from_std(listener)?;
    Ok((TcpListenerStream::new(listener), endpoint))
}

struct TlsConfig {
    server: Option<ServerTlsConfig>,
    cert: Vec<u8>,
}

impl TlsConfig {
    fn new() -> Result<Self> {
        let env_cert = env::var("PLUGIN_CLIENT_CERT").unwrap_or_default();
        if env_cert.is_empty() {
            return Ok(Self {
                server: None,
                cert: Default::default(),
            });
        }

        // Read client certificate and put it into the cert store
        let mut client_root_cert_store = rustls::RootCertStore::empty();
        let mut pem_buffer = std::io::Cursor::new(env_cert.clone());
        client_root_cert_store
            .add_pem_file(&mut pem_buffer)
            .or(Err(anyhow!("Could not parse client certificate")))?;

        // Parameters to generate the server certificate
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

        // Generate the server certificate and its keys
        let server_cert = rcgen::Certificate::from_params(cp)?;

        let mut key_buffer = std::io::Cursor::new(server_cert.serialize_private_key_pem());
        let mut key = pemfile::pkcs8_private_keys(&mut key_buffer).or(Err(anyhow!(
            "Could not generate the private key of the server certificate"
        )))?;

        let server_cert_der = server_cert.serialize_der_with_signer(&server_cert)?;
        let p = pem::Pem::new("CERTIFICATE".to_string(), server_cert_der.clone());
        let server_cert_pem = pem::encode(&p);

        let mut cert_buffer = std::io::Cursor::new(server_cert_pem);
        let tls_cert = pemfile::certs(&mut cert_buffer).unwrap();

        cert_buffer.rewind()?;

        let raw_cert = env_cert.as_bytes();
        let x509_cert = x509_parser::pem::parse_x509_pem(raw_cert)?.1;
        let mut server_config = rustls::ServerConfig::new(std::sync::Arc::new(CertVerifier {
            cert: x509_cert.contents,
            root_store: client_root_cert_store,
        }));
        server_config.set_single_cert(
            tls_cert,
            key.pop().ok_or(anyhow!(
                "Could not get the private key of the server certificate"
            ))?,
        )?;
        server_config.versions = vec![ProtocolVersion::TLSv1_2];
        let mut tls_config = ServerTlsConfig::new();
        tls_config.rustls_server_config(server_config);

        Ok(Self {
            server: Some(tls_config),
            cert: server_cert_der,
        })
    }

    fn builder(&self) -> Result<tonic::transport::server::Server> {
        if let Some(ref tls_config) = &self.server {
            Ok(tonic::transport::server::Server::builder().tls_config(tls_config.clone())?)
        } else {
            Ok(tonic::transport::server::Server::builder())
        }
    }
}
