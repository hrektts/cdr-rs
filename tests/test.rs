#[macro_use]
extern crate serde_derive;

extern crate cdr;
extern crate serde;

use std::{u8, u16, i16, u32, i32, u64, i64, f32, f64};
use std::fmt::Debug;
use std::io::Cursor;

use cdr::{Bounded, CdrBe, CdrLe, ErrorKind, Infinite, PlCdrBe, PlCdrLe, Result};

const ENCAPSULATION_HEADER_SIZE: u64 = 4;

fn check<T>(element: T, maybe_size: Option<u64>)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    if let Some(size) = maybe_size {
        assert!(size >= ENCAPSULATION_HEADER_SIZE);
    }

    check_serialized_size(&element, maybe_size);
    check_round_trip(&element, maybe_size);
    check_capacity_shortage(&element, maybe_size);
    check_size_limit(&element, maybe_size);
}

fn check_serialized_size<T>(element: &T, maybe_size: Option<u64>)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    if let Some(serialized_size) = maybe_size {
        let size = cdr::calc_serialized_size(&element);
        assert_eq!(serialized_size as u64, size);
    }
}

fn check_round_trip<T>(element: &T, maybe_size: Option<u64>)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    let size = match maybe_size {
        Some(v) => v as u64,
        None => cdr::calc_serialized_size(&element),
    };
    {
        let encoded = cdr::serialize::<_, _, CdrBe>(element, Infinite).unwrap();
        let decoded = cdr::deserialize::<T>(&encoded).unwrap();

        assert_eq!(*element, decoded);
        assert_eq!(size, encoded.len() as u64);
    }
    {
        let encoded = cdr::serialize::<_, _, CdrLe>(element, Infinite).unwrap();
        let decoded = cdr::deserialize::<T>(&encoded).unwrap();

        assert_eq!(*element, decoded);
        assert_eq!(size, encoded.len() as u64);
    }
    {
        let encoded = cdr::serialize::<_, _, PlCdrBe>(element, Infinite).unwrap();
        let decoded = cdr::deserialize::<T>(&encoded).unwrap();

        assert_eq!(*element, decoded);
        assert_eq!(size, encoded.len() as u64);
    }
    {
        let encoded = cdr::serialize::<_, _, PlCdrLe>(element, Infinite).unwrap();
        let decoded = cdr::deserialize::<T>(&encoded).unwrap();

        assert_eq!(*element, decoded);
        assert_eq!(size, encoded.len() as u64);
    }
}

fn check_capacity_shortage<T>(element: &T, maybe_size: Option<u64>)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    let bound = calc_invalid_size(element, maybe_size);
    let mut buf = [0u8; 2000];
    let mut buf = Cursor::new(&mut buf[0..bound as usize]);

    assert!(cdr::serialize_into::<_, _, _, CdrBe>(&mut buf, &element, Infinite).is_err());
    assert!(cdr::serialize_into::<_, _, _, CdrLe>(&mut buf, &element, Infinite).is_err());
    assert!(cdr::serialize_into::<_, _, _, PlCdrBe>(&mut buf, &element, Infinite)
                .is_err());
    assert!(cdr::serialize_into::<_, _, _, PlCdrLe>(&mut buf, &element, Infinite)
                .is_err());
}

fn check_size_limit<T>(element: &T, maybe_size: Option<u64>)
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    let bound = calc_invalid_size(element, maybe_size);

    assert!(cdr::serialize::<_, _, CdrBe>(&element, Bounded(bound)).is_err());
    assert!(cdr::serialize::<_, _, CdrLe>(&element, Bounded(bound)).is_err());
    assert!(cdr::serialize::<_, _, PlCdrBe>(&element, Bounded(bound)).is_err());
    assert!(cdr::serialize::<_, _, PlCdrLe>(&element, Bounded(bound)).is_err());
    {
        let encoded = cdr::serialize::<_, _, CdrBe>(&element, Infinite).unwrap();
        let mut encoded = encoded.as_slice();
        assert!(cdr::deserialize_from::<_, T, _>(&mut encoded, Bounded(bound)).is_err());
    }
    {
        let encoded = cdr::serialize::<_, _, CdrLe>(&element, Infinite).unwrap();
        let mut encoded = encoded.as_slice();
        assert!(cdr::deserialize_from::<_, T, _>(&mut encoded, Bounded(bound)).is_err());
    }
    {
        let encoded = cdr::serialize::<_, _, PlCdrBe>(&element, Infinite).unwrap();
        let mut encoded = encoded.as_slice();
        assert!(cdr::deserialize_from::<_, T, _>(&mut encoded, Bounded(bound)).is_err());
    }
    {
        let encoded = cdr::serialize::<_, _, PlCdrLe>(&element, Infinite).unwrap();
        let mut encoded = encoded.as_slice();
        assert!(cdr::deserialize_from::<_, T, _>(&mut encoded, Bounded(bound)).is_err());
    }
}

