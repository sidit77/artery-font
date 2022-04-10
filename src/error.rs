use std::fmt::{Display, Formatter};
use std::io::Error as IoError;
use std::string::FromUtf8Error as Utf8Error;
use std::error::Error as StdError;
#[cfg(feature = "png")]
use png::DecodingError as PngError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Utf8(Utf8Error),
    Decode(String),
    #[cfg(feature = "png")]
    Png(PngError)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Utf8(err) => err.fmt(f),
            Error::Decode(err) => write!(f, "Decoding Error: {}", err),
            #[cfg(feature = "png")]
            Error::Png(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {

}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

#[cfg(feature = "png")]
impl From<PngError> for Error {
    fn from(err: PngError) -> Self {
        Self::Png(err)
    }
}
