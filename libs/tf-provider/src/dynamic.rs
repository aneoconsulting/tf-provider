use crate::result::Result;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DynamicValue {
    MessagePack(Vec<u8>),
    Json(Vec<u8>),
}

impl DynamicValue {
    pub fn deserialize<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::MessagePack(mp) => match rmp_serde::from_slice::<T>(mp.as_slice()) {
                Ok(value) => Result::from(value),
                Err(err) => Result::from_error(err),
            },
            Self::Json(json) => match serde_json::from_slice::<T>(json.as_slice()) {
                Ok(value) => Result::from(value),
                Err(err) => Result::from_error(err),
            },
        }
    }

    pub fn serialize<T>(value: &T) -> Result<DynamicValue>
    where
        T: Serialize,
    {
        match rmp_serde::to_vec(value) {
            Ok(value) => Result::from(Self::MessagePack(value)),
            Err(err) => Result::from_error(err),
        }
    }
}
