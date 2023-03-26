use std::borrow::Cow;

use futures::{stream, StreamExt};
use tf_provider::{AttributePath, Diagnostics, Value, ValueMap, ValueNumber, ValueString};

use crate::connection::Connection;

use super::{
    state::{DataSourceState, ResourceState, StateRead},
    with_env,
};

impl<'a, T: Connection> ResourceState<'a, T> {
    pub async fn read<'b>(
        &mut self,
        diags: &mut Diagnostics,
        connect: &T,
        env: &Vec<(Cow<'b, str>, Cow<'b, str>)>,
    ) -> Option<()> {
        read_all(
            diags,
            connect,
            &self.connect,
            &self.read,
            &mut self.state,
            env,
            self.command_concurrency,
        )
        .await
    }
}

impl<'a, T: Connection> DataSourceState<'a, T> {
    pub async fn read<'b>(
        &mut self,
        diags: &mut Diagnostics,
        connect: &T,
        env: &Vec<(Cow<'b, str>, Cow<'b, str>)>,
    ) -> Option<()> {
        read_all(
            diags,
            connect,
            &self.connect,
            &self.read,
            &mut self.outputs,
            env,
            self.command_concurrency,
        )
        .await
    }
}

pub async fn read_all<'a, 'b, T: Connection>(
    diags: &mut Diagnostics,
    connect: &T,
    connect_config: &Value<T::Config<'a>>,
    reads: &ValueMap<'a, Value<StateRead<'a>>>,
    outputs: &mut ValueMap<'a, ValueString<'a>>,
    env: &Vec<(Cow<'b, str>, Cow<'b, str>)>,
    concurrency: ValueNumber,
) -> Option<()> {
    let outputs = outputs.as_mut_option()?;

    let connection_default = Default::default();
    let connect_config = connect_config.as_ref().unwrap_or(&connection_default);

    let reads_default = Default::default();
    let reads = reads.as_ref().unwrap_or(&reads_default);

    let concurrency = concurrency.clone().unwrap_or(4) as usize;

    let mut read_tasks = Vec::new();

    for (name, value) in outputs.iter_mut() {
        if !value.is_unknown() {
            continue;
        }
        if let Some(Value::Value(read)) = reads.get(name) {
            let cmd = read.cmd.as_str();

            read_tasks.push(async move {
                let result = connect
                    .execute(connect_config, cmd, with_env(&env, &read.env))
                    .await;
                (name, (value), result)
            });
        } else {
            diags.error(
                    "Unknown output has no `read` block associated",
                    format!("The output `state.{name}` is unknown, and there is no known `read[\"{name}\"]` block to give it a value."),
                    AttributePath::new("state").key(name.to_string())
                );
        }
    }

    for (name, value, result) in stream::iter(read_tasks.into_iter())
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await
    {
        let attr_path = AttributePath::new("read")
            .key(name.to_string())
            .attribute("cmd");
        match result {
            Ok(res) => {
                if res.status == 0 {
                    if res.stderr.len() > 0 {
                        diags.warning(
                            "`read` succeeded but stderr was not empty",
                            res.stderr,
                            attr_path,
                        );
                    }

                    *value = Value::Value(res.stdout.into());
                } else {
                    diags.warning(
                        format!("`read` failed with status code: {}", res.status),
                        res.stderr,
                        attr_path,
                    );
                }
            }
            Err(err) => {
                diags.warning("Failed to read resource state", err.to_string(), attr_path);
            }
        }
    }

    Some(())
}
