//! Query decay data directly from the IAEA chart of nuclides
//!
//! This crate is intended to make using decay data from the IAEA chart of
//! nuclides API slightly less painful.
//!
//! Decay data are made available by either:
//! - Loading pre-fetched data (recommended)
//! - Fetching directly from the API if an internet connection is available
//!
//! Fetch requests for large numbers of nuclides are parallelised.
//!
//! See <https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html> for
//! further information.
//!
//! ## Implementation
//!
//! The structure remains relatively consistent with the CSV data returned from
//! the IAEA.
//!
//! Every nuclide request returns a series of [Record]s. A collection of these
//! records is called a [RecordSet].
//!
//! For example:
//!
//! ```rust, no_run
//! # use ntools_iaea::{Record, fetch_nuclide, RadType, RecordSet};
//! // Get all records for the Cobalt-60 nuclide
//! let cobalt60: RecordSet = fetch_nuclide("co60", RadType::Gamma).unwrap();
//!
//! // Find the 1173 keV emission as an example
//! let example: &Record = cobalt60
//!     .iter()
//!     .find(|record| record.energy.unwrap() == 1173.228)
//!     .unwrap();
//!
//! // Print a summary of this record
//! println!("{example}");
//! ```
//!
//! This will contain all common decay data fields under the [Record], and any
//! special data unique to the radiation type. In this case, the [Gamma] data.
//!
//! ```text
//! Record
//!   Energy           1173.228 +/- 0.003 keV
//!   Intensity        99.85 +/- 0.03 %
//!   Half life        166344200 +/- 12096 s
//!   Decay mode       B-
//!   Branching        100 +/- None %
//!   State            5+
//!   Q value          2822.81 +/- 21 keV
//!   Parent symbol    Co
//!   Parent z         27
//!   Parent n         33
//!   Parent energy    0 +/- None keV
//!   Daughter symbol  Ni
//!   Daughter z       28
//!   Daughter n       32
//! Gamma
//!   Start level      2505.748 keV
//!   End level        1332.508 keV
//!   Multipolarity    E2(+M3)
//!   Mixing ratio     -0.0025 +/- 22
//!   Conversion coef. 0.0001722 +/- None
//! ```

// Modules
mod common;
mod error;
mod fetch;
mod load;
mod parsers;
mod record;
mod special;

// Re-exports of anything important with in-lined documentation for simplicity
#[doc(inline)]
pub use common::{IsomerState, Nuclide, RadType};

#[doc(inline)]
pub use error::Error;

#[doc(inline)]
pub use record::{Record, RecordSet};

#[doc(inline)]
pub use load::{load_all, load_available, load_nuclide, load_nuclides};

#[doc(inline)]
pub use fetch::{
    fetch_all, fetch_available, fetch_csv, fetch_nuclide, fetch_nuclides, prefetch_binary,
    prefetch_json,
};

#[doc(inline)]
pub use special::{Alpha, BetaMinus, BetaPlus, Electron, Gamma, SpecialData, Xray};
