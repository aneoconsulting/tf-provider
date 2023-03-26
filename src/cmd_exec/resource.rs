use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt::Debug;

use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use tf_provider::{
    AttributePath, Diagnostics, Resource, Schema, Value, ValueEmpty, ValueList, ValueMap,
    ValueString,
};

use crate::connection::Connection;
use crate::utils::{WithCmd, WithEnv, WithNormalize, WithSchema};

use super::state::{ResourceState, StateUpdate};
use super::{prepare_envs, with_env};

#[derive(Debug, Default)]
pub struct CmdExecResource<T: Connection> {
    pub(super) connect: T,
}

impl<T: Connection> CmdExecResource<T> {
    pub fn new(connect: T) -> Self {
        Self { connect }
    }
}

#[async_trait]
impl<T> Resource for CmdExecResource<T>
where
    T: Connection,
    T: Debug,
    T: Clone,
{
    type State<'a> = ResourceState<'a, T>;
    type PrivateState<'a> = ValueEmpty;
    type ProviderMetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(ResourceState::<T>::schema())
    }

    async fn validate<'a>(&self, diags: &mut Diagnostics, config: Self::State<'a>) -> Option<()> {
        self.validate(diags, &config, Default::default()).await;

        if diags.errors.len() == 0 {
            Some(())
        } else {
            None
        }
    }

    async fn read<'a>(
        &self,
        diags: &mut Diagnostics,
        state: Self::State<'a>,
        private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let state_env = prepare_envs(&[(&state.inputs, "INPUT_"), (&state.state, "STATE_")]);

        let mut state = state.clone();
        state.normalize(diags);

        // Mark all values unknown to force their read
        state.state = Value::Value(
            state
                .read
                .iter()
                .flatten()
                .map(|(name, _)| (name.clone(), Value::Unknown))
                .collect(),
        );

        state.read(diags, &self.connect, &state_env).await;

        Some((state, private_state))
    }

    async fn plan_create<'a>(
        &self,
        diags: &mut Diagnostics,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = proposed_state.clone();
        state.id = ValueString::Unknown;
        state.state = Value::Unknown;
        state.normalize(diags);

        Some((state, Default::default()))
    }
    async fn plan_update<'a>(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State<'a>,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(
        Self::State<'a>,
        Self::PrivateState<'a>,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        let mut state = proposed_state.clone();
        state.normalize(diags);

        let previous_state_default = Default::default();
        let previous_state = prior_state
            .state
            .as_ref()
            .unwrap_or(&previous_state_default);
        let previous_reads_default = Default::default();
        let previous_reads = prior_state.read.as_ref().unwrap_or(&previous_reads_default);

        match &state.read {
            Value::Value(reads) => {
                // Mark all values unknown to force their read
                state.state = Value::Value(
                    reads
                        .iter()
                        .map(|(name, read)| {
                            (
                                name.clone(),
                                match (previous_reads.get(name), previous_state.get(name)) {
                                    (_, None) => Value::Unknown,
                                    (None, Some(val)) => val.clone(),
                                    (Some(previous_read), Some(val)) => {
                                        if previous_read == read {
                                            val.clone()
                                        } else {
                                            Value::Unknown
                                        }
                                    }
                                },
                            )
                        })
                        .collect(),
                );
            }
            Value::Null => {
                state.read = Value::Value(Default::default());
                state.state = Value::Value(Default::default());
            }
            Value::Unknown => {
                state.state = Value::Unknown;
            }
        }

        let modified = find_modified(&prior_state.inputs, &proposed_state.inputs);
        let mut trigger_replace = Default::default();

        if !modified.is_empty() {
            if let Some((update, _)) = find_update(&proposed_state.update, &modified) {
                if let Value::Value(outputs) = &mut state.state {
                    let reloads_default = Default::default();
                    let reloads = update.reloads.as_ref().unwrap_or(&reloads_default);
                    for name in reloads {
                        if let Some(value) = outputs.get_mut(name.as_str()) {
                            *value = Value::Unknown;
                        }
                    }
                }
            } else {
                trigger_replace = modified
                    .into_iter()
                    .map(|name| {
                        AttributePath::new("inputs").key(name.unwrap_or_default().into_owned())
                    })
                    .collect();
            }
        }

        Some((state, prior_private_state, trigger_replace))
    }

    async fn plan_destroy<'a>(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        _prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        Some(())
    }

    async fn create<'a>(
        &self,
        diags: &mut Diagnostics,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        planned_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = planned_state.clone();
        state.normalize(diags);
        state.id = ValueString::Value(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
        );

        let connection_default = Default::default();
        let connection = planned_state
            .connect
            .as_ref()
            .unwrap_or(&connection_default);

        let state_env = prepare_envs(&[(&planned_state.inputs, "INPUT_")]);

        let create_cmd = state.create.cmd();
        if create_cmd != "" {
            let attr_path = AttributePath::new("create").index(0).attribute("cmd");
            match self
                .connect
                .execute(
                    connection,
                    create_cmd,
                    with_env(&state_env, state.create.env()),
                )
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
                    diags.error("Failed to create resource", err.to_string(), attr_path);
                }
            }
        }

        if diags.errors.len() > 0 {
            return None;
        }

        state.read(diags, &self.connect, &state_env).await;

        Some((state, planned_private_state))
    }
    async fn update<'a>(
        &self,
        diags: &mut Diagnostics,
        prior_state: Self::State<'a>,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        planned_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let connection_default = Default::default();
        let connection = planned_state
            .connect
            .as_ref()
            .unwrap_or(&connection_default);

        let mut state = planned_state.clone();
        state.normalize(diags);
        if !state.id.is_value() {
            state.id = ValueString::Value(
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(30)
                    .map(char::from)
                    .collect(),
            );
        }

        let state_env = prepare_envs(&[
            (&planned_state.inputs, "INPUT_"),
            (&prior_state.inputs, "PREVIOUS_"),
            (&prior_state.state, "STATE_"),
        ]);

        let modified = find_modified(&prior_state.inputs, &planned_state.inputs);
        if let Some((update, attr_path)) = find_update(&planned_state.update, &modified) {
            let attr_path = attr_path.attribute("cmd");
            let update_cmd = update.cmd();
            if update_cmd != "" {
                match self
                    .connect
                    .execute(connection, update_cmd, with_env(&state_env, update.env()))
                    .await
                {
                    Ok(res) => {
                        if res.stdout.len() > 0 {
                            diags.warning(
                                "`update` stdout was not empty",
                                res.stdout,
                                attr_path.clone(),
                            );
                        }
                        if res.status == 0 {
                            if res.stderr.len() > 0 {
                                diags.warning(
                                    "`update` succeeded but stderr was not empty",
                                    res.stderr,
                                    attr_path,
                                );
                            }
                        } else {
                            diags.error(
                                format!("`update` failed with status code: {}", res.status),
                                res.stderr,
                                attr_path,
                            );
                        }
                    }
                    Err(err) => {
                        diags.error("Failed to update resource", err.to_string(), attr_path);
                    }
                }
            } else {
                diags.error_short("`update` cmd should not be null or empty", attr_path);
                return None;
            }
        }

        state.read(diags, &self.connect, &state_env).await;

        Some((state, planned_private_state))
    }
    async fn destroy<'a>(
        &self,
        diags: &mut Diagnostics,
        state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        let connection_default = Default::default();
        let connection = state.connect.as_ref().unwrap_or(&connection_default);

        let state_env = prepare_envs(&[(&state.inputs, "INPUT_"), (&state.state, "STATE_")]);

        let destroy_cmd = state.destroy.cmd();
        if destroy_cmd != "" {
            let attr_path = AttributePath::new("destroy").index(0).attribute("cmd");
            match self
                .connect
                .execute(
                    connection,
                    destroy_cmd,
                    with_env(&state_env, state.destroy.env()),
                )
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
                    diags.error("Failed to destroy resource", err.to_string(), attr_path);
                }
            }
        }
        Some(())
    }
}

