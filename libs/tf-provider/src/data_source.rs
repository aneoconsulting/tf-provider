use crate::schema::Schema;
use crate::{
    dynamic::DynamicValue,
    result::{get, Result},
};

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a data source
trait DataSource: Send + Sync {
    /// State of the data source
    type State: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned;

    /// Get the schema of the data source
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the data source
    fn validate(&self, config: Self::State) -> Result<()>;
    /// Read the new state of the data source
    fn read(
        &self,
        config: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<Self::State>;
}

pub trait DynamicDataSource: Send + Sync {
    /// Get the schema of the data source
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the data source
    fn validate(&self, config: DynamicValue) -> Result<()>;
    /// Read the new state of the data source
    fn read(&self, config: DynamicValue, provider_meta_state: DynamicValue)
        -> Result<DynamicValue>;
}

impl<T: DataSource> DynamicDataSource for T {
    fn schema(&self) -> Result<Schema> {
        <T as DataSource>::schema(self)
    }
    fn validate(&self, config: DynamicValue) -> Result<()> {
        let config = get!(config.deserialize());
        <T as DataSource>::validate(self, config)
    }
    fn read(
        &self,
        config: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<DynamicValue> {
        let config = get!(config.deserialize());
        let provider_meta_state = get!(provider_meta_state.deserialize());
        let state = get!(<T as DataSource>::read(self, config, provider_meta_state));
        DynamicValue::serialize(&state)
    }
}