fn calc_invalid_size<T>(element: &T, maybe_size: Option<u64>) -> u64
    where T: serde::Serialize + serde::Deserialize + PartialEq + Debug
{
    match maybe_size {
        Some(v) if v > 0 => v - 1,
        _ => cdr::calc_serialized_size(&element) - 1,
    }
}

#[test]
fn test_octet() {
    check(u8::MIN, Some(4 + 1));
    check(u8::MAX, Some(4 + 1));
}

#[test]
fn test_char() {
    check('a', Some(4 + 1));
    check('Z', Some(4 + 1));
}

#[test]
fn test_unsigned_short() {
    check(u16::MIN, Some(4 + 2));
    check(u16::MAX, Some(4 + 2));
}

#[test]
fn test_short() {
    check(i16::MIN, Some(4 + 2));
    check(i16::MAX, Some(4 + 2));
}

#[test]
fn test_unsigned_long() {
    check(u32::MIN, Some(4 + 4));
    check(u32::MAX, Some(4 + 4));
}

#[test]
fn test_long() {
    check(i32::MIN, Some(4 + 4));
    check(i32::MAX, Some(4 + 4));
}

#[test]
fn test_unsigned_long_long() {
    check(u64::MIN, Some(4 + 8));
    check(u64::MAX, Some(4 + 8));
}

#[test]
fn test_long_long() {
    check(i64::MIN, Some(4 + 8));
    check(i64::MAX, Some(4 + 8));
}

#[test]
fn test_float() {
    check(f32::MIN, Some(4 + 4));
    check(f32::MAX, Some(4 + 4));
}

#[test]
fn test_double() {
    check(f64::MIN, Some(4 + 8));
    check(f64::MAX, Some(4 + 8));
}

#[test]
fn test_bool() {
    check(false, Some(4 + 1));
    check(true, Some(4 + 1));
}

#[test]
fn test_string() {
    check("".to_string(), Some(4 + 4));
    check("a".to_string(), Some(4 + 5));
}

#[test]
fn test_unsigned_short_alignment() {
    check(('a', 1u16), Some(4 + 1 + 1 + 2));
    check((1u8, 1u16), Some(4 + 1 + 1 + 2));
    check((1i8, 1u16), Some(4 + 1 + 1 + 2));
    check((1u16, 1u16), Some(4 + 2 + 2));
    check((1i16, 1u16), Some(4 + 2 + 2));
    check((1u32, 1u16), Some(4 + 4 + 2));
    check((1i32, 1u16), Some(4 + 4 + 2));
    check((1f32, 1u16), Some(4 + 4 + 2));
    check((1f64, 1u16), Some(4 + 8 + 2));
    check((true, 1u16), Some(4 + 1 + 1 + 2));
    check(("a".to_string(), 1u16), Some(4 + 5 + 1 + 2));
}

#[test]
fn test_short_alignment() {
    check(('a', 1i16), Some(4 + 1 + 1 + 2));
    check((1u8, 1i16), Some(4 + 1 + 1 + 2));
    check((1i8, 1i16), Some(4 + 1 + 1 + 2));
    check((1u16, 1i16), Some(4 + 2 + 2));
    check((1i16, 1i16), Some(4 + 2 + 2));
    check((1u32, 1i16), Some(4 + 4 + 2));
    check((1i32, 1i16), Some(4 + 4 + 2));
    check((1f32, 1i16), Some(4 + 4 + 2));
    check((1f64, 1i16), Some(4 + 8 + 2));
    check((true, 1i16), Some(4 + 1 + 1 + 2));
    check(("a".to_string(), 1i16), Some(4 + 5 + 1 + 2));
}

