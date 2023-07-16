use serde::{Deserialize, Serialize};

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Rename {
    pub old_path: Path,
    pub new_path: Path,
}
