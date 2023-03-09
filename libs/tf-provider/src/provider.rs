use std::collections::HashMap;

use crate::data_source::DynamicDataSource;
use crate::diagnostics::Diagnostics;
use crate::raw::RawValue;
use crate::resource::DynamicResource;
use crate::schema::Block;
use crate::schema::Schema;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a provider
#[async_trait]
pub trait Provider: Send + Sync + 'static {
    /// Configuration of the provider
    type Config: Serialize + DeserializeOwned + Send;
    /// State of the provider metadata
    type MetaState: Serialize + DeserializeOwned + Send;

    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;

    /// Validate the configuration of the provider
    async fn validate(&self, diags: &mut Diagnostics, config: Self::Config) -> Option<()>;

    /// Configure the provider
    async fn configure(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: Self::Config,
    ) -> Option<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        _ = diags;
        Some(Schema {
            version: 1,
            block: Block::empty(),
        })
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>>;
}

#[async_trait]
pub trait DynamicProvider: Send + Sync + 'static {
    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;

    /// Validate the configuration of the provider
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()>;

    /// Configure the provider
    async fn configure(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: RawValue,
    ) -> Option<()>;

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        _ = diags;
        Some(Schema {
            version: 1,
            block: Block::empty(),
        })
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>>;

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>>;
}

#[async_trait]
impl<T: Provider> DynamicProvider for T {
    /// Get the schema of the provider
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Provider>::schema(self, diags)
    }

    /// Validate the configuration of the provider
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Provider>::validate(self, diags, config).await
    }

    /// Configure the provider
    async fn configure(
        &self,
        diags: &mut Diagnostics,
        terraform_version: String,
        config: RawValue,
    ) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Provider>::configure(self, diags, terraform_version, config).await
    }

    /// Get the schema for the provider metadata (defaults to empty)
    fn meta_schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Provider>::meta_schema(self, diags)
    }

    /// Get the resources of the provider
    fn get_resources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicResource>>> {
        <T as Provider>::get_resources(self, diags)
    }

    /// Get the data sources of the provider
    fn get_data_sources(
        &self,
        diags: &mut Diagnostics,
    ) -> Option<HashMap<String, Box<dyn DynamicDataSource>>> {
        <T as Provider>::get_data_sources(self, diags)
    }
}
