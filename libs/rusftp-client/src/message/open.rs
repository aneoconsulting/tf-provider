/*
SSH_FXP_OPEN: 3
| u32: id | u32: filename length | u8[filename length]: filename | u32: pflags | u32: attr flags | {if size in flags} u64: size | {if uid/gid in flags} u32 : uid | {if uid/gid in flags} u32 : gid | {if perm in flags} u32: perms | {if acmodtime in flags} u32: atime | {if acmodtime in flags} u32: mtime |

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
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::FileAttrs;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Open {
    pub filename: Bytes,
    pub pflags: u32,
    pub attrs: FileAttrs,
}

impl SftpDecode for Open {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let filename = Bytes::decode(buf)?;
        let pflags = u32::decode(buf)?;
        let attrs = FileAttrs::decode(buf)?;

        Ok(Open {
            filename,
            pflags,
            attrs,
        })
    }
}

impl SftpEncode for &Open {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.filename.encode(buf)?;
        self.pflags.encode(buf)?;
        self.attrs.encode(buf)?;

        Ok(())
    }
}
