//! Result and Error types for the utils module

/// Type alias for `Result<T, utils::Error>`
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
/// The error type for `ntools_utils`
pub enum Error {
    /// An empty slice of floats for SliceExt
    SliceContainsNoValues,

    /// The slice of float values contains things like NAN or INFINITY
    SliceContainsUndefinedValues,

    /// A catch-all for functions returning errors that should be unreachable
    UncapturedErrorCondition,

    /// Value that is searched for outside of the min/max of the array
    ValueOutsideOfBounds {
        value: f64,
        lower_bound: f64,
        upper_bound: f64,
    },

    /// For when a slice has fewer than the minimum required values
    BelowMinimumSliceLength {
        length: usize,
        minimum_required: usize,
    },

    /// The tolerance for voxels should not be greater than 100% of the width
    UnreasonableBoundaryTolerance {
        tolerance: f64,
        minimum: f64,
        maximum: f64,
    },
}

// error boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

// implement standard error trait to use with ? operator
impl std::error::Error for Error {}
