use serde::{Deserialize, Serialize};

use super::{Attrs, Path};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Name {
    pub filename: Path,
    pub long_name: Path,
    pub attrs: Attrs,
}
