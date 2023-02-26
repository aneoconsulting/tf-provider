use tokio::sync::broadcast;
use tonic::transport::Server;

use plugin::{grpc_broker_server::GrpcBrokerServer, grpc_controller_server::GrpcControllerServer, grpc_stdio_server::GrpcStdioServer};
use plugin::{GrpcIo, GrpcStdio, GrpcController, GrpcBroker};

mod plugin;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let (tx, _) = broadcast::channel(10);
    let grpc_io = GrpcIo{tx: tx.clone()};

    let grpc_broker = GrpcBroker{io: grpc_io.clone()};
    let grpc_controller = GrpcController{io:grpc_io};
    let grpc_stdio = GrpcStdio{tx: tx};

    Server::builder()
        .add_service(GrpcBrokerServer::new(grpc_broker))
        .add_service(GrpcControllerServer::new(grpc_controller))
        .add_service(GrpcStdioServer::new(grpc_stdio))
        .serve(addr)
        .await?;

    Ok(())
}
