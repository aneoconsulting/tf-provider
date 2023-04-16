/*
SSH_FXP_WRITE: 6
| u32: id | u32: handle length | u8[handle length]: handle | u64: offset | u32: data length | u8[data length]: data |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::Handle;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Write {
    pub handle: Handle,
    pub offset: u64,
    pub data: Bytes,
}

impl SftpDecode for Write {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let handle = Handle::decode(buf)?;
        let offset = u64::decode(buf)?;
        let data = Bytes::decode(buf)?;
        Ok(Write {
            handle,
            offset,
            data,
        })
    }
}

impl SftpEncode for &Write {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.handle.encode(buf)?;
        self.offset.encode(buf)?;
        self.data.encode(buf)?;

        Ok(())
    }
}
