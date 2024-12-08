//! Result and Error types for ntools-meshtal

use derive_more::From;

use crate::format::Format;
use crate::point::Point;

/// Type alias for Result<T, mesh::Error>
pub(crate) type Result<T> = core::result::Result<T, Error>;

/// The error type for `ntools-mesh`
#[derive(Debug, From)]
pub enum Error {
    /// Errors from std::io
    #[from]
    Io(std::io::Error),

    /// Errors from the vtkio crate
    #[from]
    Vtkio(vtkio::Error),

    /// Errors from ntools utilities
    #[from]
    NtoolsUtils(ntools_utils::Error),

    /// Unable to create a `target` type from `input`
    FailedToParseType { target: String, input: String },

    /// Unable to detect the mesh type from the contect of a file
    UnknownMeshFormat { mesh_id: u32, format: Format },

    /// The tally <mesh_id> could not be found in a file
    TallyNotFound { mesh_id: u32 },

    /// Unable to find a point within the mesh
    PointNotFound { point: Point },

    /// Empty collection: i.e. vector, array, slice, etc... of len()==0
    EmptyCollection,

    /// Index outside an acceptable index range
    IndexOutOfBounds {
        minimum: usize,
        maximum: usize,
        actual: usize,
    },

    /// Collection length does not match the expectation
    UnexpectedLength { expected: usize, found: usize },

    /// The number of voxels in a [Mesh](crate::mesh::Mesh) does not match the expectation
    UnexpectedNumberOfVoxels {
        id: u32,
        expected: usize,
        found: usize,
    },

    /// Clearer parser errors with better context
    FailedParse { reason: String, context: String },

    /// Raw nom crate errors
    Nom(String),
}

// Boilerplate for the library. Anyone using the library is a developer and
// will only care about the debug form anyway. Applications should convert the
// errors to something with more readable, high-level context for the user.
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// todo: dumb hack for lazy mapping of nom error types for now
// this should really implement nom::error::ParseError<&str> and
// nom::error::ContextError<&str> for Error really
impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        Self::Nom(format!("{err:?}"))
    }
}
