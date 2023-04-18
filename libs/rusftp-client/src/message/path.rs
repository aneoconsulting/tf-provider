use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::FileAttrs;

pub type Path = Bytes;
//super::strong_alias!(Path: Bytes);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PathAttrs {
    pub path: Path,
    pub attrs: FileAttrs,
}

impl SftpDecode for PathAttrs {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let path = Path::decode(buf)?;
        let attrs = FileAttrs::decode(buf)?;
        Ok(PathAttrs { path, attrs })
    }
}

impl SftpEncode for &PathAttrs {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.path.encode(buf)?;
        self.attrs.encode(buf)?;

        Ok(())
    }
}
