// Crate types
use crate::error::Result;
use crate::reader::Reader;
use crate::{Header, Kcode, Tally, Tmesh};

// Other libraries
use log::info;
use std::path::Path;

/// Data structure to store MCTAL file content
///
/// This [Mctal] is the primary data structure containing the parsed file data.
///
/// The MCNP MCTAL file contains all tally results of one dump of a RUNTAPE file
/// in a relatively fixed-format.
///
/// There are many data collections available, but can be broken into one of the
/// following blocks:
///
/// | Data block | Description                                                  |
/// | ---------- | ------------------------------------------------------------ |
/// | [Header]   | file metadata and run information                            |
/// | [Tally]    | all standard `F` tallies, including tally fluctuation chart  |
/// | [Tmesh]    | superimposed Mesh Tally Type A (a.k.a TMESH)                 |
/// | [Kcode]    | criticality results (KCODE)                                  |
///
/// Note that `TMESH` mesh tallies are written to the MCTAL file, while `FMESH`
/// mesh tallies are not. Tools for reading `FMESH` data are available in other
/// ntools crates (See [mesh](https://repositony.github.io/ntools/ntools_mesh/index.html)).
///
#[derive(Debug, Default)]
pub struct Mctal {
    /// Header information and metadata
    pub header: Header,
    /// Collection of standard tallies
    pub tallies: Vec<Tally>,
    /// Collection of TMESH tallies (MCNPv6.3 only)
    pub tmesh: Vec<Tmesh>,
    /// Kcode run information
    pub kcode: Option<Kcode>,
}

impl Mctal {
    /// Create a new empty [Mctal] struct with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a MCTAL file
    ///
    /// Parses the file at `path` into the [Mctal] data structure for
    /// post-processing.
    ///
    /// The `path` may be a [&str], [String], [Path], etc..
    ///
    /// Example
    /// ```rust, no_run
    /// # use ntools_mctal::Mctal;
    /// // Read every tally contained in the MCTAL file
    /// let mctal: Mctal = Mctal::from_file("path/to/mctal_file").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        info!("Reading {:?}", path.as_ref().file_name().unwrap());
        Reader::new(path)?.read()
    }

    /// Get a reference to header data
    pub fn get_header(&self) -> &Header {
        &self.header
    }

    /// Find a specific tally
    ///
    /// If the tally exists it is returned as `Some(&Tally)`, otherwise `None`.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::{Mctal, Tally};
    /// // Dummy tally with id=104
    /// let mut tally = Tally::default();
    /// tally.id = 104;
    ///
    /// // Add the dummy tally to a MCTAL
    /// let mut mctal = Mctal::default();
    /// mctal.tallies = vec![tally];
    ///
    /// // Will return `Some(&Tally)` if it exists, otherwise `None`
    /// assert!(mctal.get_tally(104).is_some());
    /// assert!(mctal.get_tally(114).is_none());
    /// ```
    pub fn get_tally(&self, id: u32) -> Option<&Tally> {
        self.tallies.iter().find(|tally| tally.id == id)
    }

    /// Find a specific tmesh
    ///
    /// If the mesh exist it is returned as `Some(&Tmesh)`, otherwise `None`.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::{Mctal, Tmesh};
    /// // Dummy mesh with id=104
    /// let mut tmesh = Tmesh::default();
    /// tmesh.id = 104;
    ///
    /// // Add the dummy mesh to a MCTAL
    /// let mut mctal = Mctal::default();
    /// mctal.tmesh = vec![tmesh];
    ///
    /// // Will return `Some(&Tmesh)` if it exists, otherwise `None`
    /// assert!(mctal.get_tmesh(104).is_some());
    /// assert!(mctal.get_tmesh(114).is_none());
    /// ```
    pub fn get_tmesh(&self, id: u32) -> Option<&Tmesh> {
        self.tmesh.iter().find(|tmesh| tmesh.id == id)
    }

    /// Get a reference to kcode results
    pub fn get_kcode(&self) -> Option<&Kcode> {
        self.kcode.as_ref()
    }
}
