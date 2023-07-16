use std::borrow::Cow;

use bytes::{Buf, BufMut, Bytes};
use serde::{ser::SerializeTuple, Deserialize, Serialize};

use crate::decoder::SftpDecoder;
use crate::encoder::SftpEncoder;
use crate::Error;

mod attrs;
mod close;
mod data;
mod extended;
mod extended_reply;
mod fsetstat;
mod fstat;
mod handle;
mod init;
mod lstat;
mod mkdir;
mod name;
mod open;
mod opendir;
mod path;
mod read;
mod readdir;
mod readlink;
mod realpath;
mod remove;
mod rename;
mod rmdir;
mod setstat;
mod stat;
mod status;
mod symlink;
mod version;
mod write;

pub use attrs::{Attrs, Owner, Permisions, Time};
pub use close::Close;
pub use data::Data;
pub use extended::Extended;
pub use extended_reply::ExtendedReply;
pub use fsetstat::FSetStat;
pub use fstat::FStat;
pub use handle::Handle;
pub use init::Init;
pub use lstat::LStat;
pub use mkdir::MkDir;
pub use name::Name;
pub use open::{pflags, Open};
pub use opendir::OpenDir;
pub use path::Path;
pub use read::Read;
pub use readdir::ReadDir;
pub use readlink::ReadLink;
pub use realpath::RealPath;
pub use remove::Remove;
pub use rename::Rename;
pub use rmdir::RmDir;
pub use setstat::SetStat;
pub use stat::Stat;
pub use status::{Status, StatusCode};
pub use symlink::Symlink;
pub use version::Version;
pub use write::Write;

macro_rules! messages {
    ($($name:ident: $discriminant:expr)*) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum Message {
            $($name($name) = $discriminant,)*
        }

        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum MessageKind {
            $($name = $discriminant,)*
        }

        impl Message {
            pub fn kind(&self) -> MessageKind {
                match self {
                    $(Self::$name(_) => MessageKind::$name,)*
                }
            }
        }

        impl MessageKind {
            pub fn code(&self) -> u8 {
                match self {
                    $(Self::$name => $discriminant,)*
                }
            }
        }

        impl From<Message> for MessageKind {
            fn from(value: Message) -> Self {
                value.kind()
            }
        }

        impl Serialize for Message {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {

                let mut state = serializer.serialize_tuple(2)?;
                state.serialize_element(&self.code())?;

                match self {
                    $(Message::$name(value) => state.serialize_element(value)?,)*
                }
                state.end()
            }
        }

        impl<'de> Deserialize<'de> for Message {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = Message;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a type code and a message content")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let no_value = || ::serde::de::Error::custom("no value found");
                        let code = seq.next_element::<u8>()?.ok_or_else(no_value)?;
                        let content = match code {
                            $($discriminant => seq.next_element::<$name>()?.ok_or_else(no_value)?.into(),)*
                            _ => return Err(::serde::de::Error::custom("invalid message type")),
                        };
                        Ok(content)
                    }
                }

                deserializer.deserialize_tuple(3, Visitor)
            }
        }

        impl<'a> Serialize for MessageWithId<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let id = match self.message.as_ref() {
                    Message::Init(_) | Message::Version(_) => None,
                    _ => Some(self.id),
                };

                let mut state = serializer.serialize_tuple(3)?;
                state.serialize_element(&self.message.code())?;
                state.serialize_element(&id)?;

                match self.message.as_ref() {
                    $(Message::$name(value) => state.serialize_element(&value)?,)*
                }
                state.end()
            }
        }

        impl<'de> Deserialize<'de> for MessageWithId<'de> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = MessageWithId<'de>;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(formatter, "a type code, an id, and a message content")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let no_value = || ::serde::de::Error::custom("no value found");
                        let code = seq.next_element::<u8>()?.ok_or_else(no_value)?;
                        let (id, message) = if code == Init::DISCRIMINANT || code == Version::DISCRIMINANT {
                            seq.next_element()?.ok_or_else(no_value)?;
                            if code == Init::DISCRIMINANT {
                                (0, Message::Init(seq.next_element()?.ok_or_else(no_value)?))
                            } else {
                                (0, Message::Version(seq.next_element()?.ok_or_else(no_value)?))
                            }
                        } else {
                            let id = seq.next_element()?.ok_or_else(no_value)?;
                            let message = match code {
                                $($discriminant => seq.next_element::<$name>()?.ok_or_else(no_value)?.into(),)*
                                _ => return Err(::serde::de::Error::custom("invalid message type")),
                            };
                            (id, message)
                        };
                        Ok(MessageWithId { id, message: Cow::Owned(message) })
                    }
                }

                deserializer.deserialize_tuple(3, Visitor)
            }
        }

        $(
            impl $name {
                #[allow(dead_code)]
                const DISCRIMINANT: u8 = $discriminant;
            }
            impl From<$name> for Message {
                fn from(value: $name) -> Self {
                    Self::$name(value)
                }
            }

            impl TryFrom<Message> for $name {
                type Error = ();
                fn try_from(value: Message) -> Result<Self, Self::Error> {
                    if let Message::$name(value) = value {
                        Ok(value)
                    } else {
                        Err(())
                    }
                }
            }
        )*
    };
}

