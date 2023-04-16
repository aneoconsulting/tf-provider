/*
SSH_FXP_VERSION: 2
(VERSION) | u32: version | u32: ext0 name length | u8[ext0 name length]: ext0 name | u32: ext0 value length | u8[ext0 value length]: ext0 value | ...
*/

use std::collections::BTreeMap;

use bytes::Bytes;

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Version {
    pub version: u32,
    pub extensions: BTreeMap<Bytes, Bytes>,
}

impl SftpDecode for Version {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let mut version = Version {
            version: 0,
            extensions: Default::default(),
        };

        while buf.remaining() >= 2 * std::mem::size_of::<u32>() {
            let key = Bytes::decode(buf)?;
            let val = Bytes::decode(buf)?;
            version.extensions.insert(key, val);
        }

        Ok(version)
    }
}

impl SftpEncode for &Version {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        for (key, val) in &self.extensions {
            key.encode(buf)?;
            val.encode(buf)?;
        }
        Ok(())
    }
}
