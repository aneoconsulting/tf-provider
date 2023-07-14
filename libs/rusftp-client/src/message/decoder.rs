use bytes::Buf;
use serde::{
    de::{self, IntoDeserializer},
    Deserializer,
};

#[derive(Debug)]
pub enum Error {
    NotEnoughData,
    Unsupported,
    InvalidChar,
    Custom(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotEnoughData => f.write_str("Decode Error: Not enough data"),
            Error::Unsupported => f.write_str("Decode Error: Unsupported"),
            Error::InvalidChar => f.write_str("Decode Error: Invalid character"),
            Error::Custom(msg) => write!(f, "Decode Error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub struct SftpDecoder<'de> {
    pub(crate) buf: &'de [u8],
}

impl<'de> SftpDecoder<'de> {
    fn decode_bool(&mut self) -> Result<bool, Error> {
        if self.buf.remaining() < std::mem::size_of::<u8>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u8() != 0)
    }
    fn decode_u8(&mut self) -> Result<u8, Error> {
        if self.buf.remaining() < std::mem::size_of::<u8>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u8())
    }
    fn decode_u16(&mut self) -> Result<u16, Error> {
        if self.buf.remaining() < std::mem::size_of::<u16>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u16())
    }
    fn decode_u32(&mut self) -> Result<u32, Error> {
        if self.buf.remaining() < std::mem::size_of::<u32>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u32())
    }
    fn decode_u64(&mut self) -> Result<u64, Error> {
        if self.buf.remaining() < std::mem::size_of::<u64>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u64())
    }
    fn decode_u128(&mut self) -> Result<u128, Error> {
        if self.buf.remaining() < std::mem::size_of::<u128>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_u128())
    }
    fn decode_f32(&mut self) -> Result<f32, Error> {
        if self.buf.remaining() < std::mem::size_of::<f32>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_f32())
    }
    fn decode_f64(&mut self) -> Result<f64, Error> {
        if self.buf.remaining() < std::mem::size_of::<f64>() {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(self.buf.get_f64())
    }
    fn decode_bytes(&mut self) -> Result<&'de [u8], Error> {
        let len = self.decode_u32()? as usize;
        if self.buf.remaining() < len {
            return Err(Error::NotEnoughData).unwrap();
        }
        Ok(&self.buf[0..len])
    }
    fn decode_str(&mut self) -> Result<&'de str, Error> {
        match std::str::from_utf8(self.decode_bytes()?) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::InvalidChar).unwrap(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut SftpDecoder<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.decode_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.decode_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.decode_u16()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.decode_u32()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.decode_u64()? as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i128(self.decode_u128()? as i128)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.decode_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.decode_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.decode_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.decode_u64()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u128(self.decode_u128()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.decode_f32()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.decode_f64()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(c) = char::from_u32(self.decode_u32()?) {
            visitor.visit_char(c)
        } else {
            Err(Error::InvalidChar)
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.decode_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.decode_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.decode_bytes()?)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let nel = self.decode_u32()? as usize;
        self.deserialize_tuple(nel, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            nel: len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let nel = self.decode_u32()? as usize;
        visitor.visit_map(SftpDecoderSeq { decoder: self, nel })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SftpDecoderSeq<'a, 'de> {
    decoder: &'a mut SftpDecoder<'de>,
    nel: usize,
}

impl<'a, 'de> de::SeqAccess<'de> for SftpDecoderSeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.nel == 0 {
            return Ok(None);
        }

        self.nel -= 1;
        Ok(Some(seed.deserialize(&mut *self.decoder)?))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.nel)
    }
}
impl<'a, 'de> de::MapAccess<'de> for SftpDecoderSeq<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.nel == 0 {
            return Ok(None);
        }

        self.nel -= 1;
        Ok(Some(seed.deserialize(&mut *self.decoder)?))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.decoder)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.nel)
    }
}

impl<'a, 'de> de::EnumAccess<'de> for &'a mut SftpDecoder<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.decode_u32()?;
        Ok((seed.deserialize(variant.into_deserializer())?, self))
    }
}
impl<'a, 'de> de::VariantAccess<'de> for &'a mut SftpDecoder<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }
}
