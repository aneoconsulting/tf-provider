use crate::{diagnostics::Diagnostics, tfplugin6};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DynamicValue {
    MessagePack(Vec<u8>),
    Json(Vec<u8>),
}

impl DynamicValue {
    pub fn deserialize<T>(&self, diags: &mut Diagnostics) -> Option<T>
    where
        T: DeserializeOwned,
    {
        match self {
            Self::MessagePack(mp) => {
                let slice = if mp.is_empty() {
                    &[0xc0_u8] // null MessagePack
                } else {
                    mp.as_slice()
                };
                match rmp_serde::from_slice::<T>(slice) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        diags.root_error_short(err.to_string());
                        None
                    }
                }
            }
            Self::Json(json) => {
                let slice = if json.is_empty() {
                    "null".as_bytes()
                } else {
                    json.as_slice()
                };
                match serde_json::from_slice::<T>(slice) {
                    Ok(value) => Some(value),
                    Err(err) => {
                        diags.root_error_short(err);
                        None
                    }
                }
            }
        }
    }

    pub fn serialize_vec<T>(diags: &mut Diagnostics, value: &T) -> Option<Vec<u8>>
    where
        T: Serialize,
    {
        match rmp_serde::to_vec_named(value) {
            Ok(value) => Some(value),
            Err(err) => {
                diags.root_error_short(err);
                None
            }
        }
    }
    pub fn serialize<T>(diags: &mut Diagnostics, value: &T) -> Option<DynamicValue>
    where
        T: Serialize,
    {
        Some(Self::MessagePack(Self::serialize_vec(diags, value)?))
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
