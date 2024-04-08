//! Result and Error types for the posvol module

/// Type alias for `Result<T, posvol::Error>`
pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
/// The error type for `ntools-posvol`
pub enum Error {
    /// Underlying file I/O error
    #[error("failure in file I/O")]
    IOError(#[from] std::io::Error),

    /// Failure to deserialise a byte stream
    #[error("failed to deserialise byte stream")]
    UnableToDeserialise(#[from] Box<bincode::ErrorKind>),

    /// Failure to serialise to a JSON string
    #[error("failed serde JSON operation")]
    JSONError(#[from] serde_json::Error),

    /// Unexpected length of bytes based on file content
    #[error("unexpected byte length (expected {expected:?}, found {found:?})")]
    UnexpectedByteLength { expected: i32, found: i32 },
}
