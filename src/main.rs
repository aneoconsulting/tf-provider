#![feature(once_cell)]

//use std::sync::OnceLock;


use std::pin::Pin;
use std::fmt::Write;

use futures::{Stream, StreamExt};
use tokio::sync::broadcast;
use tonic::{transport::Server, Request, Response, Status};

use plugin::grpc_broker_server::{GrpcBroker, GrpcBrokerServer};
use plugin::grpc_controller_server::{GrpcController, GrpcControllerServer};
use plugin::grpc_stdio_server::{GrpcStdio, GrpcStdioServer};
use plugin::{ConnInfo, Empty, StdioData};

pub mod plugin {
    tonic::include_proto!("plugin");
}
pub mod tfplugin6 {
    tonic::include_proto!("tfplugin6");
}

#[derive(Debug)]
pub struct MyGrpcBroker {
    pub io: GrpcIo,
}

#[tonic::async_trait]
impl GrpcBroker for MyGrpcBroker {
    type StartStreamStream = Pin<Box<dyn Stream<Item = Result<ConnInfo, Status>> + Send + 'static>>;
    async fn start_stream(
        &self,
        request: Request<tonic::Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        let mut stream = request.into_inner();

        let io = self.io.clone();
        let output = async_stream::try_stream! {
            while let Some(conn_info) = stream.next().await {
                let conn_info = conn_info?;
                write!(io.stdout(), "relay {}\n", conn_info).unwrap();
                yield conn_info.clone();
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StartStreamStream))
    }
}

impl core::fmt::Display for ConnInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ConnInfo{{service_id: {}, network: {}, address: {} }}", self.service_id, self.network, self.address)
    }
}

#[derive(Debug)]
pub struct MyGrpcController {
    pub io: GrpcIo,
}

#[tonic::async_trait]
impl GrpcController for MyGrpcController {
    async fn shutdown(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        write!(self.io.stderr(), "shutdown requested\n").unwrap();

        Ok(Response::new(Empty{}))
    }
}

#[derive(Debug)]
pub struct MyGrpcStdio {
    pub tx: broadcast::Sender<StdioData>,
}

#[tonic::async_trait]
impl GrpcStdio for MyGrpcStdio {
    type StreamStdioStream = Pin<Box<dyn Stream<Item = Result<StdioData, Status>> + Send + 'static>>;
    async fn stream_stdio(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::StreamStdioStream>, tonic::Status> {
        let mut rx = self.tx.subscribe();

        let output = async_stream::try_stream! {
            loop {
                let iodata = rx.recv().await;
                if let Ok(iodata) = iodata {
                    yield iodata.clone();
                } else if let Err(broadcast::error::RecvError::Lagged(n)) = iodata {
                    eprintln!("IO over grpc lags behind by {n} messages!");
                } else {
                    break;
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StreamStdioStream))
    }
}

#[derive(Debug,Clone)]
pub struct GrpcIoStream<'a> {
    pub tx: &'a broadcast::Sender<StdioData>,
    pub channel: i32,
}

impl std::fmt::Write for GrpcIoStream<'_> {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        match self.tx.send(StdioData{channel: self.channel, data: s.as_bytes().to_vec()}) {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

#[derive(Debug,Clone)]
pub struct GrpcIo {
    pub tx: broadcast::Sender<StdioData>,
}

impl GrpcIo {
    fn stdout(&self) -> GrpcIoStream {
        GrpcIoStream{tx: &self.tx, channel: 1}
    }
    fn stderr(&self) -> GrpcIoStream {
        GrpcIoStream{tx: &self.tx, channel: 2}
    }
}





#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let (tx, _) = broadcast::channel(10);
    let grpc_io = GrpcIo{tx: tx.clone()};

    let grpc_broker = MyGrpcBroker{io: grpc_io.clone()};
    let grpc_controller = MyGrpcController{io:grpc_io};
    let grpc_stdio = MyGrpcStdio{tx: tx};

    Server::builder()
        .add_service(GrpcBrokerServer::new(grpc_broker))
        .add_service(GrpcControllerServer::new(grpc_controller))
        .add_service(GrpcStdioServer::new(grpc_stdio))
        .serve(addr)
        .await?;

    Ok(())
}
