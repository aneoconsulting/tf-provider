use bytes::{Buf, Bytes};

use crate::error::Error;

pub(crate) trait SftpDecode: Sized {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error>;
}

/**
 * Decode unsigned integers
 */
impl SftpDecode for u8 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<u8>() {
            Ok(buf.get_u8())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for u16 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<u16>() {
            Ok(buf.get_u16())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for u32 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<u32>() {
            Ok(buf.get_u32())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for u64 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<u64>() {
            Ok(buf.get_u64())
        } else {
            Err(Error)
        }
    }
}

/**
 * Decode signed integers
 */
impl SftpDecode for i8 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<i8>() {
            Ok(buf.get_i8())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for i16 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<i16>() {
            Ok(buf.get_i16())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for i32 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<i32>() {
            Ok(buf.get_i32())
        } else {
            Err(Error)
        }
    }
}

impl SftpDecode for i64 {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        if buf.remaining() >= std::mem::size_of::<i64>() {
            Ok(buf.get_i64())
        } else {
            Err(Error)
        }
    }
}

/**
 * Containers
 */

impl<T: SftpDecode> SftpDecode for Vec<T> {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        let nel = u32::decode(buf)? as usize;

        let mut vec = Vec::with_capacity(nel);

        for _ in 0..nel {
            vec.push(T::decode(buf)?);
        }

        Ok(vec)
    }
}

impl SftpDecode for Bytes {
    fn decode(buf: &mut dyn Buf) -> Result<Self, Error> {
        let length = u32::decode(buf)? as usize;
        if buf.remaining() >= length {
            Ok(buf.copy_to_bytes(length))
        } else {
            Err(Error)
        }
    }
}
