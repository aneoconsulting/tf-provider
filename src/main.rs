use anyhow::Result;
use cmd_provider::CmdProvider;
use tf_provider::serve;

mod cmd_exec;
mod cmd_file;
mod cmd_provider;
mod connection;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    serve("cmd", CmdProvider::default()).await
}
