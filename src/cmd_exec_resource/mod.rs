use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use tf_provider::{AttributePath, Diagnostics, Resource, Schema, Value, ValueEmpty};

use crate::connection::Connection;
use crate::utils::WithSchema;
use crate::utils::WithValidate;

mod state;
mod validate;

use state::State;

#[derive(Debug, Default)]
pub struct CmdExecResource<T: Connection> {
    ph: PhantomData<T>,
}

#[async_trait]
impl<T> Resource for CmdExecResource<T>
where
    T: Connection,
    T: for<'e> Deserialize<'e>,
    T: Serialize,
    T: Debug,
    T: Clone,
{
    type State = Value<State<T>>;
    type PrivateState = ValueEmpty;
    type ProviderMetaState = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(State::<T>::schema())
    }

    async fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()> {
        let Value::Value(config) = &config else {
            return Some(());
        };

        config.validate(diags, Default::default()).await;

        if diags.errors.len() == 0 {
            Some(())
        } else {
            None
        }
    }

    async fn read(
        &self,
        diags: &mut Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        let mut state = state;
        if let Value::Value(ref mut state) = state {
            let connection_default = Default::default();
            let connection = state.connection.as_ref().unwrap_or(&connection_default);

            if let Value::Value(ref reads) = state.read {
                let mut outputs = if let Value::Value(state) = std::mem::take(&mut state.state) {
                    state
                } else {
                    HashMap::with_capacity(reads.len())
                };
                let attr_path = AttributePath::new("read");

                for (name, read) in reads {
                    if let Value::Value(read) = read {
                        let attr_path = attr_path.clone().key(name).key("cmd");

                        let default_env = HashMap::default();
                        let cmd = read.cmd.as_ref().map(|v| v.into()).unwrap_or_default();
                        let env = read
                            .env
                            .as_ref()
                            .unwrap_or(&default_env)
                            .iter()
                            .map(|(k, v)| {
                                (k.into(), v.as_ref().map(|v| v.into()).unwrap_or_default())
                            })
                            .collect();

                        match connection.execute(cmd, env).await {
                            Ok(res) => {
                                if res.status == 0 {
                                    if res.stderr.len() > 0 {
                                        diags.warning(
                                            "`read` succeeded but stderr was not empty",
                                            std::str::from_utf8(res.stderr.as_slice())
                                                .unwrap_or("<stderr is not a valid utf-8 and cannot be printed>"),
                                            attr_path.clone(),
                                        );
                                    }

                                    match std::str::from_utf8(res.stdout.as_slice()) {
                                        Ok(stdout) => {
                                            _ = outputs
                                                .insert(name.clone(), Value::Value(stdout.into()));
                                        }
                                        Err(err) => diags.error("`read` succeeded, but stdout was not a valid utf-8 string", err, attr_path.clone())
                                    }
                                } else {
                                    diags.error(
                                        format!("`read` failed with status code: {}", res.status),
                                        std::str::from_utf8(res.stderr.as_slice()).unwrap_or(
                                            "<stderr is not a valid utf-8 and cannot be printed>",
                                        ),
                                        attr_path,
                                    );
                                }
                            }
                            Err(err) => {
                                diags.error("Failed to read resource state", err, attr_path);
                            }
                        };
                    }
                }
                state.state = Value::Value(outputs);
            }
        }
        Some((state, private_state))
    }

    async fn plan(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        proposed_state: Self::State,
        config_state: Self::State,
        prior_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(
        Self::State,
        Self::PrivateState,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        diags.root_warning("prior_state", format!("{:#?}", prior_state));
        diags.root_warning("proposed_state", format!("{:#?}", proposed_state));
        diags.root_warning("config_state", format!("{:#?}", config_state));

        let state = match proposed_state {
            Value::Value(proposed_state) => {
                let mut state = proposed_state.clone();
                if state.id.is_null() {
                    state.id = Value::Unknown;
                }
                Value::Value(state)
            }
            Value::Null => Value::Null, // destruction
            Value::Unknown => Value::Unknown,
        };

        Some((state, prior_private_state, vec![]))
        //Some((State::default().into(), prior_private_state, vec![]))
    }

    async fn apply(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        let state = match planned_state {
            Value::Value(planned_state) => {
                let mut state = planned_state.clone();
                if !state.id.is_value() {
                    state.id = Value::Value(
                        thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(30)
                            .map(char::from)
                            .collect(),
                    );
                }
                Value::Value(state)
            }
            Value::Null => Value::Null, // destruction
            Value::Unknown => Value::Unknown,
        };

        Some((state, planned_private_state))
    }
}
