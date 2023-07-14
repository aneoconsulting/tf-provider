use bytes::{Buf, Bytes};

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

mod attrs;
mod data;
mod extended_reply;
mod extended_request;
mod handle;
mod name;
mod open;
mod path;
mod read;
mod rename;
mod status;
mod symlink;
mod version;
mod write;

pub mod decoder;
pub mod encoder;

pub use attrs::{FileAttrs, FileOwner, FilePermisions, FileTime};
pub use data::Data;
pub use extended_reply::ExtendedReply;
pub use extended_request::ExtendedRequest;
pub use handle::{Handle, HandleAttrs};
pub use name::{Name, SingleName};
pub use open::{Open, PFlags};
pub use path::{Path, PathAttrs};
pub use read::Read;
pub use rename::Rename;
pub use status::{Status, StatusCode};
pub use symlink::Symlink;
pub use version::Version;
pub use write::Write;

pub type Init = Version;
pub type Close = Handle;
pub type LStat = Path;
pub type FStat = Handle;
pub type SetStat = PathAttrs;
pub type FSetStat = HandleAttrs;
pub type OpenDir = Path;
pub type ReadDir = Handle;
pub type Remove = Path;
pub type MkDir = PathAttrs;
pub type RmDir = Path;
pub type RealPath = Path;
pub type Stat = Path;
pub type ReadLink = Path;

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(u8)]
#[non_exhaustive]
pub enum Message {
    Init(Init) = 1,
    Version(Version) = 2,
    Open(Open) = 3,
    Close(Close) = 4,
    Read(Read) = 5,
    Write(Write) = 6,
    LStat(LStat) = 7,
    FStat(FStat) = 8,
    SetStat(SetStat) = 9,
    FSetStat(FSetStat) = 10,
    OpenDir(OpenDir) = 11,
    ReadDir(ReadDir) = 12,
    Remove(Remove) = 13,
    MkDir(MkDir) = 14,
    RmDir(RmDir) = 15,
    RealPath(RealPath) = 16,
    Stat(Stat) = 17,
    Rename(Rename) = 18,
    ReadLink(ReadLink) = 19,
    Symlink(Symlink) = 20,

    Status(Status) = 101,
    Handle(Handle) = 102,
    Data(Data) = 103,
    Name(Name) = 104,
    Attrs(FileAttrs) = 105,

    Extended(ExtendedRequest) = 200,
    ExtendedReply(ExtendedReply) = 201,
}

impl Message {
    fn kind(&self) -> u8 {
        match self {
            Self::Init(_) => 1,
            Self::Version(_) => 2,
            Self::Open(_) => 3,
            Self::Close(_) => 4,
            Self::Read(_) => 5,
            Self::Write(_) => 6,
            Self::LStat(_) => 7,
            Self::FStat(_) => 8,
            Self::SetStat(_) => 9,
            Self::FSetStat(_) => 10,
            Self::OpenDir(_) => 11,
            Self::ReadDir(_) => 12,
            Self::Remove(_) => 13,
            Self::MkDir(_) => 14,
            Self::RmDir(_) => 15,
            Self::RealPath(_) => 16,
            Self::Stat(_) => 17,
            Self::Rename(_) => 18,
            Self::ReadLink(_) => 19,
            Self::Symlink(_) => 20,
            Self::Status(_) => 101,
            Self::Handle(_) => 102,
            Self::Data(_) => 103,
            Self::Name(_) => 104,
            Self::Attrs(_) => 105,
            Self::Extended(_) => 200,
            Self::ExtendedReply(_) => 201,
        }
    }

