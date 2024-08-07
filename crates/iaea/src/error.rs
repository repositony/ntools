//! Result and Error types for the IAEA data module

use crate::RadType;

/// Type alias for `Result<T, iaea::Error>`
pub(crate) type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
/// The error type for `ntools-iaea`
pub enum Error {
    /// Underlying file I/O error
    #[error("failure in file I/O")]
    Io(#[from] std::io::Error),

    /// Failure during GET request to IAEA API
    #[error("request to IAEA API failed")]
    FailedRequest(#[from] minreq::Error),

    /// Failure to serialise to a JSON string
    #[error("failed serde JSON operation")]
    Json(#[from] serde_json::Error),

    /// Unexpected length of bytes based on file content
    #[error("failed CSV operation")]
    Csv(#[from] csv::Error),

    /// Radiation type could not be infered from the provided string
    #[error("failed infer radiation type from \"{hint:?}\"")]
    CouldNotInferRadType { hint: String },

    /// Failure to serialize/deserialize a byte stream
    #[error("failed binary (de)serialization")]
    FailedBinaryOp(#[from] Box<bincode::ErrorKind>),

    /// Specific hinted Nuclide conversion error
    #[error("failed to parse \"{text:?}\" to Nuclide")]
    FailedStrToNuclide { text: String },

    /// Generic TryFrom/TryInto for Nuclide conversion
    #[error("failed to coerce generic type into Nuclide")]
    FailedNuclideConversion,

    /// Invalid nuclide state for IAEA API queries
    #[error("IAEA API does not allow elements")]
    InvalidNuclideQuery,

    /// Raised if the hashmap of collected nuclide data is empty
    #[error("hashmap of collected nuclide data is empty")]
    EmptyDataMap,

    /// Unexpected length of bytes based on file content
    #[error("failed to find \"{nuclide:?}\" for {rad_type:?}")]
    FailedToLoad { nuclide: String, rad_type: RadType },
}
