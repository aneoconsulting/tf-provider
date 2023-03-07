use crate::result::{get, Result};
use crate::schema::Schema;
use crate::{attribute_path::AttributePath, dynamic::DynamicValue};

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a resource
pub trait Resource: Send + Sync {
    /// State of the resource
    type State: Serialize + DeserializeOwned;
    /// Private state of the resource
    type PrivateState: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned;

    /// Get the schema of the resource
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the resource
    fn validate(&self, config: Self::State) -> Result<()>;
    /// Read the new state of the resource
    fn read(
        &self,
        state: Self::State,
        private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState)>;
    /// Plan the changes on the resource
    fn plan(
        &self,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    fn apply(
        &self,
        prior_state: Self::State,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState)>;
    /// Import an existing resource
    fn import(&self, id: String) -> Result<(Self::State, Self::PrivateState)> {
        _ = id;
        Result::from_error("Import is not supported")
    }
    /// Upgrade the resource
    fn upgrade(&self, version: i64, prior_state: DynamicValue) -> Result<Self::State> {
        _ = version;
        _ = prior_state;
        Result::from_error("Upgrade is not supported")
    }
}

pub trait DynamicResource: Send + Sync {
    /// Get the schema of the resource
    fn schema(&self) -> Result<Schema>;
    /// Validate the configuration of the resource
    fn validate(&self, config: DynamicValue) -> Result<()>;
    /// Read the new state of the resource
    fn read(
        &self,
        state: DynamicValue,
        private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue)>;
    /// Plan the changes on the resource
    fn plan(
        &self,
        prior_state: DynamicValue,
        proposed_state: DynamicValue,
        config_state: DynamicValue,
        prior_private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    fn apply(
        &self,
        prior_state: DynamicValue,
        planned_state: DynamicValue,
        config_state: DynamicValue,
        planned_private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue)>;
    fn import(&self, id: String) -> Result<(DynamicValue, DynamicValue)>;
    fn upgrade(&self, version: i64, prior_state: DynamicValue) -> Result<DynamicValue>;
}

impl<T: Resource> DynamicResource for T {
    /// Get the schema of the resource
    fn schema(&self) -> Result<Schema> {
        <T as Resource>::schema(self)
    }
    /// Validate the configuration of the resource
    fn validate(&self, config: DynamicValue) -> Result<()> {
        let config = get!(config.deserialize());
        <T as Resource>::validate(self, config)
    }
    /// Read the new state of the resource
    fn read(
        &self,
        state: DynamicValue,
        private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue)> {
        let state = get!(state.deserialize());
        let private_state = get!(private_state.deserialize());
        let provider_meta_state = get!(provider_meta_state.deserialize());
        let (state, private_state) = get!(<T as Resource>::read(
            self,
            state,
            private_state,
            provider_meta_state
        ));
        let state = get!(DynamicValue::serialize(&state));
        let private_state = get!(DynamicValue::serialize(&private_state));
        Result::from((state, private_state))
    }
    /// Plan the changes on the resource
    fn plan(
        &self,
        prior_state: DynamicValue,
        proposed_state: DynamicValue,
        config_state: DynamicValue,
        prior_private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue, Vec<AttributePath>)> {
        let prior_state = get!(prior_state.deserialize());
        let proposed_state = get!(proposed_state.deserialize());
        let config_state = get!(config_state.deserialize());
        let prior_private_state = get!(prior_private_state.deserialize());
        let provider_meta_state = get!(provider_meta_state.deserialize());
        let (state, private_state, destroy_triggers) = get!(<T as Resource>::plan(
            self,
            prior_state,
            proposed_state,
            config_state,
            prior_private_state,
            provider_meta_state
        ));
        let state = get!(DynamicValue::serialize(&state));
        let private_state = get!(DynamicValue::serialize(&private_state));
        Result::from((state, private_state, destroy_triggers))
    }
    /// Apply the changes on the resource
    fn apply(
        &self,
        prior_state: DynamicValue,
        planned_state: DynamicValue,
        config_state: DynamicValue,
        planned_private_state: DynamicValue,
        provider_meta_state: DynamicValue,
    ) -> Result<(DynamicValue, DynamicValue)> {
        let prior_state = get!(prior_state.deserialize());
        let planned_state = get!(planned_state.deserialize());
        let config_state = get!(config_state.deserialize());
        let planned_private_state = get!(planned_private_state.deserialize());
        let provider_meta_state = get!(provider_meta_state.deserialize());
        let (state, private_state) = get!(<T as Resource>::apply(
            self,
            prior_state,
            planned_state,
            config_state,
            planned_private_state,
            provider_meta_state
        ));
        let state = get!(DynamicValue::serialize(&state));
        let private_state = get!(DynamicValue::serialize(&private_state));
        Result::from((state, private_state))
    }
    fn import(&self, id: String) -> Result<(DynamicValue, DynamicValue)> {
        let (state, private_state) = get!(<T as Resource>::import(self, id));
        let state = get!(DynamicValue::serialize(&state));
        let private_state = get!(DynamicValue::serialize(&private_state));
        Result::from((state, private_state))
    }
    fn upgrade(&self, version: i64, prior_state: DynamicValue) -> Result<DynamicValue> {
        let state = get!(<T as Resource>::upgrade(self, version, prior_state));
        DynamicValue::serialize(&state)
    }
}
