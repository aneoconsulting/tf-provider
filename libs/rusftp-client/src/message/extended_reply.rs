/*
SSH_FXP_EXTENDED_REPLY: 201
| u32: id | u8[frame length - 5]: data |
 */

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ExtendedReply {
    #[serde(rename = "data_implicit_length")]
    pub data: Bytes,
}
