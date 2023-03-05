use crate::attribute::AttributePath;
use crate::diagnostics::Result;
use crate::schema::Schema;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a resource
pub trait Resource {
    /// State of the resource
    type State: Serialize + DeserializeOwned;
    /// Private state of the resource
    type PrivateState: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned;

    /// Get the schema of the resource
    fn schema(&mut self) -> Result<Schema>;
    /// Validate the configuration of the resource
    fn validate(&mut self, config: Self::State) -> Result<()>;
    /// Read the new state of the resource
    fn read(
        &mut self,
        state: Self::State,
        private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState)>;
    /// Plan the changes on the resource
    fn plan(
        &mut self,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState, Vec<AttributePath>)>;
    /// Apply the changes on the resource
    fn apply(
        &mut self,
        prior_state: Self::State,
        planned_state: Self::State,
        config_state: Self::State,
        planned_private_state: Self::PrivateState,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<(Self::State, Self::PrivateState)>;
    fn import(&mut self, id: String) -> Result<(Self::State, Self::PrivateState)>;
    fn upgrade(&mut self, version: i64, prior_state: Self::State) -> Result<Self::State>;
}
