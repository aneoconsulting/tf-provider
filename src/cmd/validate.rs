use std::collections::{BTreeMap, BTreeSet};

use tf_provider::{AttributePath, Diagnostics, Value};

use crate::connection::Connection;
use crate::utils::DisplayJoinable;

use super::{
    state::{DataSourceState, ResourceState, StateUpdate},
    GenericCmdDataSource,
};

impl super::state::StateCmd<'_> {
    async fn validate(&self, diags: &mut Diagnostics, mut attr_path: AttributePath) {
        if self.dir.is_unknown() {
            diags.warning("`dir` is not known during planning", "It is recommended that the command does not depend on any resource, and use variables instead.", attr_path.clone().attribute("dir"));
        }
        attr_path.add_attribute("cmd");
        match self.cmd.as_ref() {
            Value::Value(cmd) => {
                if cmd.len() == 0 {
                    diags.error_short("`cmd` cannot be empty", attr_path);
                }
            }
            Value::Null => {
                diags.error_short("`cmd` cannot be null", attr_path);
            }
            Value::Unknown => {
                diags.warning("`cmd` is not known during planning", "It is recommended that the command does not depend on any resource, and use variables instead.", attr_path);
            }
        }
    }
}

impl super::state::StateUpdate<'_> {
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) {
        self.cmd.validate(diags, attr_path.clone()).await;
        for (name, set) in [("triggers", &self.triggers), ("reloads", &self.reloads)] {
            let attr_path = attr_path.clone().attribute(name);
            match set {
                Value::Value(set) => {
                    for k in set {
                        match k {
                            Value::Value(k) => {
                                if k.len() == 0 {
                                    diags.error(
                                        format!("Element of `update.{}` is empty", name),
                                        format!("Elements of `update.{}` cannot be empty.", name),
                                        attr_path.clone().key(k.to_string()),
                                    );
                                }
                            }
                            Value::Null => {
                                diags.error(
                                    format!("Element of `update.{}` is null", name),
                                    format!("Elements of `update.{}` cannot be null.", name),
                                    attr_path.clone(),
                                );
                            }
                            Value::Unknown => {
                                diags.error(
                                    format!(
                                        "Element of `update.{}` is not known during planning",
                                        name
                                    ),
                                    format!("Elements of `update.{}` cannot be unkown.", name),
                                    attr_path.clone(),
                                );
                            }
                        }
                    }
                }
                Value::Null => (),
                Value::Unknown => {
                    diags.error(
                        format!("`update.{}` is not known during planning", name),
                        format!("`update.{}` cannot be unkown.", name),
                        attr_path,
                    );
                }
            }
        }
    }
}

