//! Mesh tally tools and file parsing
//!
//! Module for storing and using mesh data from all output formats and geometry
//! types.
//!
//! All of the parsing and reader logic is re-exported to make reading
//! files very simple, regardless of format. For example:
//!
//! ```rust, no_run
//! # use ntools_mesh::{read,read_target};
//! // Extract ALL meshes from a file into a Vec<Mesh>
//! let mesh_list = read("/path/to/meshtal.msht").unwrap();
//!
//! // Extract just one target Mesh from a file
//! let mesh = read_target("/path/to/meshtal.msht", 104).unwrap();
//! ```
//!
//! ## Supported output formats
//!
//! For more detail, see the `OUT` keyword for the `FMESH` card definition in
//! the [MCNPv6.2](https://mcnp.lanl.gov/pdf_files/TechReport_2017_LANL_LA-UR-17-29981_WernerArmstrongEtAl.pdf)
//! or [MCNPv6.3](https://mcnpx.lanl.gov/pdf_files/TechReport_2022_LANL_LA-UR-22-30006Rev.1_KuleszaAdamsEtAl.pdf)
//! user manuals.
//!
//! | Output format | Supported? | Description                                         |
//! | ------------- | ---------- | --------------------------------------------------- |
//! | COL           | Yes        | Column data (MCNP default)                          |
//! | CF            | Yes        | Column data including voxel volume                  |
//! | IJ            | Yes        | 2D matrix of I (col) and J (row) data, grouped by K |
//! | IK            | Yes        | 2D matrix of I (col) and K (row) data, grouped by J |
//! | JK            | Yes        | 2D matrix of J (col) and K (row) data, grouped by I |
//! | CUV (UKAEA)   | Yes        | UKAEA Cell-under-Voxel column data                  |
//! | NONE          | N/A        | `NONE` or unknown output format                     |
//!
//! Once I get my paws on MCNPv6.3 this will be extended to include the new
//! COLSCI, CFSCI, and XDMF/HDF5 formats.
//!
//! ## Supported mesh geometries
//!
//! All functionality is fully supported for both rectangular and cylindrical meshes.
//!
//! | Mesh geometry | Supported? | MCNP designators |
//! | ------------- | ---------- | ---------------- |
//! | Rectangular   | Yes        | rec, xyz         |
//! | Cylindrical   | Yes        | cyl, rzt         |
//! | Spherical     | No         | sph, rpt         |
//!
//! Spherical meshes are not currently supported because barely anyone knows
//! about them, let alone uses them. They are a low priority.
//!
//! ## Quickstart example
//!
//! Example to read in a mesh tally and perform varius operations.
//!
//! ```rust, no_run
//! # use ntools_mesh::{Mesh,read_target,mesh_to_vtk,write_vtk};
//! # use ntools_mesh::vtk::{VtkFormat};
//! fn main() {
//!     // Read mesh 104 from the meshtal file
//!     let mut mesh = read_target("./data/meshes/fmesh_104.msht", 104).unwrap();
//!
//!     // print a summary of the mesh (Display trait impemented)
//!     println!("{mesh}");
//!
//!     // rescale, get data, do whatever is needed
//!     mesh.scale(1.0e+06);
//!     println!("Maximum flux: {:?}", mesh.maximum());
//!     println!("Minimum flux: {:?}", mesh.minimum());
//!     println!("Average flux: {:?}", mesh.average());
//!
//!     // Convert to VTK with the default configuration
//!     let vtk = mesh_to_vtk(&mesh);
//!
//!     // Write the VTK to a file in one of several formats
//!     write_vtk(vtk, "my_output.vtk", VtkFormat::Xml).unwrap();
//! }
//! ```

// Split into subfiles for development, but anything important is re-exported
mod error;
mod format;
mod geometry;
mod group;
mod mesh;
mod particle;
mod point;
mod voxel;

pub mod reader;
pub mod vtk;

// inline important the mesh-related modules for a nice public API
#[doc(inline)]
pub use reader::{read, read_target};

#[doc(inline)]
pub use mesh::Mesh;

#[doc(inline)]
pub use format::Format;

#[doc(inline)]
pub use geometry::Geometry;

#[doc(inline)]
pub use group::Group;

#[doc(inline)]
pub use particle::Particle;

#[doc(inline)]
pub use voxel::{Voxel, VoxelCoordinate, VoxelSliceExt};

#[doc(inline)]
pub use vtk::{mesh_to_vtk, write_vtk};

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use point::{BoundaryTreatment, Point, PointKind};
