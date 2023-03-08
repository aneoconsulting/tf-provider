use crate::diagnostics::Diagnostics;
use crate::dynamic::DynamicValue;
use crate::schema::Schema;
use crate::utils::OptionFactor;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
/// Trait for implementing a data source
pub trait DataSource: Send + Sync {
    /// State of the data source
    type State: Serialize + DeserializeOwned + Send;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned + Send;

    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    async fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()>;
    /// Read the new state of the data source
    async fn read(
        &self,
        diags: &mut Diagnostics,
        config: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<Self::State>;
}

#[async_trait]
pub trait DynamicDataSource: Send + Sync {
    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    async fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()>;
    /// Read the new state of the data source
    async fn read(
        &self,
        diags: &mut Diagnostics,
        config: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Option<DynamicValue>;
}

#[async_trait]
impl<T: DataSource> DynamicDataSource for T {
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as DataSource>::schema(self, diags)
    }
    async fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as DataSource>::validate(self, diags, config).await
    }
    async fn read(
        &self,
        diags: &mut Diagnostics,
        config: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Option<DynamicValue> {
        let (config, provider_meta_state) = (
            config.deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;
        let state = <T as DataSource>::read(self, diags, config, provider_meta_state).await?;
        DynamicValue::serialize(diags, &state)
    }
}
