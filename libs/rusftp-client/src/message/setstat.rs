use serde::{Deserialize, Serialize};

use super::{Attrs, Path};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct SetStat {
    pub path: Path,
    pub attrs: Attrs,
}
