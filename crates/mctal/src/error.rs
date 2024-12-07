//! Result and Error type
use derive_more::From;

/// Type alias for `Result<T, mctal::Error>`
pub(crate) type Result<T> = core::result::Result<T, Error>;

/// The error type for `ntools-mctal`
#[derive(Debug, From)]
pub enum Error {
    /// Reader has reached the end of the file
    EndOfFile,

    /// Errors from std::io
    #[from]
    Io(std::io::Error),

    /// Errors from ntools utilities
    #[from]
    NtoolsUtils(ntools_utils::Error),

    /// Raw nom crate errors
    Nom(String),

    /// Check to make sure there is a Tmesh to write to. Should be unreachable.
    NoTmeshInitialised,

    /// Check to make sure there is a Tally to write to. Should be unreachable.
    NoTallyInitialised,

    /// Unexpected number of KCODE values
    UnexpectedNumberOfKcodeValues { expected: String, found: usize },

    /// Unexpected number of TMESH bounds (cora + corb + corc + 3)
    UnexpectedNumberOfTmeshBounds { expected: usize, found: usize },

    /// Unexpected length of parsed values
    UnexpectedLength { expected: usize, found: usize },

    /// Unexpected maker found in a line
    UnexpectedKeyword { expected: String, found: String },

    /// Unable to infew particle type from a string
    FailedToInferParticle { tag: String },
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

// this should really implement nom::error::ParseError<&str> and
// nom::error::ContextError<&str> for Error really
impl From<nom::Err<nom::error::Error<&str>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        Self::Nom(format!("{err:?}"))
    }
}
