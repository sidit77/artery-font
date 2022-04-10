use std::fmt::{Display, Formatter};
use std::io::Error as IoError;
use std::string::FromUtf8Error as Utf8Error;
use std::error::Error as StdError;
#[cfg(feature = "png")]
use png::DecodingError as PngError;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    Utf8Error(Utf8Error),
    DecodeError(String),
    #[cfg(feature = "png")]
    PngError(PngError)
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => err.fmt(f),
            Error::Utf8Error(err) => err.fmt(f),
            Error::DecodeError(err) => write!(f, "Decoding Error: {}", err),
            #[cfg(feature = "png")]
            Error::PngError(err) => err.fmt(f),
        }
    }
}

impl StdError for Error {

}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::IoError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

#[cfg(feature = "png")]
impl From<PngError> for Error {
    fn from(err: PngError) -> Self {
        Self::PngError(err)
    }
}

#[macro_export]
macro_rules! fail {
	($($arg:tt)*) => {{
		return Err($crate::Error::DecodeError(std::format!($($arg)*)))
	}};
}

#[macro_export]
macro_rules! ensure {
	( $x:expr, $($arg:tt)*) => {{
		if !$x {
			$crate::fail!($($arg)*);
		}
	}};
}