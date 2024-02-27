use crate::diagnostics::Diagnostics;
use crate::raw::RawValue;
use crate::schema::Schema;
use crate::utils::OptionFactor;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
/// Trait for implementing a data source
pub trait DataSource: Send + Sync {
    /// State of the data source
    type State<'a>: Serialize + Deserialize<'a> + Send;
    /// State of the provider metadata
    type ProviderMetaState<'a>: Serialize + Deserialize<'a> + Send;

    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    async fn validate<'a>(&self, diags: &mut Diagnostics, config: Self::State<'a>) -> Option<()>;
    /// Read the new state of the data source
    async fn read<'a>(
        &self,
        diags: &mut Diagnostics,
        config: Self::State<'a>,
        provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<Self::State<'a>>;
}

#[async_trait]
pub trait DynamicDataSource: Send + Sync {
    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()>;
    /// Read the new state of the data source
    async fn read(
        &self,
        diags: &mut Diagnostics,
        config: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<RawValue>;
}

#[async_trait]
impl<T: DataSource> DynamicDataSource for T {
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as DataSource>::schema(self, diags)
    }
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as DataSource>::validate(self, diags, config).await
    }
    async fn read(
        &self,
        diags: &mut Diagnostics,
        config: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<RawValue> {
        let (config, provider_meta_state) = (
            config.deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;
        let state = <T as DataSource>::read(self, diags, config, provider_meta_state).await?;
        RawValue::serialize(diags, &state)
    }
}

impl<T: DataSource + 'static> From<T> for Box<dyn DynamicDataSource> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
