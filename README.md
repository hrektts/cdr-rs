# cdr-rs

A serialization/deserialization implementation for Common Data Representation in Rust.

[![Build Status](https://travis-ci.org/hrektts/cdr-rs.svg?branch=master)](https://travis-ci.org/hrektts/cdr-rs)
[![Crates.io](https://img.shields.io/crates/v/cdr.svg?maxAge=2592000)](https://crates.io/crates/cdr)

[Documentation](https://docs.rs/cdr)

## Usage

Add this to your Cargo.toml:

``` toml
[dependencies]
cdr = "0.2.1"
```

Then add this to your crate:

``` rust
extern crate cdr;
```

## Example

``` rust
extern crate cdr;
#[macro_use]
extern crate serde_derive;

use cdr::{CdrBe, Infinite};

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

    let encoded = cdr::serialize::<_, _, CdrBe>(&triangle, Infinite).unwrap();
    let decoded = cdr::deserialize::<Polygon>(&encoded[..]).unwrap();

    assert!(triangle == decoded);
}
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## References

- [Common Object Request Broker Architecture (CORBA) Specification, Version 3.3](http://www.omg.org/spec/CORBA/3.3/), Part 2: CORBA Interoperability
