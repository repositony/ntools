//! Analysis tools for FISPACT-II inventory calculations
//!
#![doc = include_str!("../readme.md")]

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
pub use error::{Error, Result};

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
