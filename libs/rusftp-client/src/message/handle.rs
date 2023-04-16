/*
SSH_FXP_HANDLE: 102
(HANDLE) | u32: id | u32: handle length | u8[handle length]: handle |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::FileAttrs;

super::strong_alias!(Handle: Bytes);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HandleAttrs {
    pub handle: Handle,
    pub attrs: FileAttrs,
}

impl SftpDecode for HandleAttrs {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let handle = Handle::decode(buf)?;
        let attrs = FileAttrs::decode(buf)?;
        Ok(HandleAttrs { handle, attrs })
    }
}

impl SftpEncode for &HandleAttrs {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.handle.encode(buf)?;
        self.attrs.encode(buf)?;

        Ok(())
    }
}
