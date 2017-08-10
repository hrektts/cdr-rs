use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

pub type Error = Box<ErrorKind>;

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match **self {
            ErrorKind::Io(ref err) => std::error::Error::description(err),
            _ => {
                // If you want a better message, use Display::fmt or to_string().
                "CDR error"
            }
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        unimplemented!();
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match **self {
            ErrorKind::Io(ref err) => fmt::Display::fmt(err, f),
            ErrorKind::Message(ref msg) => f.write_str(msg),
            ErrorKind::NumberOutOfRange => f.write_str("number out of range"),
            ErrorKind::SequenceMustHaveLength => f.write_str("sequence or map has unknown size"),
            ErrorKind::SizeLimit => f.write_str("size limit"),
            ErrorKind::TypeNotSupported => f.write_str("type not supported"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
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
    Io(std::io::Error),
    Message(String),
    NumberOutOfRange,
    SequenceMustHaveLength,
    SizeLimit,
    TypeNotSupported,
}
