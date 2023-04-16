/*
SSH_FXP_EXTENDED_REPLY: 201
| u32: id | u8[frame length - 5]: data |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExtendedReply {
    pub data: Bytes,
}

impl SftpDecode for ExtendedReply {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        Ok(ExtendedReply {
            data: buf.copy_to_bytes(buf.remaining()),
        })
    }
}

impl SftpEncode for &ExtendedReply {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        buf.put_slice(self.data.as_ref());

        Ok(())
    }
}
