use anyhow::Result;
use cmd_provider::CmdProvider;
use tf_provider::serve;

mod cmd_provider;
mod cmd_resource;

#[tokio::main]
async fn main() -> Result<()> {
    serve(CmdProvider::default()).await
}
