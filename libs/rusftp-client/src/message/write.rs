use serde::{Deserialize, Serialize};

use super::{Data, Handle};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Write {
    pub handle: Handle,
    pub offset: u64,
    pub data: Data,
}
