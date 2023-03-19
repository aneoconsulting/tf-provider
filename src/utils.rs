use async_trait::async_trait;

use tf_provider::{AttributePath, Diagnostics, Schema, Value};

pub(crate) trait WithSchema {
    fn schema() -> Schema;
}

#[async_trait]
pub(crate) trait WithValidate {
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath);
}

pub(crate) trait WithNormalize {
    fn normalize(&mut self, diags: &mut Diagnostics);
}

pub(crate) trait WithCmd {
    fn cmd(&self) -> &str;
}

impl<T: WithCmd> WithCmd for Value<T> {
    fn cmd(&self) -> &str {
        self.as_ref().map_or("", WithCmd::cmd)
    }
}

pub(crate) trait WithEnv {
    type Env;
    fn env(&self) -> &Self::Env;
}

impl<T, E> WithEnv for Value<T>
where
    T: WithEnv<Env = Value<E>>,
{
    type Env = T::Env;
    fn env(&self) -> &Self::Env {
        self.as_ref().map_or(&Value::Null, WithEnv::env)
    }
}
