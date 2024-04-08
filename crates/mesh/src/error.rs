//! Result and Error types for ntools-meshtal

/// Type alias for Result<T, mesh::Error>
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for the `ntools-mesh` crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed input/output stream")]
    IOError(#[from] std::io::Error),

    #[error("vtkio error")]
    VtkioError(#[from] vtkio::Error),

    #[error("failed to infer particle from \"{0}\"")]
    FailedToInferParticle(String),

    #[error("parser failed")]
    ParseError(String),

    #[error("format of mesh {0} is unknown")]
    UnknownMeshFormat(u32),

    #[error(
        "inconsistent number of voxels in mesh {id:?} (expected {expected:?}, found {found:?})"
    )]
    UnexpectedNumberOfVoxels {
        id: u32,
        expected: usize,
        found: usize,
    },

    #[error(
        "inconsistent length material cells per voxel array (expected {expected:?}, found {found:?})"
    )]
    UnexpectedMcpvLength { expected: usize, found: usize },

    #[error("failed to infer geometry from \"{0}\"")]
    FailedToInferGeometry(String),

    #[error("failed to infer group from \"{0}\"")]
    FailedToInferGroup(String),

    #[error("tally \"{0}\" not found")]
    TallyNotFound(u32),

    #[error("failed to infer void_off status from \"{0}\"")]
    FailedToInferVoidoff(String),

    #[error("failure in mesh operations")]
    MeshError(String),
}
