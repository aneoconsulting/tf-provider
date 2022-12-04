#![feature(once_cell)]

//use std::sync::OnceLock;


use std::pin::Pin;

use futures::{Stream, StreamExt};
//use tokio::sync::mpsc;
//use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

use plugin::grpc_broker_server::{GrpcBroker, GrpcBrokerServer};
use plugin::{ConnInfo};

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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let grpc_broker = MyGrpcBroker::default();

    Server::builder()
        .add_service(GrpcBrokerServer::new(grpc_broker))
        .serve(addr)
        .await?;

    Ok(())
}
