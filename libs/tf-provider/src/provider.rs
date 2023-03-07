use std::collections::HashMap;

use crate::data_source::DynamicDataSource;
use crate::dynamic::DynamicValue;
use crate::resource::DynamicResource;
use crate::result::get;
use crate::result::Result;
use crate::schema::Block;
use crate::schema::Schema;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a provider
pub trait Provider: Send + Sync + 'static {
    /// Configuration of the provider
    type Config: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type MetaState: Serialize + DeserializeOwned;

    /// Get the schema of the provider
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the provider
    fn validate(&self, config: Self::Config) -> Result<()>;
    /// Configure the provider
    fn configure(&self, version: i64, config: Self::Config) -> Result<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self) -> Result<Schema> {
        Schema {
            version: 1,
            block: Block::empty(),
        }
        .into()
    }

    /// Get the resources of the provider
    fn get_resources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicDataSource>>>;
}

pub trait DynamicProvider: Send + Sync + 'static {
    /// Get the schema of the provider
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the provider
    fn validate(&self, config: DynamicValue) -> Result<()>;
    /// Configure the provider
    fn configure(&self, version: i64, config: DynamicValue) -> Result<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self) -> Result<Schema> {
        Schema {
            version: 1,
            block: Block::empty(),
        }
        .into()
    }

    /// Get the resources of the provider
    fn get_resources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicDataSource>>>;
}

impl<T: Provider> DynamicProvider for T {
    /// Get the schema of the provider
    fn schema(&self) -> Result<Schema> {
        <T as Provider>::schema(self)
    }
    /// Validate the configuration of the provider
    fn validate(&self, config: DynamicValue) -> Result<()> {
        let config = get!(config.deserialize());
        <T as Provider>::validate(self, config)
    }
    /// Configure the provider
    fn configure(&self, version: i64, config: DynamicValue) -> Result<()> {
        let config = get!(config.deserialize());
        <T as Provider>::configure(self, version, config)
    }

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self) -> Result<Schema> {
        <T as Provider>::meta_schema(self)
    }

    /// Get the resources of the provider
    fn get_resources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicResource>>> {
        <T as Provider>::get_resources(self)
    }

    /// Get the data sources of the provider
    fn get_data_sources<'a>(&'a self) -> Result<&'a HashMap<String, Box<dyn DynamicDataSource>>> {
        <T as Provider>::get_data_sources(self)
    }
}
