use serde::{Deserialize, Serialize};

use super::{Attrs, Path};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Open {
    pub filename: Path,
    pub pflags: u32,
    pub attrs: Attrs,
}

pub mod pflags {
    pub const READ: u32 = 0x00000001;
    pub const WRITE: u32 = 0x00000002;
    pub const APPEND: u32 = 0x00000004;
    pub const CREATE: u32 = 0x00000008;
    pub const TRUNCATE: u32 = 0x00000010;
    pub const EXCLUDE: u32 = 0x00000020;
}
