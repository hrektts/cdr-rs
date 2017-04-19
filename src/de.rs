use std::cmp;
use std::io::Read;
use std::marker::PhantomData;
use std::mem;
use std::str;

use byteorder::{ByteOrder, ReadBytesExt};
use serde::de;
use serde::de::value::ValueDeserializer;
use super::encapsulation::{CdrBe, CdrLe, Encapsulation, PlCdrBe, PlCdrLe};
use super::error::{Error, ErrorKind, Result};
use super::size::{Infinite, SizeLimit};

const BLOCK_SIZE: usize = 65_536;

pub struct Deserializer<R, S, C> {
    reader: R,
    size_limit: S,
    pos: u64,
    phantom: PhantomData<C>,
}

impl<R, S, C> Deserializer<R, S, C>
    where R: Read,
          S: SizeLimit,
          C: Encapsulation,
          C::E: ByteOrder
{
    pub fn new(reader: R, size_limit: S) -> Self {
        Self {
            reader: reader,
            size_limit: size_limit,
            pos: 0,
            phantom: PhantomData,
        }
    }

    fn read_padding_of<T>(&mut self) -> Result<()> {
        let alignment = mem::size_of::<T>();
        let mut padding = [0; 8];
        self.pos %= 8;
        match (self.pos as usize) % alignment {
            0 => Ok(()),
            n @ 1...7 => {
                let amt = alignment - n;
                self.read_size(amt as u64)
                    .and_then(|_| {
                                  self.reader
                                      .read_exact(&mut padding[..amt])
                                      .map_err(Into::into)
                              })
            }
            _ => unreachable!(),
        }
    }

    fn read_size(&mut self, size: u64) -> Result<()> {
        self.pos += size;
        self.size_limit.add(size)
    }

    fn read_size_of<T>(&mut self) -> Result<()> {
        self.read_size(mem::size_of::<T>() as u64)
    }

    fn read_string(&mut self) -> Result<String> {
        String::from_utf8(self.read_vec()?).map_err(|_| {
            ErrorKind::Message("error while decodint utf8 string".to_string()).into()
        })
    }

    fn read_vec(&mut self) -> Result<Vec<u8>> {
        de::Deserialize::deserialize(&mut *self).and_then(|mut len: u32| {
            let mut result = Vec::new();
            let mut offset = 0;
            while len > 0 {
                let reserve = cmp::min(len as usize, BLOCK_SIZE);
                self.read_size(reserve as u64)
                    .and_then(|_| Ok(result.resize(offset + reserve, 0)))
                    .and_then(|_| {
                                  self.reader
                                      .read_exact(&mut result[offset..])
                                      .map_err(Into::into)
                              })
                    .and_then(|_| {
                                  len -= reserve as u32;
                                  offset += reserve;
                                  Ok(())
                              })?
            }
            Ok(result)
        })
    }

    fn reset_pos(&mut self) -> Result<()> {
        self.pos = 0;
        Ok(())
    }
}

