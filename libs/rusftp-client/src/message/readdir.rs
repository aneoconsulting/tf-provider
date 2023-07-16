use serde::{Deserialize, Serialize};

use super::Handle;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ReadDir {
    pub handle: Handle,
}
