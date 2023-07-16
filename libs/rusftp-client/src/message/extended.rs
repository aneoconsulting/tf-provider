use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Extended {
    pub request: Bytes,
    #[serde(rename = "data_implicit_length")]
    pub data: Bytes,
}