#[test]
fn test_unsigned_long_alignment() {
    check(('a', 1u32), Some(4 + 1 + 3 + 4));
    check((1u8, 1u32), Some(4 + 1 + 3 + 4));
    check((1i8, 1u32), Some(4 + 1 + 3 + 4));
    check((1u16, 1u32), Some(4 + 2 + 2 + 4));
    check((1i16, 1u32), Some(4 + 2 + 2 + 4));
    check((1u32, 1u32), Some(4 + 4 + 4));
    check((1i32, 1u32), Some(4 + 4 + 4));
    check((1f32, 1u32), Some(4 + 4 + 4));
    check((1f64, 1u32), Some(4 + 8 + 4));
    check((true, 1u32), Some(4 + 1 + 3 + 4));
    check(("a".to_string(), 1u32), Some(4 + 5 + 3 + 4));
}

#[test]
fn test_long_alignment() {
    check(('a', 1i32), Some(4 + 1 + 3 + 4));
    check((1u8, 1i32), Some(4 + 1 + 3 + 4));
    check((1i8, 1i32), Some(4 + 1 + 3 + 4));
    check((1u16, 1i32), Some(4 + 2 + 2 + 4));
    check((1i16, 1i32), Some(4 + 2 + 2 + 4));
    check((1u32, 1i32), Some(4 + 4 + 4));
    check((1i32, 1i32), Some(4 + 4 + 4));
    check((1f32, 1i32), Some(4 + 4 + 4));
    check((1f64, 1i32), Some(4 + 8 + 4));
    check((true, 1i32), Some(4 + 1 + 3 + 4));
    check(("a".to_string(), 1i32), Some(4 + 5 + 3 + 4));
}

#[test]
fn test_unsigned_long_long_alignment() {
    check(('a', 1u64), Some(4 + 1 + 7 + 8));
    check((1u8, 1u64), Some(4 + 1 + 7 + 8));
    check((1i8, 1u64), Some(4 + 1 + 7 + 8));
    check((1u16, 1u64), Some(4 + 2 + 6 + 8));
    check((1i16, 1u64), Some(4 + 2 + 6 + 8));
    check((1u32, 1u64), Some(4 + 4 + 4 + 8));
    check((1i32, 1u64), Some(4 + 4 + 4 + 8));
    check((1f32, 1u64), Some(4 + 4 + 4 + 8));
    check((1f64, 1u64), Some(4 + 8 + 8));
    check((true, 1u64), Some(4 + 1 + 7 + 8));
    check(("a".to_string(), 1u64), Some(4 + 5 + 3 + 8));
}

#[test]
fn test_long_long_alignment() {
    check(('a', 1i64), Some(4 + 1 + 7 + 8));
    check((1u8, 1i64), Some(4 + 1 + 7 + 8));
    check((1i8, 1i64), Some(4 + 1 + 7 + 8));
    check((1u16, 1i64), Some(4 + 2 + 6 + 8));
    check((1i16, 1i64), Some(4 + 2 + 6 + 8));
    check((1u32, 1i64), Some(4 + 4 + 4 + 8));
    check((1i32, 1i64), Some(4 + 4 + 4 + 8));
    check((1f32, 1i64), Some(4 + 4 + 4 + 8));
    check((1f64, 1i64), Some(4 + 8 + 8));
    check((true, 1i64), Some(4 + 1 + 7 + 8));
    check(("a".to_string(), 1i64), Some(4 + 5 + 3 + 8));
}

#[test]
fn test_float_alignment() {
    check(('a', 1f32), Some(4 + 1 + 3 + 4));
    check((1u8, 1f32), Some(4 + 1 + 3 + 4));
    check((1i8, 1f32), Some(4 + 1 + 3 + 4));
    check((1u16, 1f32), Some(4 + 2 + 2 + 4));
    check((1i16, 1f32), Some(4 + 2 + 2 + 4));
    check((1u32, 1f32), Some(4 + 4 + 4));
    check((1f32, 1f32), Some(4 + 4 + 4));
    check((1f32, 1f32), Some(4 + 4 + 4));
    check((1f64, 1f32), Some(4 + 8 + 4));
    check((true, 1f32), Some(4 + 1 + 3 + 4));
    check(("a".to_string(), 1f32), Some(4 + 5 + 3 + 4));
}

#[test]
fn test_double_alignment() {
    check(('a', 1f64), Some(4 + 1 + 7 + 8));
    check((1u8, 1f64), Some(4 + 1 + 7 + 8));
    check((1i8, 1f64), Some(4 + 1 + 7 + 8));
    check((1u16, 1f64), Some(4 + 2 + 6 + 8));
    check((1i16, 1f64), Some(4 + 2 + 6 + 8));
    check((1u32, 1f64), Some(4 + 4 + 4 + 8));
    check((1i32, 1f64), Some(4 + 4 + 4 + 8));
    check((1f32, 1f64), Some(4 + 4 + 4 + 8));
    check((1f64, 1f64), Some(4 + 8 + 8));
    check((true, 1f64), Some(4 + 1 + 7 + 8));
    check(("a".to_string(), 1f64), Some(4 + 5 + 3 + 8));
}

