//! Measuring the size of (de)serialized data.
//! Measuring the size of (de)serialized data.

use std;

use serde::ser;

use crate::error::{Error, ErrorKind, Result};

/// Limits on the number of bytes that can be read or written.
pub trait SizeLimit {
    fn add(&mut self, n: u64) -> Result<()>;
    fn limit(&self) -> Option<u64>;
}

/// A `SizeLimit` that restricts serialized or deserialized messages so that
/// they do not exceed a certain byte length.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Bounded(pub u64);

impl SizeLimit for Bounded {
    #[inline]
    fn add(&mut self, n: u64) -> Result<()> {
        if self.0 >= n {
            self.0 -= n;
            Ok(())
        } else {
            Err(ErrorKind::SizeLimit.into())
        }
    }

    #[inline]
    fn limit(&self) -> Option<u64> {
        Some(self.0)
    }
}

/// A `SizeLimit` without a limit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
                return Err(ErrorKind::SizeLimit.into());
            }
        }
        Ok(())
    }

    fn limit(&self) -> Option<u64> {
        unreachable!();
    }
}

struct SizeChecker<S> {
    counter: S,
    pos: usize,
}

macro_rules! add_value  {
    ($own:expr, $T:ty) => {{
        add_padding_of!($own, $T)?;
        // Compiler-hint return type
        let result: Result<()> = $own.add_size(std::mem::size_of::<$T>() as u64);
        result
    }};
}

macro_rules! add_padding_of {
    ($own:expr, $T:ty) => {{
        const ALIGNMENT: usize = std::mem::size_of::<$T>();
        const REM_MASK: usize = ALIGNMENT - 1; // mask like 0x0, 0x1, 0x3, 0x7
        match ($own.pos as usize) & REM_MASK {
            0 => Ok(()),
            n @ 1...7 => {
                let amt = ALIGNMENT - n;
                // Compiler-hint return type
                let result: Result<()> = $own.add_size(amt as u64);
                result
            }
            _ => unreachable!(),
        }
    }};
}

impl<S> SizeChecker<S>
where
    S: SizeLimit,
{
    pub fn new(counter: S) -> SizeChecker<S> {
        SizeChecker { counter, pos: 0 }
    }

    fn add_size(&mut self, size: u64) -> Result<()> {
        self.pos += size as usize;
        self.counter.add(size)
    }

    fn add_usize_as_u32(&mut self, v: usize) -> Result<()> {
        if v > std::u32::MAX as usize {
            return Err(ErrorKind::NumberOutOfRange.into());
        }

        ser::Serializer::serialize_u32(self, v as u32)
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

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        add_value!(self, u8)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        add_value!(self, u8)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        add_value!(self, u16)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        add_value!(self, u32)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        add_value!(self, u64)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        add_value!(self, i8)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        add_value!(self, i16)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        add_value!(self, i32)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        add_value!(self, i64)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        add_value!(self, f32)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        add_value!(self, f64)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        add_value!(self, u8)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        add_value!(self, u32)?;
        self.add_size(v.len() as u64 + 1) // adds the length 1 of a terminating character
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        add_value!(self, u32)?;
        self.add_size(v.len() as u64)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        //none
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        add_value!(self, u8)?;
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_u32(variant_index)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

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
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(ErrorKind::SequenceMustHaveLength)?;
        self.add_usize_as_u32(len)?;
        Ok(SizeCompound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SizeCompound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(SizeCompound { ser: self })
    }

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

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(ErrorKind::TypeNotSupported.into())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SizeCompound { ser: self })
    }

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

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[doc(hidden)]
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

/// Returns the size that an object would be if serialized.
pub fn calc_serialized_data_size<T: ?Sized>(value: &T) -> u64
where
    T: ser::Serialize,
{
    let mut checker = SizeChecker {
        counter: Counter {
            total: 0,
            limit: None,
        },
        pos: 0,
    };

    value.serialize(&mut checker).ok();
    checker.counter.total
}

/// Given a maximum size limit, check how large an object would be if it were
/// to be serialized.
pub fn calc_serialized_data_size_bounded<T: ?Sized>(value: &T, max: u64) -> Result<u64>
where
    T: ser::Serialize,
{
    let mut checker = SizeChecker {
        counter: Bounded(max),
        pos: 0,
    };

    match value.serialize(&mut checker) {
        Ok(_) => Ok(max - checker.counter.0),
        Err(e) => Err(e),
    }
}
