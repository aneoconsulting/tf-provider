use serde::{Deserialize, Serialize};

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Stat {
    pub path: Path,
}
