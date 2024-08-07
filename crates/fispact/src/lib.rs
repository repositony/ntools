//! Analysis tools for FISPACT-II inventory calculations
//!
//! The `fispact` module contains useful utilities for quickly processing
//! FISPACT-II outputs.
//!
//! Currently the JSON output is fully deserialised to useful structures. The
//! TAB and legacy output files will be supported as needed.
//!
//! # Quickstart example
//!
//! To quickly load a JSON inventory into structures for further analysis, this
//! is a simple read.
//!
//! ```rust, no_run
//! # use ntools_fispact::{Inventory,read_json};
//! // Read the JSON contents of the file as an instance of `Inventory`.
//! let inventory: Inventory = read_json("path/to/results.json").unwrap();
//! ```
//!
//! # Core concepts
//!
//! The library is structured much like the output files for simplicity.
//!
//! - [Inventory] - all data deserialised from the JSON
//!     - [RunData] - metadata about the run
//!     - [Interval]s - list of intervals for every step of the calculation
//!         - [Nuclide]s - list of nuclides and their associated data
//!         - [Spectrum] - basic histogram of gamma lines for FISPACT-II v5.
//!
//! ## Important name changes
//!
//! Several tweaks were made to the deserialiser and data structures are not
//! entirely a 1:1 match for the JSON dictionary.
//!
//! #### Changes to sample mass
//!
//! The masses are inconsistent between nuclides and the interval, often leading
//! to//! people forgetting to change the units to and from grams or kilograms.
//!
//! All masses are **converted to grams** in the background such that [Nuclide]
//! and [Interval] mass values are consistent.
//!
//! #### Changes to sample dose
//!
//! The sample dose rate for the interval is converted into something more
//! concise that works better with the type system.
//!
//! For example:
//!
//! ```json
//! "dose_rate": {
//!     "type": "Point source",
//!     "distance": 1.0,
//!     "mass": 1.0,
//!     "dose": 1.0
//! }
//! ```
//!
//! This is tuned into a [Dose] of type [DoseKind]. The mass is redundant as
//! either the mass of the sample or irrelevant for a contact dose, so it is
//! dropped.
//!
//! ```rust, no_run
//! struct Dose {
//!     /// Dose rate (Sv/hr)
//!     rate: f64,
//!     /// Type of dose
//!     kind: DoseKind,
//! }
//!
//! enum DoseKind {
//!     /// Semi-infinite slab approximation
//!     Contact,
//!     /// Point source approximation at contained distance (m)
//!     Point(f64),
//! }
//! ```
//!
//! #### Changes to sample totals
//!
//! Several of the main sample totals have been renamed for brevity throughout.
//!
//! For example:
//!
//! ```json
//! "inventory_data": [
//!     {
//!         "total_atoms": 1.0,
//!         "total_activity": 1.0,
//!         "total_mass": 1.0,
//!         "total_heat": 1.0,
//!     }
//! ]
//! ```
//!
//! becomes:
//!
//! ```rust, no_run
//! struct Interval {
//!     atoms: f64,
//!     activity: f64,
//!     mass: f64,
//!     heat: f64,
//! }
//! ```

mod error;
mod interval;
mod inventory;
mod nuclide;

#[doc(inline)]
pub use interval::{Dose, DoseKind, Interval, Spectrum};

#[doc(inline)]
pub use nuclide::{Nuclide, Stability};

#[doc(inline)]
pub use inventory::{Inventory, RunData};

#[doc(inline)]
pub use error::Error;

use error::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Read the full JSON inventory results
///
/// The `path` takes anything that can be turned into a `Path` reference, such
/// as a [&str], [String], [Path], etc...
///
/// Returns a result containing the full [Inventory] data with every [Interval].
///
/// Example
/// ```rust, no_run
/// # use ntools_fispact::{read_json, Inventory};
/// // Read the JSON inventory data
/// let inventory: Inventory = read_json("path/to/results.json").unwrap();
/// ```
pub fn read_json<P: AsRef<Path>>(path: P) -> Result<Inventory> {
    let path: &Path = Path::new(path.as_ref());
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

/// Sortable properties for [Nuclide]s and [Interval]s
#[derive(Debug, Clone, Copy)]
pub enum SortProperty {
    Activity,
    Mass,
    Dose,
    Atoms,
    Heat,
}
