use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Version {
    pub version: u32,
    #[serde(rename = "extensions_implicit_length")]
    pub extensions: BTreeMap<Bytes, Bytes>,
}
