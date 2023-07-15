/*
SSH_FXP_VERSION: 2
(VERSION) | u32: version | u32: ext0 name length | u8[ext0 name length]: ext0 name | u32: ext0 value length | u8[ext0 value length]: ext0 value | ...
*/

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Version {
    pub version: u32,
    #[serde(rename = "extensions_implicit_length")]
    pub extensions: BTreeMap<Bytes, Bytes>,
}
