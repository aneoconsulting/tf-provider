use std::fmt::Debug;

use async_trait::async_trait;

use tf_provider::{AttributePath, DataSource, Diagnostics, Schema, Value, ValueEmpty};

use crate::connection::Connection;
use crate::utils::WithSchema;

use super::prepare_envs;
use super::state::DataSourceState;

#[derive(Debug, Default)]
pub struct CmdExecDataSource<T: Connection> {
    pub(super) connect: T,
}

impl<T: Connection> CmdExecDataSource<T> {
    pub fn new(connect: T) -> Self {
        Self { connect }
    }
}

#[async_trait]
impl<T> DataSource for CmdExecDataSource<T>
where
    T: Connection,
    T: Debug,
    T: Clone,
{
    type State<'a> = DataSourceState<'a, T>;
    type ProviderMetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(DataSourceState::<T>::schema())
    }

    async fn validate<'a>(&self, diags: &mut Diagnostics, config: Self::State<'a>) -> Option<()> {
        self.validate(diags, &config, AttributePath::default())
            .await;

        if diags.errors.is_empty() {
            Some(())
        } else {
            None
        }
    }

    async fn read<'a>(
        &self,
        diags: &mut Diagnostics,
        config: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<Self::State<'a>> {
        let state_env = prepare_envs(&[(&config.inputs, "INPUT_")]);

        let mut state = config.clone();

        // Mark all values unknown to force their read
        state.outputs = Value::Value(
            state
                .read
                .iter()
                .flatten()
                .map(|(name, _)| (name.clone(), Value::Unknown))
                .collect(),
        );

        state.read(diags, &self.connect, &state_env).await;

        Some(state)
    }
}
