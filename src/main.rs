use anyhow::Result;
use generic_provider::GenericProvider;
use tf_provider::serve;

mod cmd;
mod connection;
mod file;
mod generic_provider;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    serve("generic", GenericProvider::default()).await
}