impl<'a, R, S, C> de::Deserializer for &'a mut Deserializer<R, S, C>
    where R: Read,
          S: SizeLimit,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Error = Error;

    #[inline]
    fn deserialize<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(Box::new(ErrorKind::Message("not supported".into())))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserialize::deserialize(self).and_then(|value: u8| match value {
                                                        1 => visitor.visit_bool(true),
                                                        0 => visitor.visit_bool(false),
                                                        _ => {
                Err(Box::new(ErrorKind::Message("invalid u8 when decoding bool".into())))
            }
                                                    })
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_size_of::<u8>()
            .and_then(|_| visitor.visit_u8(self.reader.read_u8()?))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<u16>()
            .and_then(|_| self.read_size_of::<u16>())
            .and_then(|_| visitor.visit_u16(self.reader.read_u16::<C::E>()?))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<u32>()
            .and_then(|_| self.read_size_of::<u32>())
            .and_then(|_| visitor.visit_u32(self.reader.read_u32::<C::E>()?))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<u64>()
            .and_then(|_| self.read_size_of::<u64>())
            .and_then(|_| visitor.visit_u64(self.reader.read_u64::<C::E>()?))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_size_of::<i8>()
            .and_then(|_| visitor.visit_i8(self.reader.read_i8()?))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<i16>()
            .and_then(|_| self.read_size_of::<i16>())
            .and_then(|_| visitor.visit_i16(self.reader.read_i16::<C::E>()?))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<i32>()
            .and_then(|_| self.read_size_of::<i32>())
            .and_then(|_| visitor.visit_i32(self.reader.read_i32::<C::E>()?))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<i64>()
            .and_then(|_| self.read_size_of::<i64>())
            .and_then(|_| visitor.visit_i64(self.reader.read_i64::<C::E>()?))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<f32>()
            .and_then(|_| self.read_size_of::<f32>())
            .and_then(|_| visitor.visit_f32(self.reader.read_f32::<C::E>()?))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        self.read_padding_of::<f64>()
            .and_then(|_| self.read_size_of::<f64>())
            .and_then(|_| visitor.visit_f64(self.reader.read_f64::<C::E>()?))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        let mut buf = [0u8; 4];
        self.reader
            .read_exact(&mut buf[..1])
            .map_err(Into::into)
            .and_then(|_| {
                let width = utf8_char_width(buf[0]);
                if 1 <= width {
                    self.reader
                        .read_exact(&mut buf[1..width])
                        .map(|_| width)
                        .map_err(Into::into)
                } else {
                    Err(Box::new(ErrorKind::Message("invalid char encoding".into())))
                }
            })
            .and_then(|width| {
                self.read_size(width as u64)?;
                let c =
                    str::from_utf8(&buf[..width])
                        .ok()
                        .and_then(|s| s.chars().next())
                        .ok_or(Box::new(ErrorKind::Message("invalid char encoding"
                                                               .into())))?;
                visitor.visit_char(c)
            })
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_str(&self.read_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_string(self.read_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_bytes(&self.read_vec()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_byte_buf(self.read_vec()?)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self,
                                  _name: &'static str,
                                  visitor: V)
                                  -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self,
                                     _name: &'static str,
                                     visitor: V)
                                     -> Result<V::Value>
        where V: de::Visitor
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserialize::deserialize(&mut *self)
            .and_then(|len: u32| self.deserialize_seq_fixed_size(len as usize, visitor))
    }

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        struct SeqVisitor<'a, R: Read + 'a, S: SizeLimit + 'a, C: Encapsulation + 'a> {
            deserializer: &'a mut Deserializer<R, S, C>,
            len: usize,
        }

        impl<'a, R: 'a, S, C> de::SeqVisitor for SeqVisitor<'a, R, S, C>
            where R: Read,
                  S: SizeLimit,
                  C: Encapsulation,
                  C::E: ByteOrder
        {
            type Error = Error;

            fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
                where T: de::DeserializeSeed
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
        }

        visitor.visit_seq(SeqVisitor {
                              deserializer: self,
                              len: len,
                          })
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        struct TupleVisitor<'a, R: 'a, S: 'a, C: 'a>(&'a mut Deserializer<R, S, C>)
            where R: Read,
                  S: SizeLimit,
                  C: Encapsulation,
                  C::E: ByteOrder;

        impl<'a, R: 'a, S, C> de::SeqVisitor for TupleVisitor<'a, R, S, C>
            where R: Read,
                  S: SizeLimit,
                  C: Encapsulation,
                  C::E: ByteOrder
        {
            type Error = Error;

            fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
                where T: de::DeserializeSeed
            {
                let value = de::DeserializeSeed::deserialize(seed, &mut *self.0)?;
                Ok(Some(value))
            }
        }

        visitor.visit_seq(TupleVisitor(self))
    }

    fn deserialize_tuple_struct<V>(self,
                                   _name: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    fn deserialize_struct<V>(self,
                             _name: &'static str,
                             fields: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: de::Visitor
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_struct_field<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }

    fn deserialize_enum<V>(self,
                           _name: &'static str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value>
        where V: de::Visitor
    {
        impl<'a, R: 'a, S, C> de::EnumVisitor for &'a mut Deserializer<R, S, C>
            where R: Read,
                  S: SizeLimit,
                  C: Encapsulation,
                  C::E: ByteOrder
        {
            type Error = Error;
            type Variant = Self;

            fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
                where V: de::DeserializeSeed
            {
                let idx: u32 = de::Deserialize::deserialize(&mut *self)?;
                let val: Result<_> = seed.deserialize(idx.into_deserializer());
                Ok((try!(val), self))
            }
        }

        visitor.visit_enum(self)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        Err(Box::new(ErrorKind::TypeNotSupported))
    }
}

impl<'a, R, S, C> de::VariantVisitor for &'a mut Deserializer<R, S, C>
    where R: Read,
          S: SizeLimit,
          C: Encapsulation,
          C::E: ByteOrder
{
    type Error = Error;

    fn visit_unit(self) -> Result<()> {
        Ok(())
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value>
        where T: de::DeserializeSeed
    {
        de::DeserializeSeed::deserialize(seed, self)
    }

    fn visit_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn visit_struct<V>(self,
                       fields: &'static [&'static str],
                       visitor: V)
                       -> Result<V::Value>
        where V: de::Visitor
    {
        de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}

impl<R, S> From<Deserializer<R, S, CdrBe>> for Deserializer<R, S, CdrLe> {
    fn from(t: Deserializer<R, S, CdrBe>) -> Self {
        Deserializer::<R, S, CdrLe> {
            reader: t.reader,
            size_limit: t.size_limit,
            pos: t.pos,
            phantom: PhantomData,
        }
    }
}

impl<R, S> From<Deserializer<R, S, CdrBe>> for Deserializer<R, S, PlCdrBe> {
    fn from(t: Deserializer<R, S, CdrBe>) -> Self {
        Deserializer::<R, S, PlCdrBe> {
            reader: t.reader,
            size_limit: t.size_limit,
            pos: t.pos,
            phantom: PhantomData,
        }
    }
}

impl<R, S> From<Deserializer<R, S, CdrBe>> for Deserializer<R, S, PlCdrLe> {
    fn from(t: Deserializer<R, S, CdrBe>) -> Self {
        Deserializer::<R, S, PlCdrLe> {
            reader: t.reader,
            size_limit: t.size_limit,
            pos: t.pos,
            phantom: PhantomData,
        }
    }
}

#[inline]
fn utf8_char_width(first_byte: u8) -> usize {
    UTF8_CHAR_WIDTH[first_byte as usize] as usize
}

// https://tools.ietf.org/html/rfc3629
const UTF8_CHAR_WIDTH: &'static [u8; 256] = &[
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1,
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1, // 0x1F
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1,
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1, // 0x3F
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1,
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1, // 0x5F
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1,
    1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1, // 0x7F
    0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
    0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // 0x9F
    0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0,
    0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0, // 0xBF
    0,0,2,2, 2,2,2,2, 2,2,2,2, 2,2,2,2,
    2,2,2,2, 2,2,2,2, 2,2,2,2, 2,2,2,2, // 0xDF
    3,3,3,3, 3,3,3,3, 3,3,3,3, 3,3,3,3, // 0xEF
    4,4,4,4, 0,0,0,0, 0,0,0,0, 0,0,0,0, // 0xFF
];

pub fn deserialize<T>(bytes: &[u8]) -> Result<T>
    where T: de::Deserialize
{
    let mut reader = bytes;
    deserialize_from::<_, _, _>(&mut reader, Infinite)
}

pub fn deserialize_from<R, T, S>(reader: &mut R, size_limit: S) -> Result<T>
    where R: Read,
          T: de::Deserialize,
          S: SizeLimit
{
    use super::encapsulation::ENCAPSULATION_HEADER_SIZE;

    let mut deserializer = Deserializer::<_, S, CdrBe>::new(reader, size_limit);

    let v: [u8; ENCAPSULATION_HEADER_SIZE as usize] =
        de::Deserialize::deserialize(&mut deserializer)?;
    deserializer.reset_pos()?;
    match v[1] {
        0 => de::Deserialize::deserialize(&mut deserializer),
        1 => de::Deserialize::deserialize(
            &mut Into::<Deserializer<_, _, CdrLe>>::into(deserializer)),
        2 => de::Deserialize::deserialize(
            &mut Into::<Deserializer<_, _, PlCdrBe>>::into(deserializer)),
        3 => de::Deserialize::deserialize(
            &mut Into::<Deserializer<_, _, PlCdrLe>>::into(deserializer)),
        _ => Err(Box::new(ErrorKind::Message("unknown encapsulation".into()))),
    }
}
