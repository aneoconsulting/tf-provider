use crate::attribute::AttributePath;

/// List of Errors and Warnings to send back to Terraform
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Diagnostics {
    /// List of errors
    pub errors: Vec<Diagnostic>,
    /// List of warnings
    pub warnings: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Add an error diagnostic
    pub fn add_error(&mut self, diag: Diagnostic) -> &mut Self {
        self.errors.push(diag);
        self
    }
    /// Add a warning diagnostic
    pub fn add_warning(&mut self, diag: Diagnostic) -> &mut Self {
        self.warnings.push(diag);
        self
    }
    /// Add an error
    pub fn error(
        &mut self,
        summary: String,
        detail: String,
        attribute: AttributePath,
    ) -> &mut Self {
        self.add_error(Diagnostic::new(summary, detail, attribute))
    }
    /// Add an error without AttributePath
    pub fn root_error(&mut self, summary: String, detail: String) -> &mut Self {
        self.add_error(Diagnostic::root(summary, detail))
    }
    /// Add an error without details
    pub fn error_short(&mut self, summary: String, attribute: AttributePath) -> &mut Self {
        self.add_error(Diagnostic::short(summary, attribute))
    }
    /// Add an error without AttributePath nor details
    pub fn root_error_short(&mut self, summary: String) -> &mut Self {
        self.add_error(Diagnostic::root_short(summary))
    }

    /// Add a warning
    pub fn warning(
        &mut self,
        summary: String,
        detail: String,
        attribute: AttributePath,
    ) -> &mut Self {
        self.add_warning(Diagnostic::new(summary, detail, attribute))
    }
    /// Add a warning without AttributePath
    pub fn root_warning(&mut self, summary: String, detail: String) -> &mut Self {
        self.add_warning(Diagnostic::root(summary, detail))
    }
    /// Add a warning without details
    pub fn error_warning(&mut self, summary: String, attribute: AttributePath) -> &mut Self {
        self.add_warning(Diagnostic::short(summary, attribute))
    }
    /// Add a warning without AttributePath nor details
    pub fn root_warning_short(&mut self, summary: String) -> &mut Self {
        self.add_warning(Diagnostic::root_short(summary))
    }
    /// Add
    pub fn add_diagnostics(&mut self, mut diags: Diagnostics) -> &mut Self {
        self.errors.append(&mut diags.errors);
        self.warnings.append(&mut diags.warnings);
        self
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostic {
    pub summary: String,
    pub detail: String,
    pub attribute: AttributePath,
}

/// Diagnostic
impl Diagnostic {
    /// Create a diagnostic
    pub fn new(summary: String, detail: String, attribute: AttributePath) -> Self {
        Self {
            summary,
            detail,
            attribute,
        }
    }
    /// Create a diagnostic without AttributePath
    pub fn root(summary: String, detail: String) -> Self {
        Self {
            summary,
            detail,
            attribute: Default::default(),
        }
    }
    /// Create a diagnostic without details
    pub fn short(summary: String, attribute: AttributePath) -> Self {
        Self {
            summary,
            detail: Default::default(),
            attribute,
        }
    }
    /// Create a diagnostic AttributePath nor details
    pub fn root_short(summary: String) -> Self {
        Self {
            summary,
            detail: Default::default(),
            attribute: Default::default(),
        }
    }
}
