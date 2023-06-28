//! Measuring the size of (de)serialized data.

use serde::ser;

use crate::error::{Error, Result};

struct SizeChecker {
    total: u64,
    limit: Option<u64>,
    pos: usize,
}

impl SizeChecker {
    fn new(limit: Option<u64>) -> Self {
        Self {
            total: 0,
            limit,
            pos: 0,
        }
    }
    fn add_padding_of<T>(&mut self) -> Result<()> {
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        match self.pos & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.add_size(amt as u64)
            }
            _ => unreachable!(),
        }
    }

    fn add_size(&mut self, size: u64) -> Result<()> {
        self.pos += size as usize;
        if let Some(limit) = self.limit {
            if self.total + size > limit {
                return Err(Error::SizeLimit);
            }
        }

        self.total += size;

        Ok(())
    }

    fn add_usize_as_u32(&mut self, v: usize) -> Result<()> {
        if v > std::u32::MAX as usize {
            return Err(Error::NumberOutOfRange);
        }

        ser::Serializer::serialize_u32(self, v as u32)
    }

    fn add_value<T>(&mut self, _v: T) -> Result<()> {
        self.add_padding_of::<T>()?;
        self.add_size(std::mem::size_of::<T>() as u64)
    }
}

macro_rules! impl_serialize_value {
    ($ser_method:ident($ty:ty)) => {
        fn $ser_method(self, v: $ty) -> Result<Self::Ok> {
            self.add_value(v)
        }
    };
}

impl<'a> ser::Serializer for &'a mut SizeChecker {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SizeCompound<'a>;
    type SerializeTuple = SizeCompound<'a>;
    type SerializeTupleStruct = SizeCompound<'a>;
    type SerializeTupleVariant = SizeCompound<'a>;
    type SerializeMap = SizeCompound<'a>;
    type SerializeStruct = SizeCompound<'a>;
    type SerializeStructVariant = SizeCompound<'a>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        self.add_value(0u8)
    }

    impl_serialize_value! { serialize_i8(i8) }
    impl_serialize_value! { serialize_i16(i16) }
    impl_serialize_value! { serialize_i32(i32) }
    impl_serialize_value! { serialize_i64(i64) }

    impl_serialize_value! { serialize_u8(u8) }
    impl_serialize_value! { serialize_u16(u16) }
    impl_serialize_value! { serialize_u32(u32) }
    impl_serialize_value! { serialize_u64(u64) }

    impl_serialize_value! { serialize_f32(f32) }
    impl_serialize_value! { serialize_f64(f64) }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        self.add_size(1)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.add_value(0_u32)?;
        self.add_size(v.len() as u64 + 1) // adds the length 1 of a terminating character
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.add_value(0_u32)?;
        self.add_size(v.len() as u64)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::TypeNotSupported)
    }

    fn serialize_some<T>(self, _v: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize + ?Sized,
    {
        Err(Error::TypeNotSupported)
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

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize + ?Sized,
    {
        self.serialize_u32(variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(Error::SequenceMustHaveLength)?;
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
        Err(Error::TypeNotSupported)
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
pub struct SizeCompound<'a> {
    ser: &'a mut SizeChecker,
}

impl<'a> ser::SerializeSeq for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        key.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for SizeCompound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ser::Serialize + ?Sized,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Returns the size that an object would be if serialized.
pub fn calc_serialized_data_size<T>(value: &T) -> u64
where
    T: ser::Serialize + ?Sized,
{
    let mut checker = SizeChecker::new(None);

    value.serialize(&mut checker).ok();
    checker.total
}

/// Given a maximum size limit, check how large an object would be if it were
/// to be serialized.
pub fn calc_serialized_data_size_bounded<T>(value: &T, max: u64) -> Result<u64>
where
    T: ser::Serialize + ?Sized,
{
    let mut checker = SizeChecker::new(Some(max));

    value.serialize(&mut checker)?;
    Ok(max - checker.total)
}
