use std;
use std::error;
use std::fmt;
use std::io;
use std::str;
use std::string;

use serde_json;

/// Represents results from this library
pub type Result<T> = std::result::Result<T, Error>;

/// Represents different failure cases
#[derive(Debug)]
pub enum Error {
    /// a std::io error
    Io(io::Error),
    /// a std::str::Utf8Error
    Utf8(str::Utf8Error),
    /// a JSON parsing related failure
    BadJson(serde_json::Error),
    /// a VLQ string was malformed and data was left over
    VlqLeftover,
    /// a VLQ string was empty and no values could be decoded.
    VlqNoValues,
    /// Overflow in Vlq handling
    VlqOverflow,
    /// a mapping segment had an unsupported size
    BadSegmentSize(u32),
    /// a reference to a non existing source was encountered
    BadSourceReference(u32),
    /// a reference to a non existing name was encountered
    BadNameReference(u32),
    /// Indicates that an indexed sourcemap was encountered when
    /// a regular sourcemap was expected
    IndexedSourcemap,
    /// Indicates that an regular (non-indexed) sourcemap was when
    /// a sourcemap index was expected
    RegularSourcemap,
    /// Indicates an invalid data URL
    InvalidDataUrl,
    /// Flatten failed
    CannotFlatten(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        From::from(err.utf8_error())
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::BadJson(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use crate::Error::*;
        match *self {
            Io(ref err) => err.description(),
            Utf8(ref err) => err.description(),
            BadJson(ref err) => err.description(),
            VlqLeftover => "vlq leftover",
            VlqNoValues => "no vlq values",
            VlqOverflow => "overflow in vlq",
            BadSegmentSize(_) => "bad segment size",
            BadSourceReference(_) => "bad source reference",
            BadNameReference(_) => "bad name reference",
            IndexedSourcemap => "unexpected indexed sourcemap",
            RegularSourcemap => "unexpected sourcemap",
            InvalidDataUrl => "invalid data URL",
            CannotFlatten(_) => "cannot flatten the given indexed sourcemap",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        use crate::Error::*;
        match *self {
            Io(ref err) => Some(&*err),
            Utf8(ref err) => Some(&*err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::Error::*;
        match *self {
            Io(ref msg) => write!(f, "{}", msg),
            Utf8(ref msg) => write!(f, "{}", msg),
            BadJson(ref err) => write!(f, "bad json: {}", err),
            VlqLeftover => write!(f, "leftover cur/shift in vlq decode"),
            VlqNoValues => write!(f, "vlq decode did not produce any values"),
            VlqOverflow => write!(f, "vlq decode caused an overflow"),
            BadSegmentSize(size) => write!(f, "got {} segments, expected 4 or 5", size),
            BadSourceReference(id) => write!(f, "bad reference to source #{}", id),
            BadNameReference(id) => write!(f, "bad reference to name #{}", id),
            IndexedSourcemap => write!(f, "encountered unexpected indexed sourcemap"),
            RegularSourcemap => write!(
                f,
                "encountered unexpected sourcemap where index was expected"
            ),
            InvalidDataUrl => write!(f, "the provided data URL is invalid"),
            CannotFlatten(ref msg) => write!(f, "cannot flatten the indexed sourcemap: {}", msg),
        }
    }
}
