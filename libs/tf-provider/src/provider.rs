use crate::data_source::DynamicDataSource;
use crate::resource::DynamicResource;
use crate::result::Result;
use crate::schema::Block;
use crate::schema::Schema;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a provider
trait Provider {
    /// Configuration of the provider
    type Config: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type MetaState: Serialize + DeserializeOwned;

    /// Get the schema of the provider
    fn schema(&mut self) -> Result<Schema>;
    /// Validate the configuration of the provider
    fn validate(&mut self, config: Self::Config) -> Result<()>;
    /// Configure the provider
    fn configure(&mut self, version: i64, config: Self::Config) -> Result<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&mut self) -> Result<Schema> {
        Schema {
            version: 1,
            block: Block::empty(),
        }
        .into()
    }

    /// Get the resources of the provider
    fn get_resources(&mut self) -> Result<Vec<Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources(&mut self) -> Result<Vec<Box<dyn DynamicDataSource>>>;
}