#[test]
fn test_seq_octet() {
    check(Vec::<u8>::new(), Some(4 + 4));
    check(vec![0u8, 1, 2], Some(4 + 4 + 1 * 3));
}

#[test]
fn test_seq_char() {
    check(Vec::<char>::new(), Some(4 + 4));
    check(vec!['a', 'b', 'c'], Some(4 + 4 + 1 * 3));
}

#[test]
fn test_seq_unsigned_short() {
    check(Vec::<u16>::new(), Some(4 + 4));
    check(vec![0u16, 1, 2], Some(4 + 4 + 2 * 3));
}

#[test]
fn test_seq_short() {
    check(Vec::<i16>::new(), Some(4 + 4));
    check(vec![0i16, 1, 2], Some(4 + 4 + 2 * 3));
}

#[test]
fn test_seq_unsigned_long() {
    check(Vec::<u32>::new(), Some(4 + 4));
    check(vec![0u32, 1, 2], Some(4 + 4 + 4 * 3));
}

#[test]
fn test_seq_long() {
    check(Vec::<i32>::new(), Some(4 + 4));
    check(vec![0i32, 1, 2], Some(4 + 4 + 4 * 3));
}

#[test]
fn test_seq_unsigned_long_long() {
    check(Vec::<u64>::new(), Some(4 + 4));
    check(vec![0u64, 1, 2], Some(4 + 4 + 4 + 8 * 3));
}

#[test]
fn test_seq_long_long() {
    check(Vec::<i64>::new(), Some(4 + 4));
    check(vec![0i64, 1, 2], Some(4 + 4 + 4 + 8 * 3));
}

#[test]
fn test_seq_float() {
    check(Vec::<f32>::new(), Some(4 + 4));
    check(vec![0f32, 1., 2.], Some(4 + 4 + 4 * 3));
}

#[test]
fn test_seq_double() {
    check(Vec::<f64>::new(), Some(4 + 4));
    check(vec![0f64, 1., 2.], Some(4 + 4 + 4 + 8 * 3));
}

#[test]
fn test_seq_bool() {
    check(Vec::<bool>::new(), Some(4 + 4));
    check(vec![false, true, false], Some(4 + 4 + 1 * 3));
}

#[test]
fn test_seq_string() {
    check(Vec::<String>::new(), Some(4 + 4));
    check(vec!["".to_string(), "a".to_string(), "b".to_string()],
          Some(4 + 4 + 4 + 4 + 1 + 3 + 4 + 1));
}

#[test]
fn test_seq_in_seq() {
    check(vec![Vec::<usize>::new()], Some(4 + 8));
    check(vec![vec![1i64, 3, 5], vec![-1, -3, -5]], Some(4 + 64));
}

#[test]
#[allow(const_err)]
fn test_array_octet() {
    check([] as [u8; 0], Some(4 + 0));
    check([0u8, 1, 2], Some(4 + 3));
}

#[test]
#[allow(const_err)]
fn test_array_char() {
    check([] as [char; 0], Some(4 + 0));
    check(['a', 'b', 'c'], Some(4 + 3));
}

#[test]
#[allow(const_err)]
fn test_array_unsigned_short() {
    check([] as [u16; 0], Some(4 + 0));
    check([0u16, 1, 2], Some(4 + 6));
}

#[test]
#[allow(const_err)]
fn test_array_short() {
    check([] as [i16; 0], Some(4 + 0));
    check([0i16, 1, 2], Some(4 + 6));
}

#[test]
#[allow(const_err)]
fn test_array_unsigned_long() {
    check([] as [u32; 0], Some(4 + 0));
    check([0u32, 1, 2], Some(4 + 12));
}

#[test]
#[allow(const_err)]
fn test_array_long() {
    check([] as [i32; 0], Some(4 + 0));
    check([0i32, 1, 2], Some(4 + 12));
}

#[test]
#[allow(const_err)]
fn test_array_unsigned_long_long() {
    check([] as [u64; 0], Some(4 + 0));
    check([0u64, 1, 2], Some(4 + 24));
}

#[test]
#[allow(const_err)]
fn test_array_long_long() {
    check([] as [i64; 0], Some(4 + 0));
    check([0i64, 1, 2], Some(4 + 24));
}

