use anyhow::Result;
use cmd_provider::CmdProvider;
use tf_provider::serve;

mod cmd_provider;
mod cmd_resource;
mod connection;
mod connection_local;

#[tokio::main]
async fn main() -> Result<()> {
    serve("cmd", CmdProvider::default()).await
}
