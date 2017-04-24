use std;
use std::io::Write;
use std::marker::PhantomData;

use byteorder::{ByteOrder, WriteBytesExt};
use serde::ser;

use encapsulation::Encapsulation;
use error::{Error, ErrorKind, Result};
use size::{calc_serialized_size, calc_serialized_size_bounded, Infinite, SizeLimit};

pub struct Serializer<W, C> {
    writer: W,
    pos: u64,
    phantom: PhantomData<C>,
}

impl<W, C> Serializer<W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: writer,
            pos: 0,
            phantom: PhantomData,
        }
    }

    fn add_pos(&mut self, size: u64) -> Result<()> {
        self.pos += size;
        Ok(())
    }

    fn reset_pos(&mut self) -> Result<()> {
        self.pos = 0;
        Ok(())
    }

    fn set_pos_of<T>(&mut self) -> Result<()> {
        self.write_padding_of::<T>()
            .and_then(|_| self.add_pos((std::mem::size_of::<T>()) as u64))
    }

    fn write_padding_of<T>(&mut self) -> Result<()> {
        let alignment = std::mem::size_of::<T>();
        let padding = [0; 8];
        self.pos %= 8;
        match (self.pos as usize) % alignment {
            0 => Ok(()),
            n @ 1...7 => {
                let amt = alignment - n;
                self.pos += amt as u64;
                self.writer
                    .write_all(&padding[..amt])
                    .map_err(Into::into)
            }
            _ => unreachable!(),
        }
    }

    fn write_usize_as_u32(&mut self, v: usize) -> Result<()> {
        if v > std::u32::MAX as usize {
            return Err(Box::new(ErrorKind::NumberOutOfRange));
        }

        ser::Serializer::serialize_u32(self, v as u32)
    }
}

impl<'a, W, C> ser::Serializer for &'a mut Serializer<W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W, C>;
    type SerializeTuple = Compound<'a, W, C>;
    type SerializeTupleStruct = Compound<'a, W, C>;
    type SerializeTupleVariant = Compound<'a, W, C>;
    type SerializeMap = Compound<'a, W, C>;
    type SerializeStruct = Compound<'a, W, C>;
    type SerializeStructVariant = Compound<'a, W, C>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.set_pos_of::<bool>()
            .and_then(|_| {
                          self.writer
                              .write_u8(if v { 1 } else { 0 })
                              .map_err(Into::into)
                      })
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.set_pos_of::<u8>()
            .and_then(|_| self.writer.write_u8(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.set_pos_of::<u16>()
            .and_then(|_| self.writer.write_u16::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.set_pos_of::<u32>()
            .and_then(|_| self.writer.write_u32::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.set_pos_of::<u64>()
            .and_then(|_| self.writer.write_u64::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.set_pos_of::<i8>()
            .and_then(|_| self.writer.write_i8(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.set_pos_of::<i16>()
            .and_then(|_| self.writer.write_i16::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.set_pos_of::<i32>()
            .and_then(|_| self.writer.write_i32::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.set_pos_of::<i64>()
            .and_then(|_| self.writer.write_i64::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.set_pos_of::<f32>()
            .and_then(|_| self.writer.write_f32::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.set_pos_of::<f64>()
            .and_then(|_| self.writer.write_f64::<C::E>(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0u8; 4];
        v.encode_utf8(&mut buf);
        let width = v.len_utf8();
        self.add_pos(width as u64)
            .and_then(|_| self.writer.write_all(&buf[..width]).map_err(Into::into))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let l = v.len();
        self.write_usize_as_u32(l)
            .and_then(|_| self.add_pos(l as u64))
            .and_then(|_| self.writer.write_all(v.as_bytes()).map_err(Into::into))
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let l = v.len();
        self.write_usize_as_u32(l)
            .and_then(|_| self.add_pos(l as u64))
            .and_then(|_| self.writer.write_all(v).map_err(Into::into))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<Self::Ok>
        where T: ser::Serialize
    {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              variant_index: u32,
                              _variant: &'static str)
                              -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self,
                                           _name: &'static str,
                                           value: &T)
                                           -> Result<Self::Ok>
        where T: ser::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            variant_index: u32,
                                            _variant: &'static str,
                                            value: &T)
                                            -> Result<Self::Ok>
        where T: ser::Serialize
    {
        self.serialize_u32(variant_index)
            .and_then(|_| value.serialize(self))
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(ErrorKind::SequenceMustHaveLength)?;
        self.write_usize_as_u32(len)?;
        Ok(Compound { ser: self })
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound { ser: self })
    }

    #[inline]
    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Ok(Compound { ser: self })
    }

    #[inline]
    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        self.serialize_u32(variant_index)?;
        Ok(Compound { ser: self })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    #[inline]
    fn serialize_struct(self,
                        _name: &'static str,
                        _len: usize)
                        -> Result<Self::SerializeStruct> {
        Ok(Compound { ser: self })
    }

    #[inline]
    fn serialize_struct_variant(self,
                                _name: &'static str,
                                variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        self.serialize_u32(variant_index)?;
        Ok(Compound { ser: self })
    }
}

pub struct Compound<'a, W: 'a, C: 'a> {
    ser: &'a mut Serializer<W, C>,
}

impl<'a, W, C> ser::SerializeSeq for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeTuple for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeTupleStruct for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeTupleVariant for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeMap for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
        where T: ser::Serialize
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeStruct for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, C> ser::SerializeStructVariant for Compound<'a, W, C>
    where W: Write,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub fn serialize<T: ?Sized, S, C>(value: &T, size_limit: S) -> Result<Vec<u8>>
    where T: ser::Serialize,
          S: SizeLimit,
          C: Encapsulation
{
    let mut writer = match size_limit.limit() {
        Some(limit) => {
            let actual_size = calc_serialized_size_bounded(value, limit)?;
            Vec::with_capacity(actual_size as usize)
        }
        None => {
            let size = calc_serialized_size(value) as usize;
            Vec::with_capacity(size)
        }
    };

    serialize_into::<_, _, _, C>(&mut writer, value, Infinite)?;
    Ok(writer)
}

pub fn serialize_into<W: ?Sized, T: ?Sized, S, C>(writer: &mut W,
                                                  value: &T,
                                                  size_limit: S)
                                                  -> Result<()>
    where W: Write,
          T: ser::Serialize,
          S: SizeLimit,
          C: Encapsulation
{
    if let Some(limit) = size_limit.limit() {
        calc_serialized_size_bounded(value, limit)?;
    }

    let mut serializer = Serializer::<_, C>::new(writer);

    ser::Serialize::serialize(&C::id(), &mut serializer)
        .and_then(|_| ser::Serialize::serialize(&C::option(), &mut serializer))
        .and_then(|_| serializer.reset_pos())
        .and_then(|_| ser::Serialize::serialize(value, &mut serializer))
}
