use bytes::BufMut;

use crate::error::Error;

pub(crate) trait SftpEncode {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error>;
}

impl<T: SftpEncode + Copy> SftpEncode for &T {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        (*self).encode(buf)
    }
}

/**
 * Decode unsigned integers
 */
impl SftpEncode for u8 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<u8>() {
            buf.put_u8(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for u16 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<u16>() {
            buf.put_u16(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for u32 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<u32>() {
            buf.put_u32(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for u64 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<u64>() {
            buf.put_u64(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/**
 * Decode signed integers
 */
impl SftpEncode for i8 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<i8>() {
            buf.put_i8(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for i16 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<i16>() {
            buf.put_i16(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for i32 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<i32>() {
            buf.put_i32(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

impl SftpEncode for i64 {
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        if buf.remaining_mut() >= std::mem::size_of::<i64>() {
            buf.put_i64(self);
            Ok(())
        } else {
            Err(Error)
        }
    }
}

/**
 * Containers
 */

impl<'a, T> SftpEncode for &'a [T]
where
    &'a T: SftpEncode,
{
    fn encode(self, buf: &mut dyn BufMut) -> Result<(), Error> {
        (self.len() as u32).encode(buf)?;
        for el in self {
            el.encode(buf)?;
        }
        Ok(())
    }
}
