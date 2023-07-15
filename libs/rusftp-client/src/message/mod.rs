use std::default::Default;

use bytes::{Buf, BufMut, Bytes};
use rusftp_macro::TaggedEnum;
use serde::{Deserialize, Serialize};

use crate::Error;

mod attrs;
mod extended_reply;
mod extended_request;
mod status;
mod version;

pub mod decoder;
pub mod encoder;

pub use attrs::{FileOwner, FilePermisions, FileTime};
use decoder::SftpDecoder;
use encoder::SftpEncoder;
pub use extended_reply::ExtendedReply;
pub use extended_request::ExtendedRequest;
pub use status::{Status, StatusCode};
pub use version::Version;

pub type Handle = MessageHandle;
pub type FileAttrs = MessageAttrs;
pub type Attrs = MessageAttrs;
pub type Data = MessageData;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Path(pub Bytes);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum PFlags {
    READ = 0x00000001,
    WRITE = 0x00000002,
    APPEND = 0x00000004,
    CREATE = 0x00000008,
    TRUNCATE = 0x00000010,
    EXCLUDE = 0x00000020,
}

#[derive(Debug, PartialEq, Eq, Clone, TaggedEnum)]
#[tagged_enum_derives(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
#[non_exhaustive]
pub enum Message {
    Init(Version) = 1,
    Version(Version) = 2,
    Open {
        filename: Path,
        pflags: u32,
        attrs: Attrs,
    } = 3,
    Close {
        handle: Handle,
    } = 4,
    Read {
        handle: Handle,
        offset: u64,
        length: u32,
    } = 5,
    Write {
        handle: Handle,
        offset: u64,
        data: Data,
    } = 6,
    LStat {
        path: Path,
    } = 7,
    FStat {
        handle: Handle,
    } = 8,
    SetStat {
        path: Path,
        attrs: Attrs,
    } = 9,
    FSetStat {
        handle: Handle,
        attrs: Attrs,
    } = 10,
    OpenDir {
        path: Path,
    } = 11,
    ReadDir {
        handle: Handle,
    } = 12,
    Remove {
        path: Path,
    } = 13,
    MkDir {
        path: Path,
        attrs: Attrs,
    } = 14,
    RmDir {
        path: Path,
    } = 15,
    RealPath {
        path: Path,
    } = 16,
    Stat {
        path: Path,
    } = 17,
    Rename {
        old_path: Path,
        new_path: Path,
    } = 18,
    ReadLink {
        path: Path,
    } = 19,
    Symlink {
        link_path: Path,
        target_path: Path,
    } = 20,

    Status(Status) = 101,
    Handle(Bytes) = 102,
    Data(Bytes) = 103,
    Name {
        filename: Path,
        long_name: Path,
        attrs: Attrs,
    } = 104,

    #[tagged_enum_serde(off)]
    #[tagged_enum_derives(Debug, PartialEq, Eq, Clone, Copy, Default)]
    Attrs {
        size: Option<u64>,
        owner: Option<FileOwner>,
        perms: Option<u32>,
        time: Option<FileTime>,
    } = 105,

    Extended(ExtendedRequest) = 200,
    ExtendedReply(ExtendedReply) = 201,
}

impl Message {
    pub fn encode(&self, id: u32) -> Result<Bytes, Error> {
        let id = match self {
            Message::Init(Version { version, .. }) => *version,
            Message::Version(Version { version, .. }) => *version,
            _ => id,
        };
        let mut encoder = SftpEncoder::new(Vec::with_capacity(16), id);

        // Reserve space for frame length
        encoder.buf.put_u32(0);

        self.serialize(&mut encoder)?;

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

        let message = Message::deserialize(&mut decoder).map_err(Into::into)?;
        let id = decoder.get_id().unwrap_or_default();

        Ok((id, message))
    }
}

impl From<Error> for Message {
    fn from(_: Error) -> Self {
        Message::Status(Status {
            code: StatusCode::BadMessage as u32,
            error: "Bad Message".into(),
            language: "en".into(),
        })
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
