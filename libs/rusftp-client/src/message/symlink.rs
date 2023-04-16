/*
SSH_FXP_SYMLINK: 20
| u32: id | u32: link path length | u8[link path length]: link path | u32: target path length | u8[target path length]: target path |
 */

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

use super::Path;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symlink {
    pub link_path: Path,
    pub target_path: Path,
}

impl SftpDecode for Symlink {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let link_path = Path::decode(buf)?;
        let target_path = Path::decode(buf)?;
        Ok(Symlink {
            link_path,
            target_path,
        })
    }
}

impl SftpEncode for &Symlink {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        self.link_path.encode(buf)?;
        self.target_path.encode(buf)?;

        Ok(())
    }
}
