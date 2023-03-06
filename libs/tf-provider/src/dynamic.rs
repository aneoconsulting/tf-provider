use crate::{result::Result, tfplugin6};
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

impl From<tfplugin6::DynamicValue> for DynamicValue {
    fn from(value: tfplugin6::DynamicValue) -> Self {
        if value.msgpack.is_empty() {
            DynamicValue::Json(value.json)
        } else {
            DynamicValue::MessagePack(value.msgpack)
        }
    }
}

impl From<DynamicValue> for tfplugin6::DynamicValue {
    fn from(value: DynamicValue) -> Self {
        match value {
            DynamicValue::MessagePack(msgpack) => Self {
                msgpack,
                json: Default::default(),
            },
            DynamicValue::Json(json) => Self {
                msgpack: Default::default(),
                json,
            },
        }
    }
}
