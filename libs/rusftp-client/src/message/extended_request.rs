/*
SSH_FXP_EXTENDED: 200
| u32: id | u32: request length | u8[request length]: request | u8[frame length - 9 - request length]: data |
 */

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct ExtendedRequest {
    pub request: Bytes,
    #[serde(rename = "data_implicit_length")]
    pub data: Bytes,
}
