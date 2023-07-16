use serde::{Deserialize, Serialize};

use super::Handle;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Read {
    pub handle: Handle,
    pub offset: u64,
    pub length: u32,
}
