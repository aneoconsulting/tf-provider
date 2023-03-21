use std::{borrow::Cow, fmt::Display};

use crate::tfplugin6;

/// Represent the path to an attribute
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct AttributePath {
    pub steps: Vec<AttributePathStep>,
}

impl AttributePath {
    /// Create a new attribute path with the `root` attribute
    pub fn new<T: Into<Cow<'static, str>>>(root: T) -> Self {
        Self {
            steps: vec![AttributePathStep::Attribute(root.into())],
        }
    }
    /// Create a new attribute path where the attribute `.name` has been appended
    pub fn attribute<T: Into<Cow<'static, str>>>(mut self, name: T) -> Self {
        self.add_attribute(name);
        self
    }
    /// Create a new attribute path where the access `["key"]` has been appended
    pub fn key<T: Into<Cow<'static, str>>>(mut self, key: T) -> Self {
        self.add_key(key);
        self
    }
    /// Create a new attribute path where the access `[idx]` has been appended
    pub fn index<T: Into<i64>>(mut self, idx: T) -> Self {
        self.add_index(idx);
        self
    }

    /// add name access to the path (ie: `.name`)
    pub fn add_attribute<T: Into<Cow<'static, str>>>(&mut self, name: T) -> &mut Self {
        self.steps.push(AttributePathStep::Attribute(name.into()));
        self
    }
    /// add key access to the path (ie: `["key"]`)
    pub fn add_key<T: Into<Cow<'static, str>>>(&mut self, key: T) -> &mut Self {
        self.steps.push(AttributePathStep::Key(key.into()));
        self
    }
    /// add index access to the path (ie: `[idx]`)
    pub fn add_index<T: Into<i64>>(&mut self, idx: T) -> &mut Self {
        self.steps.push(AttributePathStep::Index(idx.into()));
        self
    }
    /// Add step to the path
    pub fn add_step(&mut self, step: AttributePathStep) -> &mut Self {
        self.steps.push(step);
        self
    }
    /// Add multiple steps into the path
    pub fn add_steps(&mut self, mut steps: AttributePath) -> &mut Self {
        self.steps.append(&mut steps.steps);
        self
    }
}

impl Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        for step in &self.steps {
            match step {
                AttributePathStep::Attribute(name) => {
                    f.write_fmt(format_args!("{}{}", sep, name))?
                }
                AttributePathStep::Key(key) => f.write_fmt(format_args!("[{:?}]", key))?,
                AttributePathStep::Index(idx) => f.write_fmt(format_args!("[{}]", idx))?,
            }
            sep = ".";
        }
        Ok(())
    }
}

impl std::ops::AddAssign<AttributePathStep> for AttributePath {
    fn add_assign(&mut self, rhs: AttributePathStep) {
        self.steps.push(rhs);
    }
}

impl std::ops::Add<AttributePathStep> for AttributePath {
    type Output = Self;
    fn add(mut self, rhs: AttributePathStep) -> Self::Output {
        self += rhs;
        self
    }
}

impl From<AttributePathStep> for AttributePath {
    fn from(value: AttributePathStep) -> Self {
        Self { steps: vec![value] }
    }
}

impl From<AttributePath> for tfplugin6::AttributePath {
    fn from(value: AttributePath) -> Self {
        Self {
            steps: value.steps.into_iter().map(|step| step.into()).collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AttributePathStep {
    Attribute(Cow<'static, str>),
    Key(Cow<'static, str>),
    Index(i64),
}

impl Display for AttributePathStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributePathStep::Attribute(name) => f.write_str(name.as_ref()),
            AttributePathStep::Key(key) => f.write_fmt(format_args!("[{:?}]", key)),
            AttributePathStep::Index(idx) => f.write_fmt(format_args!("[{}]", idx)),
        }
    }
}

impl std::ops::Add<AttributePathStep> for AttributePathStep {
    type Output = AttributePath;
    fn add(self, rhs: AttributePathStep) -> Self::Output {
        AttributePath {
            steps: vec![self, rhs],
        }
    }
}

impl From<AttributePathStep> for tfplugin6::attribute_path::Step {
    fn from(value: AttributePathStep) -> Self {
        use tfplugin6::attribute_path::step::Selector;
        Self {
            selector: Some(match value {
                AttributePathStep::Attribute(name) => Selector::AttributeName(name.into_owned()),
                AttributePathStep::Key(key) => Selector::ElementKeyString(key.into_owned()),
                AttributePathStep::Index(idx) => Selector::ElementKeyInt(idx),
            }),
        }
    }
}
