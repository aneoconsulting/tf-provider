use crate::diagnostics::Result;
use crate::schema::Schema;

use serde::{de::DeserializeOwned, Serialize};

/// Trait for implementing a data source
trait DataSource {
    /// State of the data source
    type State: Serialize + DeserializeOwned;
    /// State of the provider metadata
    type ProviderMetaState: Serialize + DeserializeOwned;

    /// Get the schema of the data source
    fn schema(&mut self) -> Result<Schema>;
    /// Validate the configuration of the data source
    fn validate(&mut self, config: Self::State) -> Result<()>;
    /// Read the new state of the data source
    fn read(
        &mut self,
        config: Self::State,
        provider_meta_state: Self::ProviderMetaState,
    ) -> Result<Self::State>;
}
