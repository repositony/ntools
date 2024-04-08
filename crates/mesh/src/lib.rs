//! Mesh tally tools and file parsing
#![doc = include_str!("../readme.md")]

// Split into subfiles for development, but anything important is re-exported
mod error;
mod mesh;
mod particle;
mod voxel;

pub mod reader;
pub mod vtk;

// inline important the mesh-related modules for a nice public API
#[doc(inline)]
pub use reader::{read_meshtal, read_meshtal_target};

#[doc(inline)]
pub use mesh::{Format, Geometry, Mesh};

#[doc(inline)]
pub use particle::Particle;

#[doc(inline)]
pub use voxel::{Group, Voxel, VoxelCoordinate};

#[doc(inline)]
pub use vtk::{mesh_to_vtk, write_vtk};

#[doc(inline)]
pub use error::Error;