    pub fn encode(&self, id: u32) -> Result<Bytes, Error> {
        let mut vec = Vec::with_capacity(16);

        let buf = &mut vec;

        // Reserve space for frame length
        u32::encode(0, buf)?;

        // Type of the message
        self.kind().encode(buf)?;

        // ID of the message or the version of the protocol
        match self {
            Self::Init(inner) => inner.version.encode(buf)?,
            Self::Version(inner) => inner.version.encode(buf)?,
            _ => id.encode(buf)?,
        }

        // Encode the rest of the frame
        match self {
            Self::Init(inner) => inner.encode(buf)?,
            Self::Version(inner) => inner.encode(buf)?,
            Self::Open(inner) => inner.encode(buf)?,
            Self::Close(inner) => inner.encode(buf)?,
            Self::Read(inner) => inner.encode(buf)?,
            Self::Write(inner) => inner.encode(buf)?,
            Self::LStat(inner) => inner.encode(buf)?,
            Self::FStat(inner) => inner.encode(buf)?,
            Self::SetStat(inner) => inner.encode(buf)?,
            Self::FSetStat(inner) => inner.encode(buf)?,
            Self::OpenDir(inner) => inner.encode(buf)?,
            Self::ReadDir(inner) => inner.encode(buf)?,
            Self::Remove(inner) => inner.encode(buf)?,
            Self::MkDir(inner) => inner.encode(buf)?,
            Self::RmDir(inner) => inner.encode(buf)?,
            Self::RealPath(inner) => inner.encode(buf)?,
            Self::Stat(inner) => inner.encode(buf)?,
            Self::Rename(inner) => inner.encode(buf)?,
            Self::ReadLink(inner) => inner.encode(buf)?,
            Self::Symlink(inner) => inner.encode(buf)?,
            Self::Status(inner) => inner.encode(buf)?,
            Self::Handle(inner) => inner.encode(buf)?,
            Self::Data(inner) => inner.encode(buf)?,
            Self::Name(inner) => inner.as_slice().encode(buf)?,
            Self::Attrs(inner) => inner.encode(buf)?,
            Self::Extended(inner) => inner.encode(buf)?,
            Self::ExtendedReply(inner) => inner.encode(buf)?,
        }

        // write frame length at the beginning of the frame
        let frame_length = (vec.len() - 4) as u32;
        let mut buf = vec.as_mut_slice();
        frame_length.encode(&mut buf)?;

        Ok(vec.into())
    }

    pub fn decode(buf: &mut dyn Buf) -> Result<(u32, Self), Error> {
        let frame_length = u32::decode(buf)?;

        // Limit the read to this very frame
        let mut buf = buf.take(frame_length as usize);
        let buf = &mut buf;

        // Type of the message
        let kind = u8::decode(buf)?;
        // ID of the message or version of the protocol
        let id = u32::decode(buf)?;

        match kind {
            1 => {
                let mut init = Init::decode(buf)?;
                init.version = id;
                Ok((0, Self::Init(init)))
            }
            2 => {
                let mut version = Version::decode(buf)?;
                version.version = id;
                Ok((0, Self::Version(version)))
            }
            3 => Ok((id, Self::Open(SftpDecode::decode(buf)?))),
            4 => Ok((id, Self::Close(SftpDecode::decode(buf)?))),
            5 => Ok((id, Self::Read(SftpDecode::decode(buf)?))),
            6 => Ok((id, Self::Write(SftpDecode::decode(buf)?))),
            7 => Ok((id, Self::LStat(SftpDecode::decode(buf)?))),
            8 => Ok((id, Self::FStat(SftpDecode::decode(buf)?))),
            9 => Ok((id, Self::SetStat(SftpDecode::decode(buf)?))),
            10 => Ok((id, Self::FSetStat(SftpDecode::decode(buf)?))),
            11 => Ok((id, Self::OpenDir(SftpDecode::decode(buf)?))),
            12 => Ok((id, Self::ReadDir(SftpDecode::decode(buf)?))),
            13 => Ok((id, Self::Remove(SftpDecode::decode(buf)?))),
            14 => Ok((id, Self::MkDir(SftpDecode::decode(buf)?))),
            15 => Ok((id, Self::RmDir(SftpDecode::decode(buf)?))),
            16 => Ok((id, Self::RealPath(SftpDecode::decode(buf)?))),
            17 => Ok((id, Self::Stat(SftpDecode::decode(buf)?))),
            18 => Ok((id, Self::Rename(SftpDecode::decode(buf)?))),
            19 => Ok((id, Self::ReadLink(SftpDecode::decode(buf)?))),
            20 => Ok((id, Self::Symlink(SftpDecode::decode(buf)?))),
            101 => Ok((id, Self::Status(SftpDecode::decode(buf)?))),
            102 => Ok((id, Self::Handle(SftpDecode::decode(buf)?))),
            103 => Ok((id, Self::Data(SftpDecode::decode(buf)?))),
            104 => Ok((id, Self::Name(SftpDecode::decode(buf)?))),
            105 => Ok((id, Self::Attrs(SftpDecode::decode(buf)?))),
            200 => Ok((id, Self::Extended(SftpDecode::decode(buf)?))),
            201 => Ok((id, Self::ExtendedReply(SftpDecode::decode(buf)?))),
            _ => Err(Error),
        }
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

macro_rules! strong_alias {
    ($new:ident: $existing:ty) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $new($existing);

        impl crate::decode::SftpDecode for $new {
            fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, crate::Error> {
                Ok($new(<$existing>::decode(buf)?))
            }
        }

        impl crate::encode::SftpEncode for &$new {
            fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), crate::Error> {
                self.0.encode(buf)
            }
        }
    };
}

pub(self) use strong_alias;
