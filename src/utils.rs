use std::cell::RefCell;

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

pub(crate) trait WithRead: WithCmd {
    fn strip_trailing_newline(&self) -> bool;
}

impl<T: WithRead> WithRead for Value<T> {
    fn strip_trailing_newline(&self) -> bool {
        self.as_ref().map_or(true, WithRead::strip_trailing_newline)
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

pub struct DisplayJoiner<'a, T, I>
where
    T: Iterator<Item = I>,
    I: std::fmt::Display,
{
    iter: RefCell<T>,
    sep: &'a str,
}

pub trait DisplayJoinable {
    type Joiner<'a>;
    fn join_with(self, sep: &str) -> Self::Joiner<'_>;
}

impl<T, I> DisplayJoinable for T
where
    T: Iterator<Item = I>,
    I: std::fmt::Display,
{
    type Joiner<'a> = DisplayJoiner<'a, T, I>;

    fn join_with(self, sep: &str) -> Self::Joiner<'_> {
        DisplayJoiner {
            iter: RefCell::new(self),
            sep,
        }
    }
}

impl<'a, T, I> std::fmt::Display for DisplayJoiner<'a, T, I>
where
    T: Iterator<Item = I>,
    I: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        let mut iter = self.iter.try_borrow_mut().or(Err(std::fmt::Error))?;
        for elt in iter.by_ref() {
            f.write_str(sep)?;
            f.write_fmt(format_args!("{elt}"))?;
            sep = self.sep;
        }
        Ok(())
    }
}
