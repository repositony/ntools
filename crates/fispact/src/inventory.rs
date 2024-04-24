use crate::{Dose, Interval, Nuclide};

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
    pub fn activity_list(&self) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| interval.activity)
            .collect()
    }

    /// Collection of sample specific activity (Bq/g) for each [Interval]
    pub fn specific_activity_list(&self) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| interval.activity / interval.mass)
            .collect()
    }

    /// Collection of total [Dose] rates for each [Interval]
    pub fn dose_list(&self) -> Vec<Dose> {
        self.intervals
            .iter()
            .map(|interval| interval.dose)
            .collect()
    }

    /// Collection of total sample mass (g) for each [Interval]
    pub fn mass_list(&self) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| interval.mass)
            .collect()
    }

    // todo make time a type for conversions, etc...
    /// Collection of total time (s) for each [Interval]
    pub fn total_times(&self) -> Vec<f64> {
        self.intervals
            .iter()
            .map(|interval| interval.irradiation_time + interval.cooling_time)
            .collect()
    }

    /// List of any matching [Nuclide] objects throughout the [Inventory]
    pub fn nuclides(&self) -> Vec<Nuclide> {
        self.intervals
            .iter()
            .flat_map(|interval| interval.nuclides.clone())
            .collect()
    }

    /// List of names for all unique elements in the [Inventory]
    pub fn element_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .intervals
            .iter()
            .flat_map(|interval| interval.element_names())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    /// List of names for all unique nuclides in the [Inventory]
    pub fn nuclide_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .intervals
            .iter()
            .flat_map(|interval| interval.nuclide_names())
            .collect();
        names.sort();
        names.dedup();
        names
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
