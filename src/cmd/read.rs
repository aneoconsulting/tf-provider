use std::borrow::Cow;

use futures::{stream, StreamExt};
use tf_provider::{AttributePath, Diagnostics, Value, ValueMap, ValueNumber, ValueString};

use crate::{
    connection::Connection,
    utils::{WithEnv, WithRead},
};

use super::{
    state::{DataSourceState, ResourceState},
    with_env,
};

impl<'a, T: Connection> ResourceState<'a, T> {
    pub async fn read<'b>(
        &mut self,
        diags: &mut Diagnostics,
        connect: &T,
        env: &[(Cow<'b, str>, Cow<'b, str>)],
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
        env: &[(Cow<'b, str>, Cow<'b, str>)],
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

async fn read_all<'a, 'b, C, R>(
    diags: &mut Diagnostics,
    connect: &C,
    connect_config: &Value<C::Config<'a>>,
    reads: &ValueMap<'a, Value<R>>,
    outputs: &mut ValueMap<'a, ValueString<'a>>,
    env: &[(Cow<'b, str>, Cow<'b, str>)],
    concurrency: ValueNumber,
) -> Option<()>
where
    C: Connection,
    R: WithRead + WithEnv<Env = ValueMap<'a, ValueString<'a>>>,
{
    let outputs = outputs.as_mut_option()?;

    let connection_default = Default::default();
    let connect_config = connect_config.as_ref().unwrap_or(&connection_default);

    let reads_default = Default::default();
    let reads = reads.as_ref().unwrap_or(&reads_default);

    let concurrency = concurrency.unwrap_or(4) as usize;

    let mut read_tasks = Vec::new();

    for (name, value) in outputs.iter_mut() {
        if !value.is_unknown() {
            continue;
        }
        if let Some(Value::Value(read)) = reads.get(name) {
            let cmd = read.cmd();
            let dir = read.dir();
            let strip_trailing_newline = read.strip_trailing_newline();

            read_tasks.push(async move {
                let result = connect
                    .execute(connect_config, cmd, dir, with_env(env, read.env()))
                    .await;
                (name, value, strip_trailing_newline, result)
            });
        } else {
            diags.error(
                    "Unknown output has no `read` block associated",
                    format!("The output `state.{name}` is unknown, and there is no known `read[\"{name}\"]` block to give it a value."),
                    AttributePath::new("state").key(name.to_string())
                );
        }
    }

    for (name, value, strip_trailing_newline, result) in stream::iter(read_tasks.into_iter())
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
                    if !res.stderr.is_empty() {
                        diags.warning(
                            "`read` succeeded but stderr was not empty",
                            res.stderr,
                            attr_path,
                        );
                    }
                    let mut stdout: Cow<'_, _> = res.stdout.into();

                    if strip_trailing_newline && stdout.as_bytes()[stdout.len() - 1] == b'\n' {
                        stdout = match stdout {
                            Cow::Borrowed(s) => Cow::Borrowed(&s[0..s.len() - 1]),
                            Cow::Owned(mut s) => {
                                s.pop();
                                Cow::Owned(s)
                            }
                        }
                    }

                    *value = Value::Value(stdout);
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
