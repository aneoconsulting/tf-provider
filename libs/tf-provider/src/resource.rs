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
    /// Plan the creation of a new resource
    async fn plan_create(
        &self,
        diags: &mut Diagnostics,
        proposed_state: Self::State,
        config_state: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Plan the changes on the resource
    async fn plan_update(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState, Vec<AttributePath>)>;
    /// Plan the destruction of the resource
    async fn plan_destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<()>;
    /// Create a new resource
    async fn create(
        &self,
        diags: &mut Diagnostics,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Apply the changes on the resource
    async fn update(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)>;
    /// Destroy the resource
    async fn destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Option<()>;
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
    /// Plan the creation of a new resource
    async fn plan_create(
        &self,
        diags: &mut Diagnostics,
        proposed_state: RawValue,
        config_state: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)>;
    /// Plan the changes on the resource
    async fn plan_update(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        proposed_state: RawValue,
        config_state: RawValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>, Vec<AttributePath>)>;
    /// Plan the destruction of the resource
    async fn plan_destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<()>;
    /// Create a new resource
    async fn create(
        &self,
        diags: &mut Diagnostics,
        planned_state: RawValue,
        config_state: RawValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)>;
    /// Apply the changes on the resource
    async fn update(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        planned_state: RawValue,
        config_state: RawValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)>;
    /// Destroy the resource
    async fn destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<()>;
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
    /// Plan the creation of a new resource
    async fn plan_create(
        &self,
        diags: &mut Diagnostics,
        proposed_state: RawValue,
        config_state: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)> {
        let (proposed_state, config_state, provider_meta_state) = (
            proposed_state.deserialize(diags),
            config_state.deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;

        let (state, private_state) = <T as Resource>::plan_create(
            self,
            diags,
            proposed_state,
            config_state,
            provider_meta_state,
        )
        .await?;

        (
            RawValue::serialize(diags, &state),
            RawValue::serialize_vec(diags, &private_state),
        )
            .factor()
    }
    /// Plan the changes on the resource
    async fn plan_update(
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

        let (state, private_state, destroy_triggers) = <T as Resource>::plan_update(
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
    /// Plan the destruction of the resource
    async fn plan_destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        prior_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<()> {
        let (prior_state, prior_private_state, provider_meta_state) = (
            prior_state.deserialize(diags),
            RawValue::MessagePack(prior_private_state).deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;

        <T as Resource>::plan_destroy(
            self,
            diags,
            prior_state,
            prior_private_state,
            provider_meta_state,
        )
        .await
    }
    /// Create a new resource
    async fn create(
        &self,
        diags: &mut Diagnostics,
        planned_state: RawValue,
        config_state: RawValue,
        planned_private_state: Vec<u8>,
        provider_meta_state: RawValue,
    ) -> Option<(RawValue, Vec<u8>)> {
        let (planned_state, config_state, planned_private_state, provider_meta_state) = (
            planned_state.deserialize(diags),
            config_state.deserialize(diags),
            RawValue::MessagePack(planned_private_state).deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;
        let (state, private_state) = <T as Resource>::create(
            self,
            diags,
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
    /// Apply the changes on the resource
    async fn update(
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
        let (state, private_state) = <T as Resource>::update(
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
    /// Destroy the resource
    async fn destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: RawValue,
        provider_meta_state: RawValue,
    ) -> Option<()> {
        let (prior_state, provider_meta_state) = (
            prior_state.deserialize(diags),
            provider_meta_state.deserialize(diags),
        )
            .factor()?;
        <T as Resource>::destroy(self, diags, prior_state, provider_meta_state).await
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

impl<T: Resource + 'static> From<T> for Box<dyn DynamicResource> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}
