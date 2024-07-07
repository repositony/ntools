//! Tools for MCNP weight window operations
//!
//! This module is designed for the core weight window structures defined by
//! MCNP.
//!
//! The main [WeightWindow] type represents a set of weight windows for a single
//! mesh and particle type.
//!
//! ## MCNP formats
//!
//! Writers are implemented to generate the standardised UTF-8 file format for
//! WWINP/WWOUT/WWONE files for direct use in MCNP simulations.
//!
//! Details may be found in the Appendicies of the user manuals:
//!
//! - [MCNPv6.2](https://mcnp.lanl.gov/pdf_files/TechReport_2017_LANL_LA-UR-17-29981_WernerArmstrongEtAl.pdf)
//! - [MCNPv6.3](https://mcnpx.lanl.gov/pdf_files/TechReport_2022_LANL_LA-UR-22-30006Rev.1_KuleszaAdamsEtAl.pdf)
//!
//! For example,
//!
//! ```rust, no_run
//! # use ntools_weights::WeightWindow;
//! // Make a weight window set
//! let ww = WeightWindow::default();
//!
//! // Write to standard fromatted UTF-8 file
//! ww.write("/path/to/wwout");
//! ```
//!
//! For combining multiple particle types and meshes into a single file, see the
//! `write_multi_particle()` convenience function.
//!
//! ```rust, no_run
//! # use ntools_weights::{WeightWindow, write_multi_particle};
//! let neutron = WeightWindow {
//!     particle: 1,                    // Particle::Neutron
//!     ..Default::default()
//! };
//!
//! let photon = WeightWindow {
//!     particle: 2,                    // Particle::Photon
//!     ..Default::default()
//! };
//!
//! // Write a combined NP weight window file
//! let ww_sets = [photon, neutron];
//! let weight_window = write_multi_particle(&ww_sets, "wwout_NP", false);
//! ```
//!
//! ## Visualisation
//!
//! The weights may also be written out to a Visual Toolkit files using the [vtk]
//! module.
//!
//! ```rust, no_run
//! # use ntools_weights::WeightWindow;
//! # use ntools_weights::vtk::{VtkFormat,weights_to_vtk, write_vtk};
//! // Convert to VTK with the default configuration
//! let vtk = weights_to_vtk(&WeightWindow::default());
//!
//! // Wite the VTK to a file in one of several formats
//! write_vtk(vtk, "output.vtk", VtkFormat::Xml).unwrap();
//! ```
//!
//! For more details and advanced use see the vtk module documentation.

mod error;
mod operations;
pub mod vtk;
mod weight_window;

#[doc(inline)]
pub use crate::weight_window::WeightWindow;

#[doc(inline)]
pub use crate::error::Error;

#[doc(inline)]
pub use crate::operations::{write_multi_particle, write_single_particle};
