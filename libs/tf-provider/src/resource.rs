use crate::attribute_path::AttributePath;
use crate::diagnostics::Diagnostics;
use crate::dynamic::DynamicValue;
use crate::schema::Schema;
use crate::utils::OptionFactor;

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
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the resource
    fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()>;
    /// Read the new state of the resource
    fn read(
        &self,
        diags: &mut Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Plan the changes on the resource
    fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Import an existing resource
    fn import(
        &self,
        diags: &mut Diagnostics,
        id: String,
    ) -> Option<(Self::State, Self::PrivateState)> {
        _ = id;
        diags.root_error_short("Import is not supported");
        None
    }
    /// Upgrade the resource
    fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: DynamicValue,
    ) -> Option<Self::State> {
        _ = version;
        _ = prior_state;
        diags.root_error_short("Upgrade is not supported");
        None
    }
}

pub trait DynamicResource: Send + Sync {
    /// Get the schema of the resource
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the resource
    fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()>;
    /// Read the new state of the resource
    fn read(
        &self,
        diags: &mut Diagnostics,
        state: DynamicValue,
        private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>)>;
    /// Plan the changes on the resource
    fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: DynamicValue,
        proposed_state: DynamicValue,
        config_state: DynamicValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: DynamicValue,
        planned_state: DynamicValue,
        config_state: DynamicValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>)>;
    /// Import an existing resource
    fn import(&self, diags: &mut Diagnostics, id: String) -> Option<(DynamicValue, Vec<u8>)> {
        _ = id;
        diags.root_error_short("Import is not supported");
        None
    }
    /// Upgrade the resource
    fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: DynamicValue,
    ) -> Option<DynamicValue> {
        _ = version;
        _ = prior_state;
        diags.root_error_short("Upgrade is not supported");
        None
    }
}

impl<T: Resource> DynamicResource for T {
    /// Get the schema of the resource
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Resource>::schema(self, diags)
    }
    /// Validate the configuration of the resource
    fn validate(&self, diags: &mut Diagnostics, config: DynamicValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Resource>::validate(self, diags, config)
    }
    /// Read the new state of the resource
    fn read(
        &self,
        diags: &mut Diagnostics,
        state: DynamicValue,
        private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>)> {
        let (state, private_state, provider_meta_state) = (
            state.deserialize(diags),
            DynamicValue::MessagePack(private_state).deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;

        let (state, private_state) =
            <T as Resource>::read(self, diags, state, private_state, provider_meta_state)?;

        (
            DynamicValue::serialize(diags, &state),
            DynamicValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Plan the changes on the resource
    fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: DynamicValue,
        proposed_state: DynamicValue,
        config_state: DynamicValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>, Vec<AttributePath>)> {
        let (prior_state, proposed_state, config_state, prior_private_state, provider_meta_state) =
            (
                prior_state.deserialize(diags),
                proposed_state.deserialize(diags),
                config_state.deserialize(diags),
                DynamicValue::MessagePack(prior_private_state).deserialize(diags),
                provider_meta_state.deserialize(diags),
            )
                .factor()?;

        let (state, private_state, destroy_triggers) = <T as Resource>::plan(
            self,
            diags,
            prior_state,
            proposed_state,
            config_state,
            prior_private_state,
            provider_meta_state,
        )?;

        (
            DynamicValue::serialize(diags, &state),
            DynamicValue::serialize_vec(diags, &private_state),
            Some(destroy_triggers),
        )
            .factor()
    }
    /// Apply the changes on the resource
    fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: DynamicValue,
        planned_state: DynamicValue,
        config_state: DynamicValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: DynamicValue,
    ) -> Option<(DynamicValue, Vec<u8>)> {
        let (prior_state, planned_state, config_state, planned_private_state, provider_meta_state) =
            (
                prior_state.deserialize(diags),
                planned_state.deserialize(diags),
                config_state.deserialize(diags),
                DynamicValue::MessagePack(planned_private_state).deserialize(diags),
                provider_meta_state.deserialize(diags),
            )
                .factor()?;
        let (state, private_state) = <T as Resource>::apply(
            self,
            diags,
            prior_state,
            planned_state,
            config_state,
            planned_private_state,
            provider_meta_state,
        )?;
        (
            DynamicValue::serialize(diags, &state),
            DynamicValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Import an existing resource
    fn import(&self, diags: &mut Diagnostics, id: String) -> Option<(DynamicValue, Vec<u8>)> {
        let (state, private_state) = <T as Resource>::import(self, diags, id)?;
        (
            DynamicValue::serialize(diags, &state),
            DynamicValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Upgrade the resource
    fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: DynamicValue,
    ) -> Option<DynamicValue> {
        let state = <T as Resource>::upgrade(self, diags, version, prior_state)?;
        DynamicValue::serialize(diags, &state)
    }
}
