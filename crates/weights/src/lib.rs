//! Tools for MCNP weight window operations
//!
#![doc = include_str!("../readme.md")]

mod error;
mod operations;
pub mod vtk;
mod weight_window;

#[doc(inline)]
pub use crate::weight_window::WeightWindow;

#[doc(inline)]
pub use crate::error::{Error, Result};

#[doc(inline)]
pub use crate::operations::{write_multi_particle, write_single_particle};
