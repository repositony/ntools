//! Weight window generation methods for MCNP
//!
//! This crate contains methods for generating and manipulating weight windows
//! from MCNP FMESH flux tallies.
//!
//! There are many ways to do this. Currently the MAGIC method is implemented,
//! with the option for first generating meshes from the Build-Up Density
//! Exptrapolation (BUDE) method.
//!
//! # Mesh to Weight window
//!
//! ## Basic use
//!
//! A constant power factor and error tolerance are applied to all energy/time
//! groups.
//!
//! - `powers` - Softening factor used as ww=>ww^power
//! - `max_errors` - Errors above this are set to 0/analogue
//! - `total_only` - Only generate weights from [Group::Total](ntools_mesh::Group)
//!
//! Weights are calculated as `(0.5 * (v.result / flux_ref)).powf(power)`. For
//! example, applying a 0.7 de-tuning factor and setting voxels with errors
//! below 10% to analogue:
//!
//! ```rust, no_run
//! # use ntools_mesh::read_target;
//! # use ntools_wwgen::mesh_to_ww;
//! // Read tally 104 from a meshtal file
//! let mesh = read_target("./data/meshes/fmesh_104.msht", 104).unwrap();
//! // Convert the mesh into a weight window set
//! let weight_window = mesh_to_ww(&mesh, 0.7, 0.10, false);
//! ```
//!
//! By default, this generates weight windows for all time and energy groups.
//! To generate a simpler set of weight windows based only on the
//! `Group::Total`, set the `total_only` boolean to `true`.
//!
//! ### Advanced use
//!
//! Mesh tally to global weight windows with fine de-tuning and errors
//!
//! Same as `mesh_to_ww` but allows for individual de-tuning factors and error
//! tolerances for each group. If `powers` or `max_errors` have a single entry
//! this will be applied to all groups.
//!
//! - `powers` - Softening factor used as ww=>ww^power
//! - `max_errors` - Errors above this are set to 0/analogue
//!
//! A call to this may look like this, applying separate powers and errors to
//! a mesh with 3 energy groups:
//!
//! ```rust, no_run
//! # use ntools_mesh::read_target;
//! # use ntools_wwgen::mesh_to_ww_advanced;
//! // Read tally 104 from a meshtal file
//! let mesh = read_target("./data/meshes/fmesh_104.msht", 104).unwrap();
//! // Convert the mesh into a set of weight windows, using different parameters per set
//! let ww = mesh_to_ww_advanced(&mesh,
//!                              &[0.7, 0.5, 0.85],
//!                              &[0.1, 0.1, 0.15]);
//! ```
//!
//! The lists should be ordered such that they match the following nested order:
//!
//! ```text
//! for energy in energy_groups {
//!     for time in time_groups {
//!         calculate weights...
//!     }
//! }
//! ```
//!
//! For example, the following energy and time groups are related to the groups
//! shown explicitly below.
//!
//! ```text
//! Energy bin boundaries: 0.0 10.0 200.0
//! Time bin boundaries  : -1E+36 0.0 1E+16 1E+99
//! ``
//!
//! ```text
//! 0 -> Energy(10.0)   Time(0.0)       powers[0]   max_errors[0]
//! 1 -> Energy(10.0)   Time(1E+16)     powers[1]   max_errors[1]
//! 2 -> Energy(10.0)   Time(1E+99)     powers[2]   max_errors[2]
//! 3 -> Energy(200.0)  Time(0.0)       powers[3]   max_errors[3]
//! 4 -> Energy(200.0)  Time(1E+16)     powers[4]   max_errors[4]
//! 5 -> Energy(200.0)  Time(1E+99)     powers[5]   max_errors[5]
//! ```
//!
//! ## Density extrapolation
//!
//! **Warning: Extremely WIP for testing**
//!
//! A very quick and naive implementation for the work in progress implemntation
//! of the Build-Up Density Exptrapolation (BUDE) method.
//!
//! This takes several meshes as arguments:
//!
//! - void mesh (vd), run with voided materials
//! - uncollided (uc) flux mesh
//! - reduced density (rd) flux, with materials set to a fraction of their density
//!
//! These are combined to build up an estimate of the flux that can be expected
//! of a complete run with all materials in the geometry at their full density.

mod bude;
mod error;
mod magic;

#[doc(inline)]
pub use magic::{mesh_to_ww, mesh_to_ww_advanced};

#[doc(inline)]
pub use bude::extrapolate_density;

#[doc(inline)]
pub use error::Error;
