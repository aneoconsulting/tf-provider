use serde::{Deserialize, Serialize};

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Symlink {
    pub link_path: Path,
    pub target_path: Path,
}
