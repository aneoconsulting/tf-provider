#![feature(once_cell)]

//use std::sync::OnceLock;


use std::pin::Pin;

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

#[derive(Debug, Default)]
pub struct MyGrpcBroker {}

#[tonic::async_trait]
impl GrpcBroker for MyGrpcBroker {
    type StartStreamStream = Pin<Box<dyn Stream<Item = Result<ConnInfo, Status>> + Send + 'static>>;
    async fn start_stream(
        &self,
        request: Request<tonic::Streaming<ConnInfo>>,
    ) -> Result<Response<Self::StartStreamStream>, Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(conn_info) = stream.next().await {
                let conn_info = conn_info?;
                yield conn_info.clone();
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StartStreamStream))
    }
}

#[derive(Debug, Default)]
pub struct MyGrpcController {}

#[tonic::async_trait]
impl GrpcController for MyGrpcController {
    async fn shutdown(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
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
            while let Ok(iodata) = rx.recv().await {
                yield iodata.clone();
            }
        };

        Ok(Response::new(Box::pin(output) as Self::StreamStdioStream))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let grpc_broker = MyGrpcBroker::default();
    let grpc_controller = MyGrpcController::default();
    let (tx, _) = broadcast::channel(10);
    let grpc_stdio = MyGrpcStdio{tx};

    Server::builder()
        .add_service(GrpcBrokerServer::new(grpc_broker))
        .add_service(GrpcControllerServer::new(grpc_controller))
        .add_service(GrpcStdioServer::new(grpc_stdio))
        .serve(addr)
        .await?;

    Ok(())
}
