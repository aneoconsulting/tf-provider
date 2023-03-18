use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use tf_provider::{AttributePath, Diagnostics, Value};

use crate::connection::Connection;
use crate::utils::WithValidate;

#[async_trait]
impl WithValidate for super::state::StateCmd<'_> {
    async fn validate(&self, diags: &mut Diagnostics, mut attr_path: AttributePath) {
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

#[async_trait]
impl WithValidate for super::state::StateUpdate<'_> {
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) {
        _ = self.cmd.validate(diags, attr_path.clone());
        for (name, map) in [("triggers", &self.triggers), ("reloads", &self.reloads)] {
            let attr_path = attr_path.clone().attribute(name);
            match map {
                Value::Value(map) => {
                    for (k, v) in map {
                        let attr_path = attr_path.clone().key(k);
                        match v.as_ref() {
                            Value::Value(v) => {
                                if v.len() == 0 {
                                    diags.error(
                                        format!("Element of `update.{}` is empty", name),
                                        format!("Elements of `update.{}` cannot be empty.", name),
                                        attr_path,
                                    );
                                }
                            }
                            Value::Null => {
                                diags.error(
                                    format!("Element of `update.{}` is null", name),
                                    format!("Elements of `update.{}` cannot be null.", name),
                                    attr_path,
                                );
                            }
                            Value::Unknown => {
                                diags.error(
                                    format!(
                                        "Element of `update.{}` is not known during planning",
                                        name
                                    ),
                                    format!("Elements of `update.{}` cannot be unkown.", name),
                                    attr_path,
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

#[async_trait]
impl<'a, T> WithValidate for super::state::State<'a, T>
where
    T: Connection,
    T: Serialize,
    T: for<'b> Deserialize<'b>,
{
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) {
        if let Value::Value(connection) = &self.connection {
            _ = connection
                .validate(diags, attr_path.clone().attribute("connection"))
                .await;
        }
        if let Value::Value(create) = &self.create {
            _ = create.validate(diags, attr_path.clone().attribute("create"))
        }
        if let Value::Value(destroy) = &self.destroy {
            _ = destroy.validate(diags, attr_path.clone().attribute("destroy"))
        }
        match &self.read {
            Value::Value(read) => {
                let attr_path = attr_path.clone().attribute("read");
                for (name, read) in read {
                    if let Value::Value(read) = read {
                        _ = read.validate(diags, attr_path.clone().key(name));
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
        match &self.update {
            Value::Value(update) => {
                for (i, update) in update.into_iter().enumerate() {
                    if let Value::Value(update) = update {
                        _ = update
                            .validate(diags, attr_path.clone().attribute("update").index(i as i64));
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
