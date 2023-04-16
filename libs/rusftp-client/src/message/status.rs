/*
SSH_FXP_STATUS: 101
| u32: id | u32: status code | u32: error length | u8[error length]: error | u32: language length | u8[language length]: language |
 */

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum StatusCode {
    Ok = 0,
    Eof = 1,
    NoSuchFile = 2,
    PermissionDenied = 3,
    Failure = 4,
    BadMessage = 5,
    NoConnection = 6,
    ConnectionLost = 7,
    OpUnsupported = 8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Status {
    pub code: u32,
    pub error: Bytes,
    pub language: Bytes,
}

impl SftpDecode for Status {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let code = u32::decode(buf)?;
        let error = Bytes::decode(buf)?;
        let language = Bytes::decode(buf)?;
        Ok(Status {
            code,
            error,
            language,
        })
    }
}

impl SftpEncode for &Status {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.code.encode(buf)?;
        self.error.encode(buf)?;
        self.language.encode(buf)?;

        Ok(())
    }
}
