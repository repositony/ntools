//! Module for reading MCNP MCTAL files
//!
//! The MCNP MCTAL file contains all tally results of one dump of a RUNTAPE file
//! in a relatively fixed-format*.
//!
//! - [Mctal] - Primary data structure containing the parsed file data
//!
//! There are many data collections available, but can be broken into one of the
//! following blocks:
//!
//! | Data block | Description                                                  |
//! | ---------- | ------------------------------------------------------------ |
//! | [Header]   | file metadata and run information                            |
//! | [Tally]    | all standard `F` tallies, including tally fluctuation chart  |
//! | [Tmesh]    | superimposed Mesh Tally Type A (a.k.a TMESH)                 |
//! | [Kcode]    | criticality results (KCODE)                                  |
//!
//! Note that `TMESH` mesh tallies are written to the MCTAL file, while `FMESH`
//! mesh tallies are not. Tools for reading `FMESH` data are available in other
//! ntools crates (See [mesh](https://repositony.github.io/ntools/ntools_mesh/index.html)).
//!
//! \* *Note: numerical items do not need to be in the columns implied by
//! fortran formats, only blank-delimited and in  the right order*
//!
//! # Quickstart example
//!
//! Suppose there is a tally for the following:
//!
//! ```text
//! fc104   Example simple flux tally
//! f104:n  901 902 903     $ cells to tally
//! e104    1e-5 1.0 1e3    $ energy bins
//! ```
//!
//! ```rust, no_run
//! # use ntools_mctal::Mctal;
//! // Read all file data into the core data structure
//! let mctal = Mctal::from_file("/path/to/file.m").unwrap();
//!
//! // Find the data for `F104:N`
//! let tally = mctal.get_tally(104).expect("Tally 104 not found");
//!
//! // Do whatever you want with the information...
//! ```
#![allow(warnings)]

mod core;
mod error;
mod mctal;
mod parsers;
mod reader;

// flatten public API and inline the documentation
#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use mctal::Mctal;

#[doc(inline)]
pub use core::*;
