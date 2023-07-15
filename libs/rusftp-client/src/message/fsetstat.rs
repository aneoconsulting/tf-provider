use serde::{Deserialize, Serialize};

use super::{Attrs, Handle};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct FSetStat {
    pub handle: Handle,
    pub attrs: Attrs,
}
