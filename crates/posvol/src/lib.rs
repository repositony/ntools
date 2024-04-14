//! Module for working with UKAEA CuV posvol binaries
//! The 'posvol' binary generated from Cell-under-Voxel MCR2S runs lists the
//! dominant cell found inside each voxel (by volume).
//!
//! For example, a resolution of 5x5x5 on the CuV IDUM card will break every
//! voxel up into 125 regions, or `sub-voxel`s, and sample each one to find the
//! cell with the largest volume.
//!
//! This knowledge can be used for other things. For example, much finer
//! resolution VTK plots for CuV meshes.
//!
//! ## Read a posvol file
//!
//! To read the binary file simply provide the path.
//!
//! ```rust, no_run
//! # use ntools_posvol::read_posvol_file;
//! // Read the example file
//! let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
//!
//! // Print a summary of the data
//! println!("{posvol}");
//! ```  
//!
//! ## Write to other formats
//!
//! Any posvol file read into a [Posvol] may be written to a variety of file
//! formats. For example:
//!
//! ```rust, no_run
//! # use ntools_posvol::{write_ascii,write_ascii_pretty,write_json,read_posvol_file};
//! # let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
//! // Write binary data 1:1 into an ascii text file
//! write_ascii(&posvol, "./posvol_raw.txt");
//!
//! // Write a human readable ascii text file
//! write_ascii_pretty(&posvol, "./posvol_pretty.txt");
//!
//! // Dump the [Posvol] into a JSON file
//! write_json(&posvol, "./posvol_json.json");
//! ```

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