messages! {
    Init: 1
    Version: 2
    Open: 3
    Close: 4
    Read: 5
    Write: 6
    LStat: 7
    FStat: 8
    SetStat: 9
    FSetStat: 10
    OpenDir: 11
    ReadDir: 12
    Remove: 13
    MkDir: 14
    RmDir: 15
    RealPath: 16
    Stat: 17
    Rename: 18
    ReadLink: 19
    Symlink: 20
    Status: 101
    Handle: 102
    Data: 103
    Name: 104
    Attrs: 105
    Extended: 200
    ExtendedReply: 201
}

impl From<Init> for Version {
    fn from(value: Init) -> Self {
        Self {
            version: value.version,
            extensions: value.extensions,
        }
    }
}
impl From<Version> for Init {
    fn from(value: Version) -> Self {
        Self {
            version: value.version,
            extensions: value.extensions,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MessageWithId<'a> {
    id: u32,
    message: Cow<'a, Message>,
}

impl Message {
    pub fn code(&self) -> u8 {
        self.kind().code()
    }
    pub fn encode(&self, id: u32) -> Result<Bytes, Error> {
        let mut encoder = SftpEncoder::new(Vec::with_capacity(16));

        // Reserve space for frame length
        encoder.buf.put_u32(0);

        MessageWithId {
            id,
            message: Cow::Borrowed(self),
        }
        .serialize(&mut encoder)?;

        // write frame length at the beginning of the frame
        let frame_length = (encoder.buf.len() - std::mem::size_of::<u32>()) as u32;
        let mut buf = encoder.buf.as_mut_slice();
        buf.put_u32(frame_length);

        Ok(encoder.buf.into())
    }

    pub fn decode(mut buf: &[u8]) -> Result<(u32, Self), Error> {
        let frame_length = buf.get_u32() as usize;

        // Limit the read to this very frame
        let mut decoder = SftpDecoder::new(&buf[0..frame_length]);

        let message_with_id = MessageWithId::deserialize(&mut decoder).map_err(Into::into)?;

        Ok((message_with_id.id, message_with_id.message.into_owned()))
    }
}

impl From<Error> for Message {
    fn from(value: Error) -> Self {
        Self::Status(value.into())
    }
}

impl From<std::io::Error> for Message {
    fn from(value: std::io::Error) -> Self {
        Self::Status(value.into())
    }
}

impl From<russh::Error> for Message {
    fn from(value: russh::Error) -> Self {
        Self::Status(value.into())
    }
}

/*
SFTP protocol frame

Frame:
| u32: frame length | u8: type | u8[frame length - 1]: frame body |



SSH_FXP_INIT: 1
(VERSION) | u32: version | u32: ext0 name length | u8[ext0 name length]: ext0 name | u32: ext0 value length | u8[ext0 value length]: ext0 value | ...
-> VERSION

SSH_FXP_VERSION: 2
(VERSION) | u32: version | u32: ext0 name length | u8[ext0 name length]: ext0 name | u32: ext0 value length | u8[ext0 value length]: ext0 value | ...

SSH_FXP_OPEN: 3
| u32: id | u32: filename length | u8[filename length]: filename | u32: pflags | u32: attr flags | {if size in flags} u64: size | {if uid/gid in flags} u32 : uid | {if uid/gid in flags} u32 : gid | {if perm in flags} u32: perms | {if acmodtime in flags} u32: atime | {if acmodtime in flags} u32: mtime |
-> HANDLE | STATUS

    PFLAGS:
        READ: 0x00000001
        WRITE: 0x00000002
        APPEND: 0x00000004
        CREATE: 0x00000008
        TRUNCATE: 0x00000010
        EXCLUDE: 0x00000020

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


SSH_FXP_CLOSE: 4
(HANDLE) | u32: id | u32: handle length | u8[handle length]: handle |
-> STATUS

SSH_FXP_READ: 5
| u32: id | u32: handle length | u8[handle length]: handle | u64: offset | u32: length |
-> DATA | STATUS

SSH_FXP_WRITE: 6
| u32: id | u32: handle length | u8[handle length]: handle | u64: offset | u32: data length | u8[data length]: data |
-> STATUS

SSH_FXP_LSTAT: 7
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> ATTRS

SSH_FXP_FSTAT: 8
(HANDLE) | u32: id | u32: handle length | u8[handle length]: handle |
-> ATTRS

SSH_FXP_SETSTAT: 9
(PATH ATTR) | u32: id | u32: path length | u8[path length]: path | u32: attrs length | u8[attrs length]: attrs |
-> STATUS

SSH_FXP_FSETSTAT: 10
(HANDLE ATTR) | u32: id | u32: handle length | u8[handle length]: path | u32: attrs length | u8[attrs length]: attrs |
-> STATUS

SSH_FXP_OPENDIR: 11
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> HANDLE | STATUS

SSH_FXP_READDIR: 12
(HANDLE) | u32: id | u32: handle length | u8[handle length]: handle |
-> NAME | STATUS

SSH_FXP_REMOVE: 13
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> STATUS

SSH_FXP_MKDIR: 14
(PATH ATTR) | u32: id | u32: path length | u8[path length]: path | u32: attrs length | u8[attrs length]: attrs |
-> STATUS

SSH_FXP_RMDIR: 15
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> STATUS

SSH_FXP_REALPATH: 16
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> NAME | STATUS

SSH_FXP_STAT: 17
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> ATTRS | STATUS

SSH_FXP_RENAME: 18
| u32: id | u32: old path length | u8[old path length]: old path | u32: new path length | u8[new path length]: new path |
-> STATUS

SSH_FXP_READLINK: 19
(PATH) | u32: id | u32: path length | u8[path length]: path |
-> NAME | STATUS

SSH_FXP_SYMLINK: 20
| u32: id | u32: link path length | u8[link path length]: link path | u32: target path length | u8[target path length]: target path |
-> STATUS

SSH_FXP_STATUS: 101
| u32: id | u32: status code | u32: error length | u8[error length]: error | u32: language length | u8[language length]: language |

    Status Code:
        Ok: 0
        Eof: 1
        NoSuchFile: 2
        PermissionDenied: 3
        Failure: 4
        BadMessage: 5
        NoConnection: 6
        ConnectionLost: 7
        OpUnsupported: 8

SSH_FXP_HANDLE: 102
(HANDLE) | u32: id | u32: handle length | u8[handle length]: handle |

SSH_FXP_DATA: 103
(DATA) | u32: id | u32: data length | u8[data length]: data |

SSH_FXP_NAME: 104
| u32: id | u32: file count | u32: file0 name length | u8[file0 name length]: file0 name | u32: file0 long name length | u8[file0 long name length]: file0 long name | u32: file0 attr flags | {if size in file0 flags} u64: file0 size | {if uid/gid in file0 flags} u32 : file0 uid | {if uid/gid in file0 flags} u32 : file0 gid | {if perm in file0 flags} u32: file0 perms | {if acmodtime in file0 flags} u32: file0 atime | {if acmodtime in file0 flags} u32: file0 mtime | ...

    File type: (on permission)
        FIFO: 0x1000
        CHR: 0x2000
        DIR: 0x4000
        BLK: 0x6000
        REG: 0x8000
        LNK: 0xA000
        NAM: 0x5000

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

SSH_FXP_EXTENDED: 200
| u32: id | u32: request length | u8[request length]: request | u8[frame length - 9 - request length]: data |
-> EXTENDED REPLY

SSH_FXP_EXTENDED_REPLY: 201
| u32: id | u8[frame length - 5]: data |
 */
