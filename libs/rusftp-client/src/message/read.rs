/*
SSH_FXP_READ: 5
| u32: id | u32: handle length | u8[handle length]: handle | u64: offset | u32: length |
 */

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::Handle;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Read {
    pub handle: Handle,
    pub offset: u64,
    pub length: u32,
}

impl SftpDecode for Read {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let handle = Handle::decode(buf)?;
        let offset = u64::decode(buf)?;
        let length = u32::decode(buf)?;
        Ok(Read {
            handle,
            offset,
            length,
        })
    }
}

impl SftpEncode for &Read {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.handle.encode(buf)?;
        self.offset.encode(buf)?;
        self.length.encode(buf)?;

        Ok(())
    }
}
