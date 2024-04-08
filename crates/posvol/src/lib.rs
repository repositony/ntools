//! Module for working with UKAEA CuV posvol binaries
//!
#![doc = include_str!("../readme.md")]

// Split into subfiles for development, but anything important is re-exported
mod error;
mod posvol;
mod reader;
mod writer;

// Inline anything important for a nice public API
#[doc(inline)]
pub use posvol::{Dimensions, Posvol};

#[doc(inline)]
pub use reader::read_posvol_file;

#[doc(inline)]
pub use writer::{write_ascii, write_ascii_pretty, write_json};

#[doc(inline)]
pub use error::{Error, Result};
