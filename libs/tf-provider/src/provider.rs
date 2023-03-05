use crate::diagnostics::Result;
use crate::schema::Block;
use crate::schema::Schema;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a provider
trait Provider {
    /// Configuration of the provider
    type Config: Serialize + DeserializeOwned;
    type MetaState: Serialize + DeserializeOwned;

    /// Get the schema of the provider
    fn schema(&mut self) -> Result<Schema>;
    /// Validate the configuration of the provider
    fn validate(&mut self, config: Self::Config) -> Result<()>;
    /// Configure the provider
    fn configure(&mut self, version: i64, config: Self::Config) -> Result<()>;

    /// Get the scema for the provider metadata (defaults to empty)
    fn meta_schema(&mut self) -> Result<Schema> {
        Schema {
            version: 1,
            block: Block::empty(),
        }
        .into()
    }
}
