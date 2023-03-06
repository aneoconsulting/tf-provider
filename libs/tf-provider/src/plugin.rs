use std::fmt::Write;
use std::pin::Pin;

use futures::{Stream, StreamExt};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status};

tonic::include_proto!("plugin");

#[derive(Debug)]
pub struct GrpcBroker {
    pub io: GrpcIo,
    pub cancellation_token: CancellationToken,
}

#[tonic::async_trait]
impl grpc_broker_server::GrpcBroker for GrpcBroker {
    type StartStreamStream = Pin<Box<dyn Stream<Item = Result<ConnInfo, Status>> + Send + 'static>>;
    async fn start_stream(
        &self,
        request: Request<tonic::Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        let mut stream = request.into_inner();
        let token = self.cancellation_token.clone();

        let io = self.io.clone();
        let output = async_stream::try_stream! {
            loop {
                tokio::select!{
                    Some(conn_info) = stream.next() => match conn_info {
                        Ok(conn_info) => {
                            write!(io.stdout(), "relay {}\n", conn_info).unwrap_or(());
                            yield conn_info.clone();
                        },
                        Err(status) => {
                            write!(io.stderr(), "stream error {}\n", status).unwrap_or(());
                        },
                    },
                    _ = token.cancelled() => break,
                };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StartStreamStream))
    }
}

impl core::fmt::Display for ConnInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "ConnInfo{{service_id: {}, network: {}, address: {} }}",
            self.service_id, self.network, self.address
        )
    }
}

#[derive(Debug)]
pub struct GrpcController {
    pub io: GrpcIo,
    pub cancellation_token: CancellationToken,
}

#[tonic::async_trait]
impl grpc_controller_server::GrpcController for GrpcController {
    async fn shutdown(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        self.cancellation_token.cancel();

        Ok(Response::new(Empty {}))
    }
}

#[derive(Debug)]
pub struct GrpcStdio {
    pub tx: broadcast::Sender<StdioData>,
    pub cancellation_token: CancellationToken,
}

#[tonic::async_trait]
impl grpc_stdio_server::GrpcStdio for GrpcStdio {
    type StreamStdioStream =
        Pin<Box<dyn Stream<Item = Result<StdioData, Status>> + Send + 'static>>;
    async fn stream_stdio(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::StreamStdioStream>, tonic::Status> {
        let mut rx = self.tx.subscribe();
        let token = self.cancellation_token.clone();

        let output = async_stream::try_stream! {
            loop {
                tokio::select! {
                    iodata = rx.recv() =>
                        if let Ok(iodata) = iodata {
                            yield iodata.clone();
                        } else if let Err(broadcast::error::RecvError::Lagged(n)) = iodata {
                            eprintln!("IO over grpc lags behind by {n} messages!");
                        } else {
                            break;
                        },
                    _ = token.cancelled() => break,
                };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StreamStdioStream))
    }
}

#[derive(Debug, Clone)]
pub struct GrpcIoStream<'a> {
    pub tx: &'a broadcast::Sender<StdioData>,
    pub channel: i32,
}

impl std::fmt::Write for GrpcIoStream<'_> {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        match self.tx.send(StdioData {
            channel: self.channel,
            data: s.as_bytes().to_vec(),
        }) {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrpcIo {
    pub tx: broadcast::Sender<StdioData>,
}

impl GrpcIo {
    fn stdout(&self) -> GrpcIoStream {
        GrpcIoStream {
            tx: &self.tx,
            channel: 1,
        }
    }
    fn stderr(&self) -> GrpcIoStream {
        GrpcIoStream {
            tx: &self.tx,
            channel: 2,
        }
    }
}
