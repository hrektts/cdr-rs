use std;

use serde::ser;
use super::error::{Error, ErrorKind, Result};

pub trait SizeLimit {
    fn add(&mut self, n: u64) -> Result<()>;
    fn limit(&self) -> Option<u64>;
}

pub struct Bounded(pub u64);

impl SizeLimit for Bounded {
    #[inline]
    fn add(&mut self, n: u64) -> Result<()> {
        use super::encapsulation::ENCAPSULATION_HEADER_SIZE;

        if self.0 >= (n + ENCAPSULATION_HEADER_SIZE) {
            self.0 -= n;
            Ok(())
        } else {
            Err(Box::new(ErrorKind::SizeLimit))
        }
    }

    #[inline]
    fn limit(&self) -> Option<u64> {
        Some(self.0)
    }
}

pub struct Infinite;

impl SizeLimit for Infinite {
    #[inline]
    fn add(&mut self, _n: u64) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn limit(&self) -> Option<u64> {
        None
    }
}

struct Counter {
    total: u64,
    limit: Option<u64>,
}

impl SizeLimit for Counter {
    fn add(&mut self, n: u64) -> Result<()> {
        self.total += n;
        if let Some(limit) = self.limit {
            if self.total > limit {
                return Err(Box::new(ErrorKind::SizeLimit));
            }
        }
        Ok(())
    }

    fn limit(&self) -> Option<u64> {
        unreachable!();
    }
}

pub struct SizeChecker<S> {
    counter: S,
    pos: usize,
}

impl<S> SizeChecker<S>
where
    S: SizeLimit,
{
    pub fn new(counter: S) -> SizeChecker<S> {
        SizeChecker {
            counter: counter,
            pos: 0,
        }
    }

    fn add_padding_of<T>(&mut self) -> Result<()> {
        let alignment = std::mem::size_of::<T>();
        self.pos %= 8;
        match self.pos % alignment {
            0 => Ok(()),
            n @ 1...7 => {
                let amt = alignment - n;
                self.add_size(amt as u64)
            }
            _ => unreachable!(),
        }
    }

    fn add_size(&mut self, size: u64) -> Result<()> {
        self.pos += size as usize;
        self.counter.add(size)
    }

    fn add_usize_as_u32(&mut self, v: usize) -> Result<()> {
        if v > std::u32::MAX as usize {
            return Err(Box::new(ErrorKind::NumberOutOfRange));
        }

        ser::Serializer::serialize_u32(self, v as u32)
    }

    fn add_value<T>(&mut self, _v: T) -> Result<()> {
        self.add_padding_of::<T>()?;
        self.add_size(std::mem::size_of::<T>() as u64)
    }
}

impl<'a, S> ser::Serializer for &'a mut SizeChecker<S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SizeCompound<'a, S>;
    type SerializeTuple = SizeCompound<'a, S>;
    type SerializeTupleStruct = SizeCompound<'a, S>;
    type SerializeTupleVariant = SizeCompound<'a, S>;
    type SerializeMap = SizeCompound<'a, S>;
    type SerializeStruct = SizeCompound<'a, S>;
    type SerializeStructVariant = SizeCompound<'a, S>;

    #[inline]
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        self.add_value(0 as u8)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.add_value(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.add_size(v.len_utf8() as u64)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.add_value(0 as u32)?;
        self.add_size(v.len() as u64)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.add_value(0 as u32)?;
        self.add_size(v.len() as u64)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok> {
        self.add_value(0 as u8)
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        self.add_value(1 as u8)?;
        v.serialize(self)
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
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        self.serialize_u32(variant_index).and_then(
            |_| value.serialize(self),
        )
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(ErrorKind::SequenceMustHaveLength)?;
        self.add_usize_as_u32(len)?;
        Ok(SizeCompound { ser: self })
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SizeCompound { ser: self })
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SizeCompound { ser: self })
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_u32(variant_index)?;
        Ok(SizeCompound { ser: self })
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Box::new(ErrorKind::Message("type not supported".into())))
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SizeCompound { ser: self })
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_u32(variant_index)?;
        Ok(SizeCompound { ser: self })
    }
}

pub struct SizeCompound<'a, S: 'a> {
    ser: &'a mut SizeChecker<S>,
}

impl<'a, S> ser::SerializeSeq for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeTuple for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeTupleStruct for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeTupleVariant for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeMap for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeStruct for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, S> ser::SerializeStructVariant for SizeCompound<'a, S>
where
    S: SizeLimit,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub fn calc_serialized_size<T: ?Sized>(value: &T) -> u64
where
    T: ser::Serialize,
{
    use super::encapsulation::ENCAPSULATION_HEADER_SIZE;

    let mut checker = SizeChecker {
        counter: Counter {
            total: ENCAPSULATION_HEADER_SIZE,
            limit: None,
        },
        pos: 0,
    };

    value.serialize(&mut checker).ok();
    checker.counter.total
}

pub fn calc_serialized_size_bounded<T: ?Sized>(value: &T, max: u64) -> Result<u64>
where
    T: ser::Serialize,
{
    use super::encapsulation::ENCAPSULATION_HEADER_SIZE;

    if max < ENCAPSULATION_HEADER_SIZE {
        Err(Box::new(ErrorKind::SizeLimit))
    } else {
        let mut checker = SizeChecker {
            counter: Bounded(max - ENCAPSULATION_HEADER_SIZE),
            pos: 0,
        };

        match value.serialize(&mut checker) {
            Ok(_) => Ok(max - checker.counter.0),
            Err(e) => Err(e),
        }
    }
}
