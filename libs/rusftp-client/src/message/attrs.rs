/*
SSH_FXP_ATTRS: 105
| u32: id | u32: attr flags | {if size in flags} u64: size | {if uid/gid in flags} u32 : uid | {if uid/gid in flags} u32 : gid | {if perm in flags} u32: perms | {if acmodtime in flags} u32: atime | {if acmodtime in flags} u32: mtime |

    Flags:
        SIZE: 0x00000001
        UIDGID: 0x00000002
        PERMISSIONS: 0x00000004
        ACMODTIME: 0x00000008

    File type: (on permission)
        FIFO: 0x1000
        CHR: 0x2000
        DIR: 0x4000
        BLK: 0x6000
        REG: 0x8000
        LNK: 0xA000
        NAM: 0x5000
 */

use crate::decode::SftpDecode;
use crate::encode::SftpEncode;
use crate::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
pub enum FilePermisions {
    // Permissions for others
    OX = 0x0001,
    OW = 0x0002,
    OR = 0x0004,
    // Permissions for group
    GX = 0x0008,
    GW = 0x0010,
    GR = 0x0020,
    // Permissions for user
    UX = 0x0040,
    UW = 0x0080,
    UR = 0x0100,
    // Special permissions
    SX = 0x0200,
    SW = 0x0400,
    SR = 0x0800,
    // File type
    FIFO = 0x1000,
    CHR = 0x2000,
    DIR = 0x4000,
    BLK = 0x6000,
    REG = 0x8000,
    LNK = 0xA000,
    NAM = 0x5000,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FileOwner {
    pub uid: u32,
    pub gid: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct FileTime {
    pub atime: u32,
    pub mtime: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct FileAttrs {
    pub size: Option<u64>,
    pub owner: Option<FileOwner>,
    pub perms: Option<u32>,
    pub time: Option<FileTime>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
#[non_exhaustive]
enum AttrFlags {
    Size = 0x00000001,
    Owner = 0x00000002,
    Perms = 0x00000004,
    Time = 0x00000008,
}

impl SftpDecode for FileAttrs {
    fn decode(buf: &mut dyn bytes::Buf) -> Result<Self, Error> {
        let mut attrs = FileAttrs::default();
        let attr_flags = u32::decode(buf)?;

        if (attr_flags & AttrFlags::Size as u32) != 0 {
            attrs.size = Some(u64::decode(buf)?);
        }
        if (attr_flags & AttrFlags::Owner as u32) != 0 {
            let uid = u32::decode(buf)?;
            let gid = u32::decode(buf)?;
            attrs.owner = Some(FileOwner { uid, gid });
        }
        if (attr_flags & AttrFlags::Perms as u32) != 0 {
            attrs.perms = Some(u32::decode(buf)?);
        }
        if (attr_flags & AttrFlags::Time as u32) != 0 {
            let atime = u32::decode(buf)?;
            let mtime = u32::decode(buf)?;
            attrs.time = Some(FileTime { atime, mtime });
        }

        Ok(attrs)
    }
}

impl SftpEncode for &FileAttrs {
    fn encode(self, buf: &mut dyn bytes::BufMut) -> Result<(), Error> {
        let mut attr_flags = 0u32;

        if self.size.is_some() {
            attr_flags |= AttrFlags::Size as u32;
        }
        if self.owner.is_some() {
            attr_flags |= AttrFlags::Owner as u32;
        }
        if self.perms.is_some() {
            attr_flags |= AttrFlags::Perms as u32;
        }
        if self.time.is_some() {
            attr_flags |= AttrFlags::Time as u32;
        }

        attr_flags.encode(buf)?;

        if let Some(size) = self.size {
            size.encode(buf)?;
        }
        if let Some(owner) = self.owner {
            owner.uid.encode(buf)?;
            owner.gid.encode(buf)?;
        }
        if let Some(perms) = self.perms {
            perms.encode(buf)?;
        }
        if let Some(time) = self.time {
            time.atime.encode(buf)?;
            time.mtime.encode(buf)?;
        }

        Ok(())
    }
}
