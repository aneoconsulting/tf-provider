use bytes::BufMut;
use serde::ser;

use crate::Error;

pub struct SftpEncoder {
    pub(crate) buf: Vec<u8>,
    pub(crate) id: Option<u32>,
    current_field: &'static str,
}

impl SftpEncoder {
    pub fn new(buf: Vec<u8>, id: u32) -> Self {
        Self {
            buf,
            id: Some(id),
            current_field: "",
        }
    }

    fn serialize_id(&mut self) -> Result<(), Error> {
        if let Some(id) = self.id {
            self.id = None;
            if self.buf.remaining_mut() >= std::mem::size_of::<u32>() {
                self.buf.put_u32(id);
            } else {
                return Err(Error::NotEnoughData);
            }
        }
        Ok(())
    }

    fn encode_length(&self) -> bool {
        !self.current_field.ends_with("_implicit_length")
    }
    fn skip_field(&self) -> bool {
        matches!(self.current_field, "id" | "version")
    }
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

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u8>() {
            self.buf.put_u8(v as u8);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<i8>() {
            self.buf.put_i8(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<i16>() {
            self.buf.put_i16(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<i32>() {
            self.buf.put_i32(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<i64>() {
            self.buf.put_i64(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<i128>() {
            self.buf.put_i128(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u8>() {
            self.buf.put_u8(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u16>() {
            self.buf.put_u16(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u32>() {
            self.buf.put_u32(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u64>() {
            self.buf.put_u64(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u128>() {
            self.buf.put_u128(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<f32>() {
            self.buf.put_f32(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<f64>() {
            self.buf.put_f64(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        if self.buf.remaining_mut() >= std::mem::size_of::<u32>() {
            self.buf.put_u32(v as u32);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        let len = v.len();

        if len >= 0x100000000 {
            return Err(Error::Unsupported);
        }
        if self.buf.remaining_mut() >= len + std::mem::size_of::<u32>() {
            if self.encode_length() {
                self.buf.put_u32(len as u32);
            }
            self.buf.put(v);
            self.serialize_id()
        } else {
            Err(Error::NotEnoughData)
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        self.serialize_id()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        self.serialize_id()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        if self.skip_field() {
            return Ok(());
        }
        self.serialize_id()
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
            if len >= 0x100000000 {
                return Err(Error::Unsupported);
            }
            if self.encode_length() {
                self.serialize_u32(len as u32)?;
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
            if len >= 0x100000000 {
                return Err(Error::Unsupported);
            }
            if self.encode_length() {
                self.serialize_u32(len as u32)?;
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

impl<'a> ser::SerializeSeq for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl<'a> ser::SerializeTupleStruct for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl<'a> ser::SerializeTupleVariant for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
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
impl<'a> ser::SerializeStruct for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.current_field = key;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
impl<'a> ser::SerializeStructVariant for &'a mut SftpEncoder {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.current_field = key;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