fn find_modified<'a>(
    state: &'a ValueMap<'a, ValueString<'a>>,
    plan: &'a ValueMap<'a, ValueString<'a>>,
) -> BTreeSet<ValueString<'a>> {
    match (state, plan) {
        (Value::Value(state), Value::Value(plan)) => {
            let mut modified = BTreeSet::new();

            for (k, x) in state {
                if let Some(y) = plan.get(k) {
                    if x != y {
                        modified.insert(Value::Value(Cow::from(k.as_ref())));
                    }
                } else {
                    modified.insert(Value::Value(Cow::from(k.as_ref())));
                }
            }
            for (k, _) in plan {
                if !state.contains_key(k) {
                    modified.insert(Value::Value(Cow::from(k.as_ref())));
                }
            }

            modified
        }
        (_, Value::Value(plan)) => plan
            .keys()
            .map(|k| Value::Value(Cow::from(k.as_ref())))
            .collect(),
        (Value::Value(state), _) => state
            .keys()
            .map(|k| Value::Value(Cow::from(k.as_ref())))
            .collect(),
        _ => Default::default(),
    }
}

fn find_update<'a>(
    updates: &'a ValueList<Value<StateUpdate<'a>>>,
    modified: &'a BTreeSet<ValueString<'a>>,
) -> Option<(&'a StateUpdate<'a>, AttributePath)> {
    let empty_set = Default::default();
    let updates = updates.as_ref_option()?;

    let mut found: Option<(&'a StateUpdate<'a>, usize)> = None;
    for (i, update) in updates.iter().flatten().enumerate() {
        let triggers = update.triggers.as_ref().unwrap_or(&empty_set);
        if triggers.is_empty() {
            if found.is_none() {
                found = Some((update, i));
            }
        } else if triggers.is_superset(&modified) {
            if let Some((previous, _)) = found {
                let previous_triggers = previous.triggers.as_ref().unwrap_or(&empty_set);
                if previous_triggers.len() > triggers.len() {
                    found = Some((update, i));
                }
            } else {
                found = Some((update, i));
            }
        }
    }
    found.map(|(update, i)| (update, AttributePath::new("update").index(i as i64)))
}
