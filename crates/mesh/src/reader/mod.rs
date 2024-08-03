//! Parsers and logic for reading meshtal files
//!
//! All functions are re-exported to the crate root for easy access.
//!
//! # Quickstart
//!
//! The simplest methods for reading tallies from a meshtal file are the
//! convenience functions:
//!
//! ```rust, no_run
//! # use ntools_mesh::{read_meshtal, read_meshtal_target, Mesh};
//! // Extract all meshes from a file into a Vec<Mesh>
//! let mesh_list = read_meshtal("/path/to/meshtal.msht").unwrap();
//!
//! // Extract just one target mesh from a file into a single Mesh
//! let mesh = read_meshtal_target("/path/to/meshtal.msht", 104).unwrap();
//! ```
//!
//! Under the hood these initialise the [MeshtalReader]. This is made public for
//! fine control if absolutely needed, but the convenience functions are the
//! preferred API for this module.
//!
//! ```rust, no_run
//! # use ntools_mesh::{reader::MeshtalReader, Mesh};
//! # use std::path::Path;
//! // Initialise the reader and set some options
//! let mut reader = MeshtalReader::new();
//! reader.disable_progress(); // disable the progress bar
//! reader.set_target_id(104); // choose a specific tally
//!
//! // Parse the file
//! let path = Path::new("/path/to/meshtal.msht");
//! let mesh_list = reader.parse(path).unwrap();
//! ```
//!
//! # Implementation overview
//!
//! When `read_meshtal()` is called, a [MeshtalReader] is initialised and the
//! `parse()` method called on the provided file.
//!
//! This proceeds in two stages:
//! - Pre-processing to check the tally numbers and formatting styles
//! - Extraction of each tally to a [Mesh] object
//!
//! The meshtal files are parsed line-by-line using a buffered input stream so
//! that, at most, a single line is held in memory. This reduces memory
//! requirements significantly.
//!
//! A lot of the mesh data are also derivable from the header information. For
//! example, there is no need to store all of the voxel coordiante data
//! explicitly. See [Voxel](crate::Voxel) for detalis, but this further reduces
//! large meshes to ~24B per voxel.
//!
//! # Formatting notes
//!
//! While not unique to CuV formats, it is certainly the biggest offender for
//! a couple of formatting issues that break most other parsers.
//!
//! **Run-on numbers without whitespace are handled**
//!
//! > For example, `1.00E+00-2.00E+00` is recognised and parsed as
//! > `1.00E+00 -2.00E+00` instead.
//!
//! **Broken exponential formatting is handled**
//!
//! > For example, `1.00+002` is recognised and parsed as `1.00E+002` as though
//! > the missing exponent character is there.
//!
//! **Unphysical -ve results are handled**
//!
//! > Only an issue for CuV, but it is possible to have -ve flux in the results
//! > for whatever reason. These are considered to be 0.0 in the voxel results.
//!
//! **Warnings are given for precision issues**
//!
//! > Certainly a problem for Novel 1-Step calculations, but for whatever reason
//! > the MCNP developers decided that the mesh bounds were fine to have 3
//! > significant figures.
//!
//! > This means that time and energy bins that are extremely large or close
//! > together are identical and can not be used when searching for groups.
//!
//! > It is therefore recommended that groups are found by index whenever this
//! > warning is raised.
//!
//! # Cell-Under-Voxel (CuV) notes
//!
//! The UKAEA CuV modification for MCNP is a huge pain, but is fully supported
//! though all the same interfaces.
//!
//! ## Volume weighting
//!
//! There are often multiple cells that correspond to a single voxel. The goal
//! is to turn these into a single result.
//!
//! Every cell contribution to the total flux in a voxel is weighted by the
//! volume it occupies. i.e.
//!
//! ```text
//! weight = cell volume / voxel volume
//! ```
//!
//! This is then applied as appropriate to the result and uncertainty.
//!
//! ## Void records
//!
//! To save space in the meshtal files the `voidoff` option can be specified in
//! an MCNP run. This corresponds to the [VoidRecord] variants to be explicit.
//! - [VoidRecord::On] = Void cells are included in output data
//! - [VoidRecord::Off] = Void cells are excluded in output data
//!
//! Both are completely handled in the background, but it is worth noting that
//! any missing voxels for the [VoidRecord::Off] will be included in the [Mesh].
//!
//! These voxels with have a result and error of `0.0``, which is not accurate
//! but it is necessary to fill the gaps.
//!
//! ## Supplementary data
//!
//! It is worth noting that all cell data on volumes, cells, etc... are parsed
//! into [CellData]. However, these data are generally discarded to minimise
//! memory requirements.
//!
//! Future updates may store these data if it becomes useful to do so.

// reader modules
mod meshtal;
mod parsers;

// re-exports for clean API + documentation
#[doc(inline)]
pub use meshtal::{CellData, MeshtalReader, VoidRecord};

// library imports
use crate::error::Result;
use crate::Mesh;
use std::path::Path;

/// Read all meshes in a meshtal file
///
/// Returns a result containing a vector of [Mesh] structs extracted from the
/// file at `path` by the parser.
///
/// - `path` - Path to the meshtal file, can be [&str], [String], [Path], etc...
///
/// Example
/// ```rust, no_run
/// # use ntools_mesh::{Mesh, read_meshtal};
/// // Read every mesh contained in the file
/// let mesh_tallies: Vec<Mesh> = read_meshtal("path/to/meshtal.msht").unwrap();
/// ```
pub fn read_meshtal<P: AsRef<Path>>(path: P) -> Result<Vec<Mesh>> {
    let path: &Path = Path::new(path.as_ref());
    let mut reader = MeshtalReader::new();
    reader.disable_progress();
    reader.parse(path)
}

/// Read only the specified mesh from a meshtal file
///
/// Returns a result of the targeted [Mesh] if it was successfully
/// extracted from the file at `path`.
///
/// - `path` - Path to the meshtal file, can be [&str], [String], [Path], etc...
/// - `target` - Tally number of interest
///
/// Example
/// ```rust, no_run
/// # use ntools_mesh::{Mesh, read_meshtal_target};
/// // Read only tally 104 (i.e. FMESH104) from the file
/// let mesh: Mesh = read_meshtal_target("path/to/meshtal.msht", 104).unwrap();
/// ```
pub fn read_meshtal_target<P: AsRef<Path>>(path: P, target: u32) -> Result<Mesh> {
    let path: &Path = Path::new(path.as_ref());
    let mut reader = MeshtalReader::new();
    reader.disable_progress();
    reader.set_target_id(target);
    let mut mesh_list = reader.parse(path)?;
    Ok(mesh_list.remove(0))
}
