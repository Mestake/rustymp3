use std::{fmt, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    UnsupportedFormat(&'static str),
    InvalidHeader(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            UnsupportedFormat(what) => {
                write!(f, "unsupported format: {}", what)
            }
            InvalidHeader(err) => {
                write!(f, "invalid header: {}", err)
            }
        }
    }
}

// impl<'a> From<&'a str> for Error {
//     fn from(msg: &'a str) -> Self {

//     }
// }
