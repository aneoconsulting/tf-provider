use std::fmt::Display;

/// Represent the path to an attribute
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct AttributePath {
    pub steps: Vec<AttributePathStep>,
}

impl AttributePath {
    /// add name access to the path (ie: `.name`)
    pub fn add_name<T>(&mut self, name: T) -> &mut Self
    where
        T: ToString,
    {
        self.steps.push(AttributePathStep::Name(name.to_string()));
        self
    }
    /// add key access to the path (ie: `["key"]`)
    pub fn add_key<T>(&mut self, key: T) -> &mut Self
    where
        T: ToString,
    {
        self.steps.push(AttributePathStep::Key(key.to_string()));
        self
    }
    /// add index access to the path (ie: `[idx]`)
    pub fn add_index<T>(&mut self, idx: T) -> &mut Self
    where
        T: Into<i64>,
    {
        self.steps.push(AttributePathStep::Index(idx.into()));
        self
    }
    /// Add step to the path
    pub fn add_step<T>(&mut self, step: AttributePathStep) -> &mut Self {
        self.steps.push(step);
        self
    }
    /// Add multiple steps into the path
    pub fn add_steps<T>(&mut self, mut steps: AttributePath) -> &mut Self {
        self.steps.append(&mut steps.steps);
        self
    }
}

impl Display for AttributePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sep = "";
        for step in &self.steps {
            match step {
                AttributePathStep::Name(name) => f.write_fmt(format_args!("{}{}", sep, name))?,
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AttributePathStep {
    Name(String),
    Key(String),
    Index(i64),
}

impl Display for AttributePathStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributePathStep::Name(name) => f.write_str(name),
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
