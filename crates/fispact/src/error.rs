//! Result and Error types for ntools-meshtal

/// Type alias for Result<T, fispact::Error>
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for the `ntools-fispact` crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed input/output stream")]
    IOError(#[from] std::io::Error),

    #[error("failed to (de)serialise")]
    FailedSerde(#[from] serde_json::Error),
}