impl<T: Connection> super::resource::GenericCmdResource<T> {
    pub(super) async fn validate<'a>(
        &self,
        diags: &mut Diagnostics,
        config: &ResourceState<'a, T>,
        attr_path: AttributePath,
    ) {
        if let Value::Value(concurrency) = config.command_concurrency {
            if concurrency <= 0 {
                diags.error(
                    "Invalid `command_concurrency`",
                    format!("Command concurrency must be positive, but was {concurrency}."),
                    attr_path.clone().attribute("command_concurrency"),
                );
            }
        }
        if let Value::Value(connection) = &config.connect {
            _ = self
                .connect
                .validate(
                    diags,
                    attr_path.clone().attribute("connection").index(0),
                    connection,
                )
                .await;
        }
        if let Value::Value(create) = &config.create {
            create
                .validate(diags, attr_path.clone().attribute("create").index(0))
                .await;
        }
        if let Value::Value(destroy) = &config.destroy {
            destroy
                .validate(diags, attr_path.clone().attribute("destroy").index(0))
                .await;
        }
        match &config.read {
            Value::Value(read) => {
                let attr_path = attr_path.clone().attribute("read");
                for (name, read) in read {
                    if let Value::Value(read) = read {
                        read.cmd
                            .validate(diags, attr_path.clone().key(name.to_string()))
                            .await;
                    }
                }
            }
            Value::Null => (),
            Value::Unknown => {
                diags.error(
                    "`read` blocks are unknown during plan.",
                    "All `read` blocks must be known during plan.",
                    attr_path.clone().attribute("read"),
                );
            }
        }

        let reads_default = Default::default();
        let reads = config.read.as_ref().unwrap_or(&reads_default);
        match &config.update {
            Value::Value(updates) => {
                ensure_unambiguous_updates(diags, updates.as_slice());
                for (i, update) in updates.iter().enumerate() {
                    if let Value::Value(update) = update {
                        let attr_path = attr_path.clone().attribute("update").index(i as i64);
                        update.validate(diags, attr_path.clone()).await;

                        if let Value::Value(reloads) = &update.reloads {
                            for name in reloads {
                                if !reads.contains_key(name.as_str()) {
                                    diags.error(
                                        "`update.reloads` is invalid",
                                        format!("The `update` block requires to reload `{name}`, but there is no `read` block with this name."),
                                        attr_path.clone().attribute("reloads").key(name.to_string())
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Value::Null => (),
            Value::Unknown => {
                diags.error(
                    "`update` blocks are unknown during plan.",
                    "All `update` blocks must be known during plan.",
                    attr_path.clone().attribute("update"),
                );
            }
        }
    }
}

impl<T: Connection> GenericCmdDataSource<T> {
    pub(super) async fn validate<'a>(
        &self,
        diags: &mut Diagnostics,
        config: &DataSourceState<'a, T>,
        attr_path: AttributePath,
    ) {
        if let Value::Value(concurrency) = config.command_concurrency {
            if concurrency <= 0 {
                diags.error(
                    "Invalid `command_concurrency`",
                    format!("Command concurrency must be positive, but was {concurrency}."),
                    attr_path.clone().attribute("command_concurrency"),
                );
            }
        }
        if let Value::Value(connection) = &config.connect {
            _ = self
                .connect
                .validate(
                    diags,
                    attr_path.clone().attribute("connection").index(0),
                    connection,
                )
                .await;
        }

        match &config.read {
            Value::Value(read) => {
                let attr_path = AttributePath::new("read");
                for (name, read) in read {
                    if let Value::Value(read) = read {
                        read.cmd
                            .validate(diags, attr_path.clone().key(name.to_string()))
                            .await;
                    }
                }
            }
            Value::Null => (),
            Value::Unknown => {
                diags.error(
                    "`read` blocks are unknown during plan.",
                    "All `read` blocks must be known during plan.",
                    AttributePath::new("read"),
                );
            }
        }
    }
}

fn ensure_unambiguous_updates<'a>(diags: &mut Diagnostics, updates: &'a [Value<StateUpdate<'a>>]) {
    let default_triggers = Default::default();
    let mut seen = BTreeSet::new();
    let mut conflicts = BTreeMap::new();

    for (i, update0) in updates.iter().flatten().enumerate() {
        let attr_path = AttributePath::new("update")
            .index(i as i64)
            .attribute("triggers");
        let triggers0 = update0.triggers.as_ref().unwrap_or(&default_triggers);
        if !seen.insert(triggers0) {
            diags.error(
                "Duplicate `update`",
                format!(
                    "There is multiple `update` blocks that are triggered by [{}].",
                    triggers0.iter().join_with(","),
                ),
                attr_path.clone(),
            );
        }
        for update1 in updates.iter().flatten().skip(i + 1) {
            let triggers1 = update1.triggers.as_ref().unwrap_or(&default_triggers);
            if !triggers0.is_subset(triggers1) && !triggers1.is_subset(triggers0) {
                let intersection: BTreeSet<_> = triggers0
                    .intersection(triggers1)
                    .map(Clone::clone)
                    .collect();
                if !intersection.is_empty() {
                    let conflicting_triggers: &mut BTreeSet<_> =
                        conflicts.entry(intersection).or_insert(Default::default());
                    conflicting_triggers.insert(triggers0);
                    conflicting_triggers.insert(triggers1);
                }
            }
        }
    }

    for (conflict, conflicting_triggers) in conflicts {
        if !seen.contains(&conflict) {
            diags.root_error(
                "`update` ambiguity",
                format!(
                    "The update of [{}] is ambiguous and would trigger the following `update` blocks: [{}].\nYou can disambiguate the update by adding a new `update` block with `triggers = [{}]`",
                    conflict.iter().join_with(","),
                    conflicting_triggers.iter().map(|t| t.iter().join_with(",")).join_with("], ["),
                    conflict.iter().join_with(","),
                ),
            );
        }
    }
}
