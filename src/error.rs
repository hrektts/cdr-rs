use std::{self,
          fmt::{self, Display},
          io,
          str::Utf8Error};

use failure::Context;
use serde;

/// Convenient wrapper around `std::Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type.
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    pub fn kind_ref(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self.inner.get_context() {
            ErrorKind::Io(ref err) => std::error::Error::description(err),
            _ => {
                // If you want a better message, use Display::fmt or to_string().
                "CDR error"
            }
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self.inner.get_context() {
            ErrorKind::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner: inner }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        ErrorKind::Io(err).into()
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        ErrorKind::Message(msg.to_string()).into()
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        ErrorKind::Message(msg.to_string()).into()
    }
}

/// The kind of an error.
#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "{}", _0)]
    Message(String),
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "does not support the serde::Deserializer::deserialize_any method")]
    DeserializeAnyNotSupported,
    #[fail(display = "expected 0 or 1, found {}", _0)]
    InvalidBoolEncoding(u8),
    #[fail(display = "expected char of width 1, found {}", _0)]
    InvalidChar(char),
    #[fail(display = "char is not valid UTF-8")]
    InvalidCharEncoding,
    #[fail(display = "encapsulation is not valid")]
    InvalidEncapsulation,
    #[fail(display = "string is not valid")]
    InvalidUtf8Encoding(Utf8Error),
    #[fail(display = "sequence is too long")]
    NumberOutOfRange,
    #[fail(display = "sequences must have a knowable size ahead of time")]
    SequenceMustHaveLength,
    #[fail(display = "the size limit has been reached")]
    SizeLimit,
    #[fail(display = "unsupported type")]
    TypeNotSupported,
}
