use serde::{Deserialize, Serialize};

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct RmDir {
    pub path: Path,
}
