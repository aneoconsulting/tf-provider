/*
SSH_FXP_ATTRS: 105
| u32: id | u32: attr flags | {if size in flags} u64: size | {if uid/gid in flags} u32 : uid | {if uid/gid in flags} u32 : gid | {if perm in flags} u32: perms | {if acmodtime in flags} u32: atime | {if acmodtime in flags} u32: mtime |

    Flags:
        SIZE: 0x00000001
        UIDGID: 0x00000002
        PERMISSIONS: 0x00000004
        ACMODTIME: 0x00000008

    File type: (on permission)
        FIFO: 0x1000
        CHR: 0x2000
        DIR: 0x4000
        BLK: 0x6000
        REG: 0x8000
        LNK: 0xA000
        NAM: 0x5000
 */

use serde::{ser::SerializeTuple, Deserialize, Serialize};
use std::default::Default;

use super::FileAttrs;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum FilePermisions {
    // Permissions for others
    OX = 0x0001,
    OW = 0x0002,
    OR = 0x0004,
    // Permissions for group
    GX = 0x0008,
    GW = 0x0010,
    GR = 0x0020,
    // Permissions for user
    UX = 0x0040,
    UW = 0x0080,
    UR = 0x0100,
    // Special permissions
    SX = 0x0200,
    SW = 0x0400,
    SR = 0x0800,
    // File type
    FIFO = 0x1000,
    CHR = 0x2000,
    DIR = 0x4000,
    BLK = 0x6000,
    REG = 0x8000,
    LNK = 0xA000,
    NAM = 0x5000,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct FileOwner {
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct FileTime {
    pub atime: u32,
    pub mtime: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
enum AttrFlags {
    Size = 0x00000001,
    Owner = 0x00000002,
    Perms = 0x00000004,
    Time = 0x00000008,
}

impl Serialize for FileAttrs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut attr_flags = 0u32;

        if self.size.is_some() {
            attr_flags |= AttrFlags::Size as u32;
        }
        if self.owner.is_some() {
            attr_flags |= AttrFlags::Owner as u32;
        }
        if self.perms.is_some() {
            attr_flags |= AttrFlags::Perms as u32;
        }
        if self.time.is_some() {
            attr_flags |= AttrFlags::Time as u32;
        }

        let mut state = serializer.serialize_tuple(5)?;

        state.serialize_element(&attr_flags)?;
        state.serialize_element(&self.size)?;
        state.serialize_element(&self.owner)?;
        state.serialize_element(&self.perms)?;
        state.serialize_element(&self.time)?;

        state.end()
    }
}

macro_rules! next {
    ($seq:expr, $field:expr) => {
        $seq.next_element()?
            .ok_or(serde::de::Error::missing_field($field))?
    };
}

impl<'de> Deserialize<'de> for FileAttrs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FileAttrs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a flag, a size, an owner pair, a perm flag, and a time pair"
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut attrs = FileAttrs::default();
                let attr_flags: u32 = next!(seq, "attr_flags");

                if (attr_flags & AttrFlags::Size as u32) != 0 {
                    attrs.size = Some(next!(seq, "attr_size"));
                } else {
                    next!(seq, "attr_size");
                }
                if (attr_flags & AttrFlags::Owner as u32) != 0 {
                    attrs.owner = Some(next!(seq, "attr_owner"));
                } else {
                    next!(seq, "attr_owner");
                }
                if (attr_flags & AttrFlags::Perms as u32) != 0 {
                    attrs.perms = Some(next!(seq, "attr_perms"));
                } else {
                    next!(seq, "attr_perms");
                }
                if (attr_flags & AttrFlags::Time as u32) != 0 {
                    attrs.time = Some(next!(seq, "attr_time"));
                } else {
                    next!(seq, "attr_time");
                }

                Ok(attrs)
            }
        }

        deserializer.deserialize_tuple(5, Visitor)
    }
}
