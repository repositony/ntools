//! Query decay data directly from the IAEA chart of nuclides
//!
//! This crate is intended to make using decay data from the IAEA chart of nuclides
//! API slightly less painful.
//!
//! Data may be fetched directly from the IAEA for the latest data if an internet
//! connection is available, or read from pre-fetched data (recommended).
//!
//! Fetch requests for large numbers of nuclides are parallelised.
//!
//! See <https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html> for further
//! information.
//!
//! ## Implementation
//!
//! The structure remains relatively consistent with the CSV data returned from the
//! IAEA.
//!
//! Every nuclide request returns a series of [Record]s. A coollection of these
//! records is aliased to [RecordSet].
//!
//! For example:
//!
//! ```rust, no_run
//! # use ntools_iaea::{Record,fetch_nuclide_records, RadType};
//! // Get all records for the Cobalt-60 nuclide
//! let cobalt60: Vec<Record> = fetch_nuclide_records("co60", RadType::Gamma).unwrap();
//!
//! // Find the 1173 keV emission as an example
//! let example = cobalt60
//!     .iter()
//!     .find(|record| record.energy.unwrap() == 1173.228)
//!     .unwrap();
//!
//! // Print a summary of the record
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
pub use error::{Error, Result};

#[doc(inline)]
pub use record::{Record, RecordSet};

#[doc(inline)]
pub use load::{decode_binary_file, load_all_data, load_nuclide_records};

#[doc(inline)]
pub use fetch::{
    fetch_all_data, fetch_available_nuclides, fetch_nuclide_records, fetch_raw_csv,
    prefetch_binary, prefetch_json,
};

#[doc(inline)]
pub use special::{Alpha, BetaMinus, BetaPlus, Electron, Gamma, SpecialData, Xray};
