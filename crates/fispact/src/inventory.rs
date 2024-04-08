use crate::interval::Interval;

use serde::{Deserialize, Serialize};

/// Metadata for the run
#[derive(Serialize, Deserialize, Debug)]
pub struct RunData {
    /// User defined run title
    run_name: String,
    /// User defined flux comment
    flux_name: String,
    /// Timestamp for inventory calculation
    timestamp: String,
}

/// Top level inventory data
///
/// This is a simple collection of the run metadata and list of every [Interval]
/// result.
///
/// A lot of useful functionality is implemented at this level for investigating
/// properties across intervals and collecting broader summaries of data.  
#[derive(Serialize, Deserialize, Debug)]
pub struct Inventory {
    /// Collection of all run intervals
    #[serde(rename = "inventory_data")]
    intervals: Vec<Interval>,
    /// Metadata for the run
    run_data: RunData,
}

impl Inventory {
    /// Collection of total activity (Bq) for each [Interval]
    pub fn activity_list() {
        todo!()
    }

    /// Collection of total [Dose] for each [Interval]
    pub fn dose_list() {
        todo!()
    }

    /// Collection of total sample mass (g) for each [Interval]
    pub fn mass_list() {
        todo!()
    }

    /// Collection of total time (s) for each [Interval]
    pub fn total_times() {
        todo!()
    }

    /// List of any matching [Nuclide] objects throughout the [Inventory]
    pub fn nuclides() {
        todo!()
    }

    /// List of the elements seen in the [Inventory]
    pub fn elements() {
        todo!()
    }

    /// Collection of nuclide names
    pub fn nuclide_names() {
        todo!()
    }

    /// List of data for some time dependednt transient
    pub fn nuclide_transient() {
        todo!()
    }

    /// Finds the nearest [Interval] by total time
    pub fn nearest_interval() {
        todo!()
    }

    /// Applies a flux normalisation to all data in the [Inventory]
    pub fn normalise_flux() {
        todo!()
    }
}