#[test]
#[allow(const_err)]
fn test_array_float() {
    check([] as [f32; 0], Some(4 + 0));
    check([0f32, 1., 2.], Some(4 + 12));
}

#[test]
#[allow(const_err)]
fn test_array_double() {
    check([] as [f64; 0], Some(4 + 0));
    check([0f64, 1., 2.], Some(4 + 24));
}

#[test]
#[allow(const_err)]
fn test_array_bool() {
    check([] as [bool; 0], Some(4 + 0));
    check([false, true, false], Some(4 + 3));
}

#[test]
#[allow(const_err)]
fn test_array_string() {
    check([] as [String; 0], Some(4 + 0));
    check(["".to_string(), "a".to_string(), "b".to_string()],
          Some(4 + 4 + 5 + 3 + 5));
}

#[test]
#[allow(const_err)]
fn test_array_in_array() {
    check([[]] as [[usize; 0]; 1], Some(4 + 0));
    check([[3.14f64, 2.71, 1.41], [1.73, 2.23, 2.44]], Some(4 + 48));
}

#[test]
fn test_tuple() {
    check((1u32), Some(4 + 4));
    check((1u32, 2i32), Some(4 + 4 + 4));
    check((1u16, 2i16, 3.14f32, "hi".to_string()),
          Some(4 + 2 + 2 + 4 + 6));
}

#[test]
fn test_tuple_containing_padding() {
    check((true, 1u64, 'z', 2.71f32), Some(4 + 24));
}

#[test]
fn test_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct S {
        c: char,
        n: i32,
        b: bool,
        m: u64,
        s: String,
    }

    check(S {
              c: 'x',
              n: -7,
              b: true,
              m: 17,
              s: "hello".to_string(),
          },
          Some(4 + 33));
}

#[test]
fn test_struct_in_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Outer {
        i: Inner1,
        ii: Inner2,
        iii: Inner3,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Inner1 {
        a: i32,
        b: u64,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Inner2 {
        a: bool,
        b: f64,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Inner3 {
        a: char,
        b: f32,
    }

    check(Outer {
              i: Inner1 { a: -3, b: 5 },
              ii: Inner2 { a: false, b: 1.414 },
              iii: Inner3 { a: 'a', b: 1.732 },
          },
          Some(4 + 40));
}

#[test]
fn test_enum() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum E {
        One = 0,
        Two,
        Three,
    }

    check(vec![E::One, E::Two, E::Three], Some(4 + 4 + 4 * 3));
    check(vec![E::One as u32, E::Two as u32, E::Three as u32],
          Some(4 + 4 + 4 * 3));
}

#[test]
fn test_union() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum U {
        A(u32),
        B(i16, u32, u64),
        C {
            c: char,
            n: u32,
            b: bool,
            v: Vec<u8>,
        },
        D,
    }

    check(U::A(3), Some(4 + 4 + 4));
    check(U::B(1, 2, 3), Some(4 + 4 + 2 + 2 + 4 + 4 + 8));
    check(U::C {
              c: 'a',
              n: 5,
              b: true,
              v: vec![1, 1, 2, 3, 5],
          },
          Some(4 + 4 + 1 + 3 + 4 + 1 + 3 + 4 + 5));
    check(U::D, Some(4 + 4));
}

#[test]
fn test_unsupported() {
    use std::collections::{HashMap, BTreeMap};

    fn check_error_kind<T: Debug>(res: Result<T>) {
        match res.map_err(|e| *e) {
            Err(ErrorKind::TypeNotSupported) => (),
            e => panic!("unexpected error kind: {:?}", e),
        }
    }

    check_error_kind(cdr::serialize::<_, _, CdrBe>(&Some(1usize), Infinite));
    check_error_kind(cdr::serialize::<_, _, CdrBe>(&None::<usize>, Infinite));
    check_error_kind(cdr::serialize::<_, _, CdrBe>(&HashMap::<usize, usize>::new(),
                                                   Infinite));
    check_error_kind(cdr::serialize::<_, _, CdrBe>(&BTreeMap::<usize, usize>::new(),
                                                   Infinite));

    check_error_kind(cdr::deserialize::<Option<usize>>(&[0; 16]));
    check_error_kind(cdr::deserialize::<HashMap<usize, usize>>(&[0; 16]));
    check_error_kind(cdr::deserialize::<BTreeMap<usize, usize>>(&[0; 16]));
}
