//! When serializing or deserializing VDF goes wrong

use std::fmt::{self, Display};

use serde::{de, ser};

/// Alias for a `Result` with the error type `vdf_serde::Error`
pub type Result<T> = std::result::Result<T, Error>;

/// This type represents all possible errors that can occur when serializing or
/// deserializing VDF data
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Error {
    /// An error carried up from the specific Serialize/Deserialize implementation
    Message(String),

    /// A syntax error in deserializing (vague because I am lazy)
    Parse,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Parse => formatter.write_str("parse error"),
        }
    }
}

impl std::error::Error for Error {}
