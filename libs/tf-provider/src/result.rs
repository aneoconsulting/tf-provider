use crate::diagnostics::{Diagnostic, Diagnostics};

/// A value augmented with diagnostics
/// If there is any error, there is no value
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Result<T> {
    value: Option<T>,
    diags: Diagnostics,
}

impl<T> Default for Result<T> {
    fn default() -> Self {
        Result {
            value: None,
            diags: Default::default(),
        }
    }
}

impl<T> Result<T> {
    /// Get a `std::result::Result<T, Diagnostics>` from this
    pub fn get(self) -> std::result::Result<T, Diagnostics> {
        if self.diags.errors.is_empty() {
            self.value.ok_or(self.diags)
        } else {
            Err(self.diags)
        }
    }
    /// Get a `std::result::Result<&T, &Diagnostics>` from this
    pub fn get_ref(&self) -> std::result::Result<&T, &Diagnostics> {
        if self.diags.errors.is_empty() {
            self.value.as_ref().ok_or(&self.diags)
        } else {
            Err(&self.diags)
        }
    }
    /// Get a `std::result::Result<&mut T, &mut Diagnostics>` from this
    pub fn get_mut(&mut self) -> std::result::Result<&mut T, &mut Diagnostics> {
        if self.diags.errors.is_empty() {
            self.value.as_mut().ok_or(&mut self.diags)
        } else {
            Err(&mut self.diags)
        }
    }

    pub fn as_option(self) -> Option<T> {
        if self.has_errors() {
            None
        } else {
            self.value
        }
    }

    pub fn into_option(self, diags: &mut Diagnostics) -> Option<T> {
        let has_errors = self.has_errors();
        diags.add_diagnostics(self.diags);
        if has_errors {
            None
        } else {
            self.value
        }
    }

    /// Combine many diagnostics into one
    pub fn combine<Cout, Cin>(collection: Cin) -> Result<Cout>
    where
        Cin: IntoIterator<Item = Result<T>>,
        Cout: FromIterator<T>,
    {
        let mut diags = Diagnostics::default();
        let collection: Cout = collection
            .into_iter()
            .filter_map(|result| result.into_option(&mut diags))
            .collect();

        Result::with_diagnostics(collection, diags)
    }

    /// Construct a `Result` from `Diagnostics`
    pub fn from_diagnostics(diags: Diagnostics) -> Self {
        Self {
            value: None,
            diags: diags,
        }
    }
    /// Construct a `Result` from a value and `Diagnostics`
    /// Warning: if there are errors, the value is ignored
    pub fn with_diagnostics(value: T, diags: Diagnostics) -> Self {
        Self {
            value: if diags.errors.is_empty() {
                Some(value)
            } else {
                None
            },
            diags: diags,
        }
    }

    /// Construct a `Result` from an error
    pub fn from_error<E>(err: E) -> Self
    where
        E: ToString,
    {
        Self {
            value: None,
            diags: Diagnostics {
                errors: vec![Diagnostic::root_short(err.to_string())],
                warnings: vec![],
            },
        }
    }

    pub fn get_diagnostics(&self) -> &Diagnostics {
        return &self.diags;
    }

    pub fn has_errors(&self) -> bool {
        !self.diags.errors.is_empty()
    }

    pub fn map<U, F>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> U,
    {
        let value = if self.has_errors() { None } else { self.value };
        if let Some(value) = value {
            Result::with_diagnostics(f(value), self.diags)
        } else {
            Result::from_diagnostics(self.diags)
        }
    }
    pub fn and_then<U, F>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>,
    {
        let value = if self.has_errors() { None } else { self.value };
        let mut result = Result::from_diagnostics(self.diags);
        if let Some(value) = value {
            let r = f(value);
            result.diags.add_diagnostics(r.diags);
            if result.diags.errors.is_empty() {
                result.value = r.value;
            }
        }
        result
    }
}

impl<T> From<T> for Result<T> {
    fn from(value: T) -> Self {
        Self {
            value: Some(value),
            diags: Default::default(),
        }
    }
}

macro_rules! get(
    ($e:expr) => ({
        let result : Result<_> = $e;
        if result.has_errors() {
            return Result::from_diagnostics(result.get_diagnostics().clone())
        }
        match result.as_option() {
        Some(value) => value,
        None => return Result::default(),
    }})
);

/*
macro_rules! combine(
    ($($x:expr),+) => {
        let mut diags = Diagnostics::default();
        let mut has_errors = false;
        let tuple = ($(
            {
                let result : Result<_> = $x;
                diags.add_diagnostics(result.diags);
                if result {

                }
            }
        ),*;
    }
);
*/

pub(crate) use get;
