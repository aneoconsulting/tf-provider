/*
SSH_FXP_RENAME: 18
| u32: id | u32: old path length | u8[old path length]: old path | u32: new path length | u8[new path length]: new path |
 */

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rename {
    pub old_path: Path,
    pub new_path: Path,
}

impl SftpDecode for Rename {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let old_path = Path::decode(buf)?;
        let new_path = Path::decode(buf)?;
        Ok(Rename { old_path, new_path })
    }
}

impl SftpEncode for &Rename {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.old_path.encode(buf)?;
        self.new_path.encode(buf)?;

        Ok(())
    }
}
