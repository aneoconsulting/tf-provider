use crate::attribute_path::AttributePath;
use crate::diagnostics::Diagnostics;
use crate::raw::RawValue;
use crate::schema::Schema;
use crate::utils::OptionFactor;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a resource
#[async_trait]
pub trait Resource: Send + Sync {
    /// State of the resource
    type State: Serialize + DeserializeOwned + Send;
    /// Private state of the resource
    type PrivateState: Serialize + DeserializeOwned + Send;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned + Send;

    /// Get the schema of the resource
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the resource
    async fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()>;
    /// Read the new state of the resource
    async fn read(
        &self,
        diags: &mut Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Plan the changes on the resource
    async fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    async fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Import an existing resource
    async fn import(
        &self,
        diags: &mut Diagnostics,
        id: String,
    ) -> Option<(Self::State, Self::PrivateState)> {
        _ = id;
        diags.root_error_short("Import is not supported");
        None
    }
    /// Upgrade the resource
    async fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: RawValue,
    ) -> Option<Self::State> {
        _ = version;
        _ = prior_state;
        diags.root_error_short("Upgrade is not supported");
        None
    }
}

#[async_trait]
pub trait DynamicResource: Send + Sync {
    /// Get the schema of the resource
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema>;
    /// Validate the configuration of the resource
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()>;
    /// Read the new state of the resource
    async fn read(
        &self,
        diags: &mut Diagnostics,
        state: RawValue,
        private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)>;
    /// Plan the changes on the resource
    async fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        proposed_state: RawValue,
        config_state: RawValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    async fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        planned_state: RawValue,
        config_state: RawValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)>;
    /// Import an existing resource
    async fn import(&self, diags: &mut Diagnostics, id: String) -> Option<(RawValue, Vec<u8>)> {
        _ = id;
        diags.root_error_short("Import is not supported");
        None
    }
    /// Upgrade the resource
    async fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: RawValue,
    ) -> Option<RawValue> {
        _ = version;
        _ = prior_state;
        diags.root_error_short("Upgrade is not supported");
        None
    }
}

#[async_trait]
impl<T: Resource> DynamicResource for T {
    /// Get the schema of the resource
    fn schema(&self, diags: &mut Diagnostics) -> Option<Schema> {
        <T as Resource>::schema(self, diags)
    }
    /// Validate the configuration of the resource
    async fn validate(&self, diags: &mut Diagnostics, config: RawValue) -> Option<()> {
        let config = config.deserialize(diags)?;
        <T as Resource>::validate(self, diags, config).await
    }
    /// Read the new state of the resource
    async fn read(
        &self,
        diags: &mut Diagnostics,
        state: RawValue,
        private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)> {
        let (state, private_state, provider_meta_state) = (
            state.deserialize(diags),
            RawValue::MessagePack(private_state).deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;

        let (state, private_state) =
            <T as Resource>::read(self, diags, state, private_state, provider_meta_state).await?;

        (
            RawValue::serialize(diags, &state),
            RawValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Plan the changes on the resource
    async fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        proposed_state: RawValue,
        config_state: RawValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>, Vec<AttributePath>)> {
        let (prior_state, proposed_state, config_state, prior_private_state, provider_meta_state) =
            (
                prior_state.deserialize(diags),
                proposed_state.deserialize(diags),
                config_state.deserialize(diags),
                RawValue::MessagePack(prior_private_state).deserialize(diags),
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
        )
        .await?;

        (
            RawValue::serialize(diags, &state),
            RawValue::serialize_vec(diags, &private_state),
            Some(destroy_triggers),
        )
            .factor()
    }
    /// Apply the changes on the resource
    async fn apply(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        planned_state: RawValue,
        config_state: RawValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)> {
        let (prior_state, planned_state, config_state, planned_private_state, provider_meta_state) =
            (
                prior_state.deserialize(diags),
                planned_state.deserialize(diags),
                config_state.deserialize(diags),
                RawValue::MessagePack(planned_private_state).deserialize(diags),
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
        )
        .await?;
        (
            RawValue::serialize(diags, &state),
            RawValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Import an existing resource
    async fn import(&self, diags: &mut Diagnostics, id: String) -> Option<(RawValue, Vec<u8>)> {
        let (state, private_state) = <T as Resource>::import(self, diags, id).await?;
        (
            RawValue::serialize(diags, &state),
            RawValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Upgrade the resource
    async fn upgrade(
        &self,
        diags: &mut Diagnostics,
        version: i64,
        prior_state: RawValue,
    ) -> Option<RawValue> {
        let state = <T as Resource>::upgrade(self, diags, version, prior_state).await?;
        RawValue::serialize(diags, &state)
    }
}
