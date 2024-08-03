// internal modules
use crate::common::{Nuclide, RadType};
use crate::error::{Error, Result};
use crate::record::RecordSet;

// use bincode::serialize_into;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Load all nuclides from pre-fetched data
///
/// This will load all nuclides that were pre-processed into binary files from
/// the full IAEA chart of nuclides.
///
/// Specify the type of decay data with the appropriate [RadType] to get the
/// full dataset.
///
/// ```rust
/// # use ntools_iaea::{load_all, RadType, RecordSet};
/// # use std::collections::HashMap;
/// // Load all records for all pre-fetched nuclides
/// let decay_data: &HashMap<String, RecordSet> = load_all(RadType::Gamma);
/// ```
///
/// The returned hashmap is a dictionary of key value pairs where:
/// - **key** : Nuclide name, e.g. "Co60"
/// - **value** : List of all matching [Record](crate::Record)s
pub fn load_all(rad_type: RadType) -> &'static HashMap<String, RecordSet> {
    match rad_type {
        RadType::Alpha => load_alpha(),
        RadType::BetaPlus => load_betaplus(),
        RadType::BetaMinus => load_betaminus(),
        RadType::Electron => load_electron(),
        RadType::Xray => load_xray(),
        RadType::Gamma => load_gamma(),
    }
}

/// Load a list of nuclides available in pre-fetched data
///
/// Returns a list of [Nuclide]s consistent with the
/// [fetch_available()](crate::fetch_available) convenience function.
///
/// For example:
///
/// ```rust
/// # use ntools_iaea::{load_available, Nuclide, RadType};
/// /// Collect all available nuclides in the API
/// let nuclides: Vec<Nuclide> = load_available(RadType::Gamma).unwrap();
///
/// /// Print out the names
/// for nuclide in &nuclides {
///     println!("{}", nuclide.name());
/// }
/// ```
///
/// Which will output the full list of all pre-processed nuclides by name.
///
/// ```text
/// H1, H2, H3, ... Ts293, Ts294, Og294
/// ```
pub fn load_available(rad_type: RadType) -> Result<Vec<Nuclide>> {
    let data = load_all(rad_type);
    if data.is_empty() {
        return Err(Error::EmptyDataMap);
    }

    let mut nuclides: Vec<Nuclide> = data
        .keys()
        .filter_map(|k| Nuclide::try_from(k).ok())
        .collect();

    nuclides.sort();
    Ok(nuclides)
}

/// Load single nuclide from pre-fetched data
///
/// Retrieve the [RecordSet] for the specified nuclide. Will return `None` if
/// the nuclide is not found or contains no [Record](crate::Record)s for the
/// decay radiation type.  
///
/// Note this will accept a [Nuclide] or any `&str`, `String`, or `&String` that
/// will parse into a [Nuclide].
///
/// For example:
///
/// ```rust
/// # use ntools_iaea::{load_nuclide, Record, Nuclide, RadType};
/// # use ntools_utils::OptionExt;
/// // Try to get the records for Sodium-22
/// let records: Vec<Record> = load_nuclide("na22", RadType::Gamma).unwrap();
///
/// // Print the gamma energies
/// for r in records {
///     println!("{} keV", r.energy.display())
/// }
/// ```
///
/// ```text
/// 1274.537 keV
/// 511      keV
/// 0.848    keV
/// 0.848    keV
/// ```
///
/// For details of the data structure and associated convenience methods see the
/// [Record](crate::Record) type.
pub fn load_nuclide<N>(nuclide: N, rad_type: RadType) -> Option<RecordSet>
where
    N: TryInto<Nuclide> + Clone,
{
    load_nuclides(&[nuclide], rad_type)
        .values_mut()
        .next()
        .map(std::mem::take)
}

/// Load multiple nuclides from pre-fetched data
///
/// Retrieve the [RecordSet] for the specified nuclides. The returned hashmap is
/// a dictionary of key value pairs where:
///
/// - **key** : Nuclide name, e.g. "Co60"
/// - **value** : List of all matching [Record](crate::Record)s
///
/// Note this will accept a collection of [Nuclide]s or any `&str`, `String`, or
/// `&String` that will parse into a [Nuclide].
///
/// For example:
///
/// ```rust
/// # use ntools_iaea::{load_nuclides, Record, Nuclide, RadType};
/// # use ntools_utils::OptionExt;
/// // Try to get the records for Sodium-22 and Cesium-137
/// let nuclide_data = load_nuclides(&["na22", "cs137"], RadType::Gamma);
///
/// for (name, records) in nuclide_data {
///     println!("{name}:");
///     for r in records {
///         println!(" - {} keV", r.energy.display())
///     }
/// }
/// ```
///
/// ```text
/// Cs137:
///  - 283.5 keV
///  - 661.657 keV
///  - 4.966 keV
///  - 31.816 keV
///  - 32.193 keV
///  - 36.482 keV
///  - 36.827 keV
///  - 37.255 keV
/// Na22:
///  - 1274.537 keV
///  - 511 keV
///  - 0.848 keV
///  - 0.848 keV
/// ```
///
/// For details of the data structure and associated convenience methods see the
/// [Record](crate::Record) type.
pub fn load_nuclides<N>(nuclides: &[N], rad_type: RadType) -> HashMap<String, RecordSet>
where
    N: TryInto<Nuclide> + Clone,
{
    let nuclides = nuclides
        .iter()
        .cloned()
        .filter_map(|name| name.clone().try_into().ok())
        .collect::<Vec<Nuclide>>();

    let data = load_all(rad_type);

    nuclides
        .iter()
        .filter_map(|n| data.get_key_value(&n.name()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

// Only ever deserialise data once on first use, no sense doing it every time
static ALPHA: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();
static BETAPLUS: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();
static BETAMINUS: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();
static ELECTRON: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();
static XRAY: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();
static GAMMA: OnceLock<HashMap<String, RecordSet>> = OnceLock::new();

fn load_alpha() -> &'static HashMap<String, RecordSet> {
    ALPHA.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/alpha.bin"))
            .expect("unable to find pre-fetched alpha binary")
    })
}

fn load_betaplus() -> &'static HashMap<String, RecordSet> {
    BETAPLUS.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/betaplus.bin"))
            .expect("unable to find pre-fetched betaplus binary")
    })
}

fn load_betaminus() -> &'static HashMap<String, RecordSet> {
    BETAMINUS.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/betaminus.bin"))
            .expect("unable to find pre-fetched betaminus binary")
    })
}

fn load_electron() -> &'static HashMap<String, RecordSet> {
    ELECTRON.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/electron.bin"))
            .expect("unable to find pre-fetched electron binary")
    })
}

fn load_xray() -> &'static HashMap<String, RecordSet> {
    XRAY.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/xray.bin"))
            .expect("unable to find pre-fetched xray binary")
    })
}

fn load_gamma() -> &'static HashMap<String, RecordSet> {
    GAMMA.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/gamma.bin"))
            .expect("unable to find pre-fetched gamma binary")
    })
}
