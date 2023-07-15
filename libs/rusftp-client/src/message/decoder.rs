use bytes::Buf;
use serde::{
    de::{self, IntoDeserializer, SeqAccess},
    Deserializer,
};

use crate::Error;

pub struct SftpDecoder<'de> {
    pub(crate) buf: &'de [u8],
    id: Option<u32>,
    current_field: &'static str,
}

impl<'de> SftpDecoder<'de> {
    pub fn new(buf: &'de [u8]) -> Self {
        Self {
            buf,
            id: None,
            current_field: "",
        }
    }
    pub fn get_id(&self) -> Option<u32> {
        self.id
    }
    fn decode_length(&self) -> bool {
        !self.current_field.ends_with("_implicit_length")
    }
    fn skip_field(&self) -> Option<u32> {
        if matches!(self.current_field, "id" | "version") {
            self.id
        } else {
            None
        }
    }
    fn decode_id(&mut self) -> Result<(), Error> {
        if self.id.is_none() {
            if self.buf.remaining() < std::mem::size_of::<u32>() {
                return Err(Error::NotEnoughData);
            }
            self.id = Some(self.buf.get_u32());
        }
        Ok(())
    }

    fn decode_bool(&mut self) -> Result<bool, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id != 0);
        }
        if self.buf.remaining() < std::mem::size_of::<u8>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u8() != 0;
        self.decode_id()?;
        Ok(value)
    }
    fn decode_u8(&mut self) -> Result<u8, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as u8);
        }
        if self.buf.remaining() < std::mem::size_of::<u8>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u8();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_u16(&mut self) -> Result<u16, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as u16);
        }
        if self.buf.remaining() < std::mem::size_of::<u16>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u16();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_u32(&mut self) -> Result<u32, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id);
        }
        if self.buf.remaining() < std::mem::size_of::<u32>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u32();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_u64(&mut self) -> Result<u64, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as u64);
        }
        if self.buf.remaining() < std::mem::size_of::<u64>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u64();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_u128(&mut self) -> Result<u128, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as u128);
        }
        if self.buf.remaining() < std::mem::size_of::<u128>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_u128();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_f32(&mut self) -> Result<f32, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as f32);
        }
        if self.buf.remaining() < std::mem::size_of::<f32>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_f32();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_f64(&mut self) -> Result<f64, Error> {
        if let Some(id) = self.skip_field() {
            return Ok(id as f64);
        }
        if self.buf.remaining() < std::mem::size_of::<f64>() {
            return Err(Error::NotEnoughData);
        }
        let value = self.buf.get_f64();
        self.decode_id()?;
        Ok(value)
    }
    fn decode_bytes(&mut self) -> Result<&'de [u8], Error> {
        let len = if self.decode_length() {
            self.decode_u32()? as usize
        } else {
            self.buf.remaining()
        };
        let Some(bytes) = self.buf.get(0..len) else {
            return Err(Error::NotEnoughData);
        };
        self.buf.advance(len);
        Ok(bytes)
    }
    fn decode_str(&mut self) -> Result<&'de str, Error> {
        match std::str::from_utf8(self.decode_bytes()?) {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::InvalidChar),
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
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.current_field = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let nel = if self.decode_length() {
            Some(self.decode_u32()? as usize)
        } else {
            None
        };
        self.current_field = "";
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            fields: &[],
            nel,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.current_field = "";
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            fields: &[],
            nel: Some(len),
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.current_field = name;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let nel = if self.decode_length() {
            Some(self.decode_u32()? as usize)
        } else {
            None
        };
        self.current_field = "";
        visitor.visit_map(SftpDecoderSeq {
            decoder: self,
            fields: &[],
            nel,
        })
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.current_field = name;
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            fields,
            nel: Some(fields.len()),
        })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.current_field = name;
        visitor.visit_enum(SftpDecoderEnum {
            decoder: self,
            variants,
        })
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
    fields: &'static [&'static str],
    nel: Option<usize>,
}

impl<'a, 'de> de::SeqAccess<'de> for SftpDecoderSeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.fields {
            [] => self.decoder.current_field = "",
            [field, fields @ ..] => {
                self.decoder.current_field = field;
                self.fields = fields;
            }
        }
        eprintln!("current field: {}", self.decoder.current_field);
        match self.nel {
            Some(0) => Ok(None),
            Some(nel) => {
                self.nel = Some(nel - 1);
                Ok(Some(seed.deserialize(&mut *self.decoder)?))
            }
            None => match seed.deserialize(&mut *self.decoder) {
                Ok(value) => Ok(Some(value)),
                Err(Error::NotEnoughData) => Ok(None),
                Err(err) => Err(err),
            },
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.nel
    }
}
impl<'a, 'de> de::MapAccess<'de> for SftpDecoderSeq<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.next_element_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.decoder)
    }

    fn size_hint(&self) -> Option<usize> {
        self.nel
    }
}

pub struct SftpDecoderEnum<'a, 'de> {
    decoder: &'a mut SftpDecoder<'de>,
    variants: &'static [&'static str],
}
pub struct SftpDecoderVariant<'a, 'de> {
    decoder: &'a mut SftpDecoder<'de>,
    variant: &'static str,
}

impl<'a, 'de> de::EnumAccess<'de> for SftpDecoderEnum<'a, 'de> {
    type Error = Error;
    type Variant = SftpDecoderVariant<'a, 'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.decoder.decode_u32()?;
        Ok((
            seed.deserialize(variant.into_deserializer())?,
            SftpDecoderVariant {
                decoder: self.decoder,
                variant: self.variants.get(variant as usize).unwrap_or(&""),
            },
        ))
    }
}
impl<'a, 'de> de::VariantAccess<'de> for SftpDecoderVariant<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.decoder.current_field = self.variant;
        seed.deserialize(self.decoder)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.current_field = self.variant;
        self.decoder.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decoder.current_field = self.variant;
        self.decoder.deserialize_tuple(fields.len(), visitor)
    }
}
