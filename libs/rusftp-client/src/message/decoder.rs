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

macro_rules! decode {
    ($decode:ident, $get:ident, $ty:ty) => {
        fn $decode(&mut self) -> Result<$ty, Error> {
            if let Some(id) = self.skip_field() {
                return Ok(id as $ty);
            }
            if self.buf.remaining() < std::mem::size_of::<$ty>() {
                return Err(Error::NotEnoughData);
            }
            let value = self.buf.$get();
            self.decode_id()?;
            Ok(value)
        }
    };
}
macro_rules! deserialize {
    ($deserialize:ident, $visit:ident, $decode:ident, $ty:ty) => {
        fn $deserialize<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.$visit(self.$decode()? as $ty)
        }
    };
    ($deserialize:ident) => {
        fn $deserialize<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
            Err(Error::Unsupported)
        }
    };
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

    decode!(decode_u8, get_u8, u8);
    decode!(decode_u16, get_u16, u16);
    decode!(decode_u32, get_u32, u32);
    decode!(decode_u64, get_u64, u64);
    decode!(decode_u128, get_u128, u128);
    decode!(decode_f32, get_f32, f32);
    decode!(decode_f64, get_f64, f64);
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

    deserialize!(deserialize_any);
    deserialize!(deserialize_i8, visit_i8, decode_u8, i8);
    deserialize!(deserialize_i16, visit_i16, decode_u16, i16);
    deserialize!(deserialize_i32, visit_i32, decode_u32, i32);
    deserialize!(deserialize_i64, visit_i64, decode_u64, i64);
    deserialize!(deserialize_i128, visit_i128, decode_u128, i128);
    deserialize!(deserialize_u8, visit_u8, decode_u8, _);
    deserialize!(deserialize_u16, visit_u16, decode_u16, _);
    deserialize!(deserialize_u32, visit_u32, decode_u32, _);
    deserialize!(deserialize_u64, visit_u64, decode_u64, _);
    deserialize!(deserialize_u128, visit_u128, decode_u128, _);
    deserialize!(deserialize_f32, visit_f32, decode_f32, _);
    deserialize!(deserialize_f64, visit_f64, decode_f64, _);
    deserialize!(deserialize_str, visit_borrowed_str, decode_str, _);
    deserialize!(deserialize_string, visit_borrowed_str, decode_str, _);
    deserialize!(deserialize_bytes, visit_borrowed_bytes, decode_bytes, _);
    deserialize!(deserialize_byte_buf, visit_borrowed_bytes, decode_bytes, _);
    deserialize!(deserialize_option);
    deserialize!(deserialize_identifier);
    deserialize!(deserialize_ignored_any);

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_bool(self.decode_u8()? != 0)
    }
    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if let Some(c) = char::from_u32(self.decode_u32()?) {
            visitor.visit_char(c)
        } else {
            Err(Error::InvalidChar)
        }
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.current_field = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let seq = SftpDecoderSeq::new(self)?;
        seq.decoder.current_field = "";
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.current_field = "";
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            fields: &[],
            nel: Some(len),
        })
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.current_field = name;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let seq = SftpDecoderSeq::new(self)?;
        seq.decoder.current_field = "";
        visitor.visit_map(seq)
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.current_field = name;
        visitor.visit_seq(SftpDecoderSeq {
            decoder: self,
            fields,
            nel: Some(fields.len()),
        })
    }

    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.current_field = name;
        visitor.visit_enum(SftpDecoderEnum {
            decoder: self,
            variants,
        })
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

impl<'a, 'de> SftpDecoderSeq<'a, 'de> {
    fn new(decoder: &'a mut SftpDecoder<'de>) -> Result<Self, Error> {
        let nel = if decoder.decode_length() {
            Some(decoder.decode_u32()? as usize)
        } else {
            None
        };
        Ok(Self {
            decoder,
            fields: &[],
            nel,
        })
    }
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

impl<'a, 'de> de::EnumAccess<'de> for SftpDecoderEnum<'a, 'de> {
    type Error = Error;
    type Variant = &'a mut SftpDecoder<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.decoder.decode_u32()?;
        self.decoder.current_field = self.variants.get(variant as usize).unwrap_or(&"");
        Ok((seed.deserialize(variant.into_deserializer())?, self.decoder))
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
