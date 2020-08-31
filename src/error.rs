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

    /// Unsupported Serde data type
    UnsupportedType(&'static str),

    /// EOF too early
    EarlyEOF,

    /// EOF too late
    LateEOF,

    /// Tokenization error
    /// (This could be a nom::Err but I don't want ancient nom in my type signature)
    Tokenize(String),

    /// A mismatch between an expected token and a real token
    Expected(&'static str, String),

    /// Failed to parse from a string to some other type
    StringParse(String)
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
            Error::UnsupportedType(r#type) => write!(formatter, "unsupported Serde data type {} used", r#type),
            Error::EarlyEOF => formatter.write_str("input ended early"),
            Error::LateEOF => formatter.write_str("input ended late"),
            Error::Tokenize(err) => formatter.write_str(err),
            Error::Expected(wanted, got) => write!(formatter, "expected {}, got {}", wanted, got),
            Error::StringParse(err) => formatter.write_str(err),
        }
    }
}

impl std::error::Error for Error {}
