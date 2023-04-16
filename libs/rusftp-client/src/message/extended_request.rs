/*
SSH_FXP_EXTENDED: 200
| u32: id | u32: request length | u8[request length]: request | u8[frame length - 9 - request length]: data |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedRequest {
    pub request: Bytes,
    pub data: Bytes,
}

impl SftpDecode for ExtendedRequest {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let request = Bytes::decode(buf)?;
        let data = buf.copy_to_bytes(buf.remaining());
        Ok(ExtendedRequest { request, data })
    }
}

impl SftpEncode for &ExtendedRequest {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.request.encode(buf)?;
        buf.put_slice(self.data.as_ref());

        Ok(())
    }
}
