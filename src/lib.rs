//! A serialization/deserialization implementation for Common Data Representation.

extern crate byteorder;
extern crate serde;

pub mod de;
pub use de::{deserialize, deserialize_from, Deserializer};

mod encapsulation;
pub use encapsulation::{CdrBe, CdrLe, Encapsulation, PlCdrBe, PlCdrLe};

mod error;
pub use error::{Error, ErrorKind, Result};

pub mod ser;
pub use ser::{serialize, serialize_into, Serializer};

mod size;
pub use size::{calc_serialized_size, calc_serialized_size_bounded, SizeLimit, Bounded, Infinite};
