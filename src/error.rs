use std::error;
use std::fmt;
use std::io;
use std::result;

use serde::{de, ser};

pub type Error = Box<ErrorKind>;

impl error::Error for Error {
    fn description(&self) -> &str {
        match **self {
            ErrorKind::Io(ref err) => error::Error::description(err),
            _ => {
                // If you want a better message, use Display::fmt or to_string().
                "CDR error"
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        unimplemented!();
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match **self {
            ErrorKind::Io(ref err) => fmt::Display::fmt(err, f),
            ErrorKind::Message(ref msg) => f.write_str(msg),
            ErrorKind::NumberOutOfRange => f.write_str("number out of range"),
            ErrorKind::SequenceMustHaveLength => {
                f.write_str("sequence or map has unknown size")
            }
            ErrorKind::SizeLimit => f.write_str("size limit"),
            ErrorKind::TypeNotSupported => f.write_str("type not supported"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        ErrorKind::Io(err).into()
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorKind::Message(msg.to_string()).into()
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorKind::Message(msg.to_string()).into()
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    Message(String),
    NumberOutOfRange,
    SequenceMustHaveLength,
    SizeLimit,
    TypeNotSupported,
}

pub type Result<T> = result::Result<T, Error>;
