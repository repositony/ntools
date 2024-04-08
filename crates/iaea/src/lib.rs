//! Query decay data directly from the IAEA chart of nuclides
//!
#![doc = include_str!("../readme.md")]

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
