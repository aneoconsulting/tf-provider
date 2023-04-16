/*
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
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::FileAttrs;

pub type Name = Vec<SingleName>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleName {
    pub filename: Bytes,
    pub long_name: Bytes,
    pub attrs: FileAttrs,
}

impl SftpDecode for SingleName {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let filename = Bytes::decode(buf)?;
        let long_name = Bytes::decode(buf)?;
        let attrs = FileAttrs::decode(buf)?;

        Ok(SingleName {
            filename,
            long_name,
            attrs,
        })
    }
}

impl SftpEncode for &SingleName {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.filename.encode(buf)?;
        self.long_name.encode(buf)?;
        self.attrs.encode(buf)?;

        Ok(())
    }
}
