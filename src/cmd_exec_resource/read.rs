use std::borrow::Cow;

use async_trait::async_trait;

use futures::{stream, StreamExt};
use tf_provider::{AttributePath, Diagnostics, Value};

use crate::connection::Connection;

use super::{state::State, with_env};

#[async_trait]
pub(super) trait WithRead {
    type Connect: Connection;
    async fn read(
        &mut self,
        diags: &mut Diagnostics,
        connect: &Self::Connect,
        env: &Vec<(Cow<str>, Cow<str>)>,
    ) -> Option<()>;
}

#[async_trait]
impl<'a, T> WithRead for State<'a, T>
where
    T: Connection,
{
    type Connect = T;
    async fn read(
        &mut self,
        diags: &mut Diagnostics,
        connect: &Self::Connect,
        env: &Vec<(Cow<str>, Cow<str>)>,
    ) -> Option<()> {
        let state = self.state.as_mut_option()?;

        let connection_default = Default::default();
        let connection = self.connect.as_ref().unwrap_or(&connection_default);

        let reads_default = Default::default();
        let reads = self.read.as_ref().unwrap_or(&reads_default);

        let mut read_tasks = Vec::new();

        for (name, value) in state.iter_mut() {
            if !value.is_unknown() {
                continue;
            }
            if let Some(Value::Value(read)) = reads.get(name) {
                let cmd = read.cmd.as_str();

                read_tasks.push(async move {
                    let result = connect
                        .execute(connection, cmd, with_env(&env, &read.env))
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
            .buffer_unordered(6)
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
}
