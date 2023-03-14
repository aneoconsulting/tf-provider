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
    type State = State<T>;
    type PrivateState = ValueEmpty;
    type ProviderMetaState = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(State::<T>::schema())
    }

    async fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()> {
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

        let connection_default = Default::default();
        let connection = state.connection.as_ref().unwrap_or(&connection_default);

        let state_env: Vec<_> =
            state
                .inputs
                .iter()
                .flatten()
                .filter_map(|(k, v)| Some((format!("INPUT_{k}"), v.as_ref_option()?.as_str())))
                .chain(
                    state.inputs.iter().flatten().filter_map(|(k, v)| {
                        Some((format!("STATE_{k}"), v.as_ref_option()?.as_str()))
                    }),
                )
                .collect();

        if let Value::Value(ref reads) = state.read {
            let mut outputs = state
                .state
                .unwrap_or_else(|| HashMap::with_capacity(reads.len()));
            let attr_path = AttributePath::new("read");

            for (name, read) in reads {
                if let Value::Value(read) = read {
                    let attr_path = attr_path.clone().key(name).key("cmd");

                    let cmd = read.cmd.as_ref().map(String::as_ref).unwrap_or_default();
                    let env = read
                        .env
                        .iter()
                        .flatten()
                        .filter_map(|(cmd, env)| {
                            Some((cmd.as_str(), env.as_ref_option()?.as_str()))
                        })
                        .chain(state_env.iter().map(|(k, v)| (k.as_str(), *v)));

                    match connection.execute(cmd, env).await {
                        Ok(res) => {
                            if res.status == 0 {
                                if res.stderr.len() > 0 {
                                    diags.warning(
                                        "`read` succeeded but stderr was not empty",
                                        res.stderr,
                                        attr_path,
                                    );
                                }

                                outputs.insert(name.clone(), Value::Value(res.stdout));
                            } else {
                                diags.warning(
                                    format!("`read` failed with status code: {}", res.status),
                                    res.stderr,
                                    attr_path,
                                );
                            }
                        }
                        Err(err) => {
                            diags.warning("Failed to read resource state", err, attr_path);
                        }
                    };
                }
            }
            state.state = Value::Value(outputs);
        }

        if state.inputs.is_null() {
            state.inputs = Value::Value(Default::default());
        }

        Some((state, private_state))
    }

    async fn plan_create(
        &self,
        diags: &mut Diagnostics,
        proposed_state: Self::State,
        config_state: Self::State,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        diags.root_warning("proposed_state", format!("{:#?}", proposed_state));
        diags.root_warning("config_state", format!("{:#?}", config_state));

        let mut state = proposed_state.clone();
        state.id = Value::Unknown;

        if state.inputs.is_null() {
            state.inputs = Value::Value(Default::default());
        }

        match &state.read {
            Value::Value(reads) => {
                let mut outputs = HashMap::with_capacity(reads.len());
                for (k, _) in reads {
                    outputs.insert(k.clone(), Value::Unknown);
                }
                state.state = Value::Value(outputs);
            }
            Value::Null => {
                state.read = Value::Value(Default::default());
                state.state = Value::Value(Default::default());
            }
            Value::Unknown => {
                state.state = Value::Unknown;
            }
        }

        Some((state, Default::default()))
    }
    async fn plan_update(
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

        let mut state = proposed_state.clone();
        if state.id.is_null() {
            state.id = Value::Unknown;
        }
        if state.inputs.is_null() {
            state.inputs = Value::Value(Default::default());
        }

        Some((state, prior_private_state, vec![]))
    }

    async fn plan_destroy(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State,
        _prior_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<()> {
        diags.root_warning("prior_state", format!("{:#?}", prior_state));
        Some(())
    }

    async fn create(
        &self,
        diags: &mut Diagnostics,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        let mut state = planned_state.clone();
        state.id = Value::Value(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
        );
        if !state.inputs.is_value() {
            state.inputs = Value::Value(Default::default());
        }

        let connection_default = Default::default();
        let connection = state.connection.as_ref().unwrap_or(&connection_default);

        let state_env: Vec<_> = state
            .inputs
            .iter()
            .flatten()
            .filter_map(|(k, v)| Some((format!("INPUT_{k}"), v.as_ref_option()?.as_str())))
            .collect();

        let create_cmd = state
            .create
            .as_ref()
            .and_then(|create| create.cmd.as_ref())
            .map(String::as_str)
            .unwrap_or_default();
        if create_cmd != "" {
            let attr_path = AttributePath::new("create").index(0).attribute("cmd");
            match connection
                .execute(create_cmd, state_env.iter().map(|(k, v)| (k.as_str(), *v)))
                .await
            {
                Ok(res) => {
                    if res.stdout.len() > 0 {
                        diags.warning(
                            "`create` stdout was not empty",
                            res.stdout,
                            attr_path.clone(),
                        );
                    }
                    if res.status == 0 {
                        if res.stderr.len() > 0 {
                            diags.warning(
                                "`create` succeeded but stderr was not empty",
                                res.stderr,
                                attr_path,
                            );
                        }
                    } else {
                        diags.error(
                            format!("`create` failed with status code: {}", res.status),
                            res.stderr,
                            attr_path,
                        );
                    }
                }
                Err(err) => {
                    diags.error("Failed to create resource", err, attr_path);
                }
            }
        }

        if diags.errors.len() > 0 {
            return None;
        }

        // Read all outputs after the resource creation
        if let Value::Value(ref reads) = state.read {
            let mut outputs = HashMap::with_capacity(reads.len());
            let attr_path = AttributePath::new("read");

            for (name, read) in reads {
                if let Value::Value(read) = read {
                    let attr_path = attr_path.clone().key(name).key("cmd");

                    let cmd = read.cmd.as_ref().map(String::as_str).unwrap_or_default();
                    let env = read
                        .env
                        .iter()
                        .flatten()
                        .filter_map(|(k, v)| Some((k.as_str(), v.as_ref_option()?.as_str())))
                        .chain(state_env.iter().map(|(k, v)| (k.as_str(), *v)));

                    match connection.execute(cmd, env).await {
                        Ok(res) => {
                            if res.status == 0 {
                                if res.stderr.len() > 0 {
                                    diags.warning(
                                        "`read` succeeded but stderr was not empty",
                                        res.stderr,
                                        attr_path,
                                    );
                                }

                                outputs.insert(name.clone(), Value::Value(res.stdout));
                            } else {
                                diags.error(
                                    format!("`read` failed with status code: {}", res.status),
                                    res.stderr,
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

        Some((state, planned_private_state))
    }
    async fn update(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
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
        if !state.inputs.is_value() {
            state.inputs = Value::Value(Default::default());
        }

        Some((state, planned_private_state))
    }
    async fn destroy(
        &self,
        diags: &mut Diagnostics,
        state: Self::State,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<()> {
        let connection_default = Default::default();
        let connection = state.connection.as_ref().unwrap_or(&connection_default);

        let state_env: Vec<_> =
            state
                .inputs
                .iter()
                .flatten()
                .filter_map(|(k, v)| Some((format!("INPUT_{k}"), v.as_ref_option()?.as_str())))
                .chain(
                    state.inputs.iter().flatten().filter_map(|(k, v)| {
                        Some((format!("STATE_{k}"), v.as_ref_option()?.as_str()))
                    }),
                )
                .collect();

        let destroy_cmd = state
            .destroy
            .as_ref()
            .and_then(|create| create.cmd.as_ref())
            .map(String::as_str)
            .unwrap_or_default();
        if destroy_cmd != "" {
            let attr_path = AttributePath::new("destroy").index(0).attribute("cmd");
            match connection
                .execute(destroy_cmd, state_env.iter().map(|(k, v)| (k.as_str(), *v)))
                .await
            {
                Ok(res) => {
                    if res.stdout.len() > 0 {
                        diags.warning(
                            "`destroy` stdout was not empty",
                            res.stdout,
                            attr_path.clone(),
                        );
                    }
                    if res.status == 0 {
                        if res.stderr.len() > 0 {
                            diags.warning(
                                "`destroy` succeeded but stderr was not empty",
                                res.stderr,
                                attr_path,
                            );
                        }
                    } else {
                        diags.error(
                            format!("`destroy` failed with status code: {}", res.status),
                            res.stderr,
                            attr_path,
                        );
                    }
                }
                Err(err) => {
                    diags.error("Failed to destroy resource", err, attr_path);
                }
            }
        }
        Some(())
    }
}
