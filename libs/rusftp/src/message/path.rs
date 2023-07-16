use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Path(pub Bytes);

impl<T: Into<Bytes>> From<T> for Path {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
