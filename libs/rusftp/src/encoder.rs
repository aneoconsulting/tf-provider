use bytes::BufMut;
use serde::ser;

use crate::Error;

pub struct SftpEncoder {
    pub(crate) buf: Vec<u8>,
    current_field: &'static str,
}

impl SftpEncoder {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf,
            current_field: "",
        }
    }

    fn encode_length(&self) -> bool {
        !self.current_field.ends_with("_implicit_length")
    }
}

macro_rules! serialize {
    ($serialize:ident, $put:ident, $ty:ty) => {
        fn $serialize(self, v: $ty) -> Result<Self::Ok, Self::Error> {
            if self.buf.remaining_mut() >= std::mem::size_of::<$ty>() {
                self.buf.$put(v);
                Ok(())
            } else {
                Err(Error::NotEnoughData)
            }
        }
    };
    (trait $trait:ident: $serialize:ident $($key:ident)?) => {
        impl<'a> ser::$trait for &'a mut SftpEncoder {
            type Ok = ();
            type Error = Error;

            fn $serialize<T: ?Sized>(&mut self, $($key: &'static str,)? value: &T) -> Result<(), Self::Error>
            where
                T: serde::Serialize,
            {
                $(self.current_field = $key;)?
                value.serialize(&mut **self)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }
        }
    };
}

impl<'a> ser::Serializer for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    serialize!(serialize_i8, put_i8, i8);
    serialize!(serialize_i16, put_i16, i16);
    serialize!(serialize_i32, put_i32, i32);
    serialize!(serialize_i64, put_i64, i64);
    serialize!(serialize_i128, put_i128, i128);
    serialize!(serialize_u8, put_u8, u8);
    serialize!(serialize_u16, put_u16, u16);
    serialize!(serialize_u32, put_u32, u32);
    serialize!(serialize_u64, put_u64, u64);
    serialize!(serialize_u128, put_u128, u128);
    serialize!(serialize_f32, put_f32, f32);
    serialize!(serialize_f64, put_f64, f64);

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.serialize_u8(v as u8)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(v as u32)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let Ok(len) = u32::try_from(v.len()) else {
            return Err(Error::Unsupported);
        };

        if self.buf.remaining_mut() >= len as usize + std::mem::size_of::<u32>() {
            if self.encode_length() {
                self.buf.put_u32(len);
            }
            self.buf.put(v);
            Ok(())
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.current_field = name;
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.current_field = variant;
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            let Ok(len) = u32::try_from(len) else {
                return Err(Error::Unsupported);
            };
            if self.encode_length() {
                self.serialize_u32(len)?;
            }
        }
        self.current_field = "";
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.current_field = "";
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.current_field = name;
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.current_field = variant;
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if let Some(len) = len {
            let Ok(len) = u32::try_from(len) else {
                return Err(Error::Unsupported);
            };
            if self.encode_length() {
                self.serialize_u32(len)?;
            }
        }
        self.current_field = "";
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.current_field = name;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.current_field = variant;
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

serialize!(trait SerializeSeq: serialize_element);
serialize!(trait SerializeTuple: serialize_element);
serialize!(trait SerializeTupleStruct: serialize_field);
serialize!(trait SerializeTupleVariant: serialize_field);
serialize!(trait SerializeStruct: serialize_field key);
serialize!(trait SerializeStructVariant: serialize_field key);

impl<'a> ser::SerializeMap for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
