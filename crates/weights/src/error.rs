//! Result and Error types for ntools-weights

/// Type alias for Result<T, weights::Error>
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for the `ntools-weights` crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed input/output stream")]
    IOError(#[from] std::io::Error),

    #[error("vtkio error")]
    VtkioError(#[from] vtkio::Error),
}
