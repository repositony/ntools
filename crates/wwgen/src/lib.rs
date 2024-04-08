//! Weight window generation methods for MCNP
//!
#![doc = include_str!("../readme.md")]

mod bude;
mod error;
mod magic;

#[doc(inline)]
pub use magic::{mesh_to_ww, mesh_to_ww_advanced};

#[doc(inline)]
pub use bude::extrapolate_density;
