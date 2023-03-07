use crate::diagnostics::Diagnostics;
use crate::dynamic::DynamicValue;
use crate::schema::Schema;
use crate::utils::OptionFactor;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a data source
pub trait DataSource: Send + Sync {
    /// State of the data source
    type State: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned;

    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()>;
    /// Read the new state of the data source
    fn read(
        &self,
        diags: &mut Diagnostics,
        config: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<Self::State>;
}

pub trait DynamicDataSource: Send + Sync {
    /// Get the schema of the data source
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the data source
    fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()>;
    /// Read the new state of the data source
    fn read(
        &self,
        diags: &mut Diagnostics,
        config: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Option<DynamicValue>;
}

impl<T: DataSource> DynamicDataSource for T {
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as DataSource>::schema(self, diags)
    }
    fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as DataSource>::validate(self, diags, config)
    }
    fn read(
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
        let state = <T as DataSource>::read(self, diags, config, provider_meta_state)?;
        DynamicValue::serialize(diags, &state)
    }
}
