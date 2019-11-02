use std::{
    self,
    fmt::{self, Display},
    io,
    str::Utf8Error,
};

/// Convenient wrapper around `std::Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type.
#[derive(Debug)]
pub enum Error {
    Message(String),
    Io(io::Error),
    DeserializeAnyNotSupported,
    InvalidBoolEncoding(u8),
    InvalidChar(char),
    InvalidCharEncoding,
    InvalidEncapsulation,
    InvalidUtf8Encoding(Utf8Error),
    NumberOutOfRange,
    SequenceMustHaveLength,
    SizeLimit,
    TypeNotSupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match *self {
            Message(ref msg) => Display::fmt(msg, f),
            Io(ref err) => Display::fmt(err, f),
            DeserializeAnyNotSupported => write!(
                f,
                "does not support the serde::Deserializer::deserialize_any method"
            ),
            InvalidBoolEncoding(v) => write!(f, "expected 0 or 1, found {}", v),
            InvalidChar(v) => write!(f, "expected char of width 1, found {}", v),
            InvalidCharEncoding => write!(f, "char is not valid UTF-8"),
            InvalidEncapsulation => write!(f, "encapsulation is not valid"),
            InvalidUtf8Encoding(ref err) => Display::fmt(err, f),
            NumberOutOfRange => write!(f, "sequence is too long"),
            SequenceMustHaveLength => {
                write!(f, "sequences must have a knowable size ahead of time")
            }
            SizeLimit => write!(f, "the size limit has been reached"),
            TypeNotSupported => write!(f, "unsupported type"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match *self {
            Io(ref err) => err.description(),
            InvalidUtf8Encoding(ref err) => err.description(),
            _ => {
                // If you want a better message, use Display::fmt or to_string().
                "CDR error"
            }
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use Error::*;

        match *self {
            Io(ref err) => Some(err),
            InvalidUtf8Encoding(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}
