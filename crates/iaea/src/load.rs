// internal modules
use crate::common::RadType;
use crate::error::Result;
use crate::record::RecordSet;
use crate::Error;

// use bincode::serialize_into;
use std::fs::File;
use std::path::Path;
use std::sync::OnceLock;

/// Decode the binary file back into [Record]s
///
/// The [prefetch_binary()](crate::prefetch_binary) will produce a binary file
/// of all decay data for a particular radiation type.
///
/// This function decodes the file content  back into records
pub fn decode_binary_file<P: AsRef<Path>>(path: P) -> Result<Vec<RecordSet>> {
    let f = std::io::BufReader::new(File::open(path)?);
    let decoded = bincode::deserialize_from(f)?;
    Ok(decoded)
}

/// Load nuclide records from pre-fetched data
pub fn load_all_data(rad_type: RadType) -> &'static Vec<RecordSet> {
    match rad_type {
        RadType::Alpha => load_alpha(),
        RadType::BetaPlus => load_betaplus(),
        RadType::BetaMinus => load_betaminus(),
        RadType::Electron => load_electron(),
        RadType::Xray => load_xray(),
        RadType::Gamma => load_gamma(),
    }
}

/// Find nuclide records using pre-fetched data
pub fn load_nuclide_records(nuclide: &str, rad_type: RadType) -> Result<RecordSet> {
    load_all_data(rad_type)
        .iter()
        .find(|nuc| nuc.first().unwrap().parent_name().to_lowercase() == nuclide.to_lowercase())
        .cloned()
        .ok_or_else(|| Error::FailedToLoad {
            nuclide: nuclide.to_string(),
            rad_type,
        })
}

// fn load_by_nuclide(nuclide: &Nuclide, rad_type: RadType) -> Result<RecordSet> {}

// Only ever deserialise data once on first use, no sense doing it every time
static ALPHA: OnceLock<Vec<RecordSet>> = OnceLock::new();
static BETAPLUS: OnceLock<Vec<RecordSet>> = OnceLock::new();
static BETAMINUS: OnceLock<Vec<RecordSet>> = OnceLock::new();
static ELECTRON: OnceLock<Vec<RecordSet>> = OnceLock::new();
static XRAY: OnceLock<Vec<RecordSet>> = OnceLock::new();
static GAMMA: OnceLock<Vec<RecordSet>> = OnceLock::new();

fn load_alpha() -> &'static Vec<RecordSet> {
    ALPHA.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/alpha.bin"))
            .expect("unable to find pre-fetched alpha binary")
    })
}

fn load_betaplus() -> &'static Vec<RecordSet> {
    BETAPLUS.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/betaplus.bin"))
            .expect("unable to find pre-fetched betaplus binary")
    })
}

fn load_betaminus() -> &'static Vec<RecordSet> {
    BETAMINUS.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/betaminus.bin"))
            .expect("unable to find pre-fetched betaminus binary")
    })
}

fn load_electron() -> &'static Vec<RecordSet> {
    ELECTRON.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/electron.bin"))
            .expect("unable to find pre-fetched electron binary")
    })
}

fn load_xray() -> &'static Vec<RecordSet> {
    XRAY.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/xray.bin"))
            .expect("unable to find pre-fetched xray binary")
    })
}

fn load_gamma() -> &'static Vec<RecordSet> {
    GAMMA.get_or_init(|| {
        bincode::deserialize(include_bytes!("../data/gamma.bin"))
            .expect("unable to find pre-fetched gamma binary")
    })
}
