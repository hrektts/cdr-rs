# cdr-rs

A serialization/deserialization implementation for Common Data Representation in Rust.

[![Build Status](https://travis-ci.org/hrektts/cdr-rs.svg?branch=master)](https://travis-ci.org/hrektts/cdr-rs)

## Example

``` rust
#[macro_use]
extern crate serde_derive;
extern crate cdr;

use cdr::{deserialize, serialize, CdrBe, Infinite};

#[derive(Deserialize, Serialize, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Deserialize, Serialize, PartialEq)]
struct Polygon(Vec<Point>);

fn main() {
    let triangle = Polygon(vec![Point { x: -1.0, y: -1.0 },
                                Point { x: 1.0, y: -1.0 },
                                Point { x: 0.0, y: 0.73 }]);

    let encoded: Vec<u8> = serialize::<_, _, CdrBe>(&triangle, Infinite).unwrap();
    let decoded: Polygon = deserialize(&encoded[..]).unwrap();

    assert!(triangle == decoded);
}
```

## References

- [Common Object Request Broker Architecture (CORBA) Specification, Version 3.3](http://www.omg.org/spec/CORBA/3.3/), Part 2: CORBA Interoperability
