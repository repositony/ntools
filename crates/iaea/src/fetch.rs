// standard library
use std::collections::HashMap;
use std::format as f;
use std::iter::zip;

// internal modules
use crate::common::{BaseNuclide, IsomerState, Nuclide, RadType};
use crate::error::{Error, Result};
use crate::record::{Record, RecordSet};
use crate::special::{Alpha, BetaMinus, BetaPlus, Electron, Gamma, SpecialData, Xray};

// external crates
use csv::Reader;
use csv::{self};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::prelude::*;

// use bincode::serialize_into;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Base of the URL ised to query the IAEA API
const IAEA_API: &str = "https://nds.iaea.org/relnsd/v1/data?";

// ! Public API

/// Fetch all nuclides direct from IAEA
///
/// <div class="warning">This will make ~3500 requests</div>
///
/// This will request and process all available nuclides from the chart of
/// nuclides. This is fully parallelised, but will still take a few minutes to
/// complete.
///
/// It is strongly recommended that [fetch_nuclide()] or [fetch_nuclides()] are
/// used instead for more targeted fetch requests.   
///
/// Failure to collect a list of all known nuclides will `panic` and exit the
/// program because this is crucial missing information.
///
/// Specify the type of decay data with the appropriate [RadType] to deserialise
/// the entire chart of nuclides.
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_all, RadType, RecordSet};
/// # use std::collections::HashMap;
/// // Get all records for all nuclides available in the chart of nuclides
/// let decay_data: HashMap<String, RecordSet> = fetch_all(RadType::Gamma);
/// ```
///
/// The returned hashmap is a dictionary of key value pairs where:
/// - **key** : Nuclide name, e.g. "Co60"
/// - **value** : List of all matching [Record]s
pub fn fetch_all(rad_type: RadType) -> HashMap<String, RecordSet> {
    let nuclides = fetch_available().expect("Unable to fetch list of available nuclides");

    let progress_style =
        ProgressStyle::with_template("Fetching nuclides: {pos}/{len} [{per_sec:.2}]").unwrap();

    let map = nuclides
        .par_iter()
        .progress_with_style(progress_style)
        .with_finish(indicatif::ProgressFinish::AndLeave)
        .cloned()
        .filter_map(|n| nuclide_tuple(n, rad_type))
        .collect::<HashMap<String, RecordSet>>();

    map
}

/// Fetch a list of known nuclides
///
/// This will query the IAEA chart of nuclides with
/// `fields=ground_states&nuclides=all` to find all known nuclides.
///
/// For example:
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_available, Nuclide, RadType};
/// /// Collect all available nuclides in the API
/// let nuclides: Vec<Nuclide> = fetch_available().unwrap();
///
/// /// Print out the names
/// for nuclide in &nuclides {
///     println!("{}", nuclide.name());
/// }
/// ```
///
/// Which will output the full list of all known nuclides by name.
///
/// ```text
/// H1, H2, H3, ... Ts293, Ts294, Og294
/// ```
pub fn fetch_available() -> Result<Vec<Nuclide>> {
    // send get request to IAEA
    let url = f!("{IAEA_API}fields=ground_states&nuclides=all");

    let csv = minreq::get(url).send()?;

    // build reader directly
    let mut reader = csv::ReaderBuilder::new()
        .quoting(false)
        .trim(csv::Trim::All)
        .from_reader(csv.as_bytes());

    // Need the first filter because the chart has entries for neutons as N1,
    // N4, and N6. These are easily confused for nitrogen isotopes
    Ok(reader
        .deserialize::<BaseNuclide>()
        .filter(|record| record.as_ref().is_ok_and(|r| r.z > 0))
        .filter_map(|record| {
            record.ok().map(|r| Nuclide {
                isotope: r.z + r.n,
                symbol: r.symbol,
                state: IsomerState::Ground,
            })
        })
        .collect::<Vec<Nuclide>>())
}

/// Fetch unmodified CSV data direct from IAEA
///
/// This will return the raw, unaltered CSV received from the IAEA as a String.
///
/// For example:
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_csv, RadType};
/// println!("{}", fetch_csv("Co60", RadType::Gamma).unwrap());
/// ```
///
/// This will return the gamma decay data for Cobalt-60. The CSV data are column
/// headings followed by a variable amount of records.
///
/// ```text
/// energy,unc_en,intensity,unc_i,start_level_energy,        |
/// end_level_energy,multipolarity, mixing_ratio,unc_mr,     |
/// conversion_coeff,unc_cc,p_z,p_n,p_symbol,p_energy_shift, |
/// p_energy,unc_pe,jp,half_life,operator_hl,unc_hl,unit_hl, | Headers
/// half_life_sec,unc_hls, decay,decay_%,unc_d,q,unc_q,d_z,  |
/// d_n,d_symbol,ensdf_publication_cut-off, ensdf_authors,   |
/// Extraction_date                                          |
///
/// 58.603,0.007,2.07,0.03,58.603,0.0,M3+(E4),0.02,LT,47.3,  |
/// ,27,33,Co,,58.59,0.01,2+,10.467,,6,m,628.02,0.36,IT,     | Record[0]
/// 99.75,0.03,,,27,33,Co,31-Dec-2012,E. BROWNE and J. K.    |
/// TULI,2024-03-18                                          |
///
/// ...
///
/// 8.296,,0.0013453257951684461,0.000023104962776745224,,   |
/// ,,,,,,27,33,Co,,0,,5+, 1925.28,,14,d,166344192,12096,    | Record[N]
/// B-,100,,2822.81,21,28,32,Ni,31-Dec-2012,E. BROWNE and    |
/// J. K. TULI,2024-03-18                                    |
/// ```
///
/// Use [fetch_nuclide()] or [fetch_nuclides()] to automatically deserialise
/// these data into sets of [Record]s.
pub fn fetch_csv<N>(nuclide: N, rad_type: RadType) -> Result<String>
where
    N: TryInto<Nuclide> + Clone,
    <N as TryInto<Nuclide>>::Error: std::fmt::Debug,
{
    let nuclide: Nuclide = nuclide
        .try_into()
        .map_err(|_| Error::FailedNuclideConversion)?;

    let url = &f!(
        "{IAEA_API}fields=decay_rads&nuclides={}&rad_types={}",
        nuclide.query_name()?,
        rad_type.query_symbol()
    );

    // send get request to IAEA
    let mut csv_text = minreq::get(url).send()?.as_str()?.to_string();

    // This is dumb, but duplicate fields are no good and both 'max_energy'
    // and 'mean_energy' have 'unc_me' for their uncertainties. Only affects
    // the beta- radiation data.
    if rad_type == RadType::BetaMinus {
        csv_text = csv_text.replacen("unc_me", "unc_mean", 1);
    }

    Ok(csv_text)
}

/// Fetch single nuclide direct from IAEA
///
/// This automatically deserialises the raw CSV data into [Record]s for the
/// requested nuclide and decay radiation type. Will return `None` if there is
/// any issue with the request or if it contains no [Record]s for the decay
/// radiation type.  
///
/// Note this will accept a [Nuclide] or any `&str`, `String`, or `&String` that
/// will parse into a [Nuclide].
///
/// Use [fetch_csv()] to inspect the raw CSV data returned form the IAEA.
///
/// For example:
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_nuclide, Record, Nuclide, RadType};
/// # use ntools_support::OptionExt;
/// // Try to get the records for Sodium-22
/// let records: Vec<Record> = fetch_nuclide("na22", RadType::Gamma).unwrap();
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
/// [Record] type.
pub fn fetch_nuclide<N>(nuclide: N, rad_type: RadType) -> Option<RecordSet>
where
    N: TryInto<Nuclide> + Clone,
    <N as TryInto<Nuclide>>::Error: std::fmt::Debug,
{
    fetch_nuclides(&[nuclide], rad_type)
        .values_mut()
        .next()
        .map(std::mem::take)
}

/// Fetch multiple nuclides direct from IAEA
///
/// This automatically deserialises the raw CSV data into [Record]s for the
/// decay radiation type and all requested nuclides. The returned hashmap is a
/// dictionary of key value pairs where:
///
/// - **key** : Nuclide name, e.g. "Co60"
/// - **value** : List of all matching [Record]sRadType::Alpha
///
/// Note this will accept a collection of [Nuclide]s or any `&str`, `String`, or
/// `&String` that will parse into a [Nuclide].
///
/// Use [fetch_csv()] to inspect the raw CSV data returned form the IAEA.
///
/// For example:
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_nuclides, Record, Nuclide, RadType};
/// # use ntools_support::OptionExt;
/// // Try to get the records for Sodium-22 and Cesium-137
/// let nuclide_data = fetch_nuclides(&["na22", "cs137"], RadType::Gamma);
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
/// [Record] type.
pub fn fetch_nuclides<N>(nuclides: &[N], rad_type: RadType) -> HashMap<String, RecordSet>
where
    N: TryInto<Nuclide> + Clone,
    <N as TryInto<Nuclide>>::Error: std::fmt::Debug,
{
    nuclides
        .iter()
        .cloned()
        .filter_map(|n| nuclide_tuple(n, rad_type))
        .collect()
}

/// Generate a binary file of pre-fetched data
///
/// Can be used to update the pre-processed data under the `data/` directory
/// that are used in all `load` operations.
///
/// This will use [fetch_all()] and is therefore parallelised, but may still
/// take up to a few minutes depending on radiation type.
///
/// ```rust, no_run
/// # use ntools_iaea::{prefetch_binary, RadType};
/// // Fetch all chart of nuclide data and store in a binary file
/// prefetch_binary("/path/to/file.bin", RadType::Gamma).unwrap();
/// ```
pub fn prefetch_binary<P: AsRef<Path>>(path: P, rad_type: RadType) -> Result<()> {
    let f = BufWriter::new(File::create(path)?);

    let data = fetch_all(rad_type);
    if data.is_empty() {
        return Err(Error::EmptyDataMap);
    }

    // write to binary file
    Ok(bincode::serialize_into(f, &data)?)
}

/// Generate a JSON file for pre-fetched data
///
/// This will use [fetch_all()] and is therefore parallelised, but may still
/// take up to a few minutes depending on radiation type.
///
/// The JSON format is human-readable and simple to parse, but will use much
/// more space on disk and be slower when retrieving records.
///
/// ```rust, no_run
/// # use ntools_iaea::{prefetch_json, RadType};
/// // Fetch all chart of nuclide data and store as JSON formatted file
/// prefetch_json("/path/to/file.json", RadType::Gamma).unwrap();
/// ```
pub fn prefetch_json<P: AsRef<Path>>(path: P, rad_type: RadType) -> Result<()> {
    let f = BufWriter::new(File::create(path)?);

    let data = fetch_all(rad_type);
    if data.is_empty() {
        return Err(Error::EmptyDataMap);
    }

    // write to json file
    Ok(serde_json::to_writer_pretty(f, &data)?)
}

// ! Private functions

/// Make a reader from a csv string slice
fn csv_reader(csv: &str) -> Reader<&[u8]> {
    csv::ReaderBuilder::new()
        .quoting(false)
        .trim(csv::Trim::All)
        .from_reader(csv.as_bytes())
}

/// Convert into a tuple for collection into a hashmap
fn nuclide_tuple<N>(nuclide: N, rad_type: RadType) -> Option<(String, RecordSet)>
where
    N: TryInto<Nuclide> + Clone,
    <N as TryInto<Nuclide>>::Error: std::fmt::Debug,
{
    if let Ok(records) = generate_recordset(nuclide.clone(), rad_type) {
        if records.is_empty() {
            None
        } else {
            let nuclide: Nuclide = nuclide.try_into().expect("Nuclide should have been fine");
            Some((nuclide.name(), records))
        }
    } else {
        None
    }
}

/// Actually generates the record set form the fetched csv
fn generate_recordset<N>(nuclide: N, rad_type: RadType) -> Result<RecordSet>
where
    N: TryInto<Nuclide> + Clone,
    <N as TryInto<Nuclide>>::Error: std::fmt::Debug,
{
    let csv_text = fetch_csv(nuclide, rad_type)?;
    deserialise_records(&csv_text, rad_type)
}

/// Deserialise record data from csv into [Record]
fn deserialise_records(csv_text: &str, rad_type: RadType) -> Result<Vec<Record>> {
    // deserialise the data into our own struct
    let mut decay_data: Vec<Record> = Vec::new();

    for common in csv_reader(csv_text).deserialize::<Record>() {
        decay_data.push(common.unwrap());
    }

    // Get the data special to this particular rad_type
    let special_data = deserialise_special_data(csv_text, rad_type, decay_data.len())?;

    // Update the records
    for (d, s) in zip(&mut decay_data, special_data) {
        d.special_data = s;
    }

    Ok(decay_data)
}

/// Deserialise special data from csv into [SpecialData] variants
fn deserialise_special_data(
    csv_text: &str,
    rad_type: RadType,
    n_records: usize,
) -> Result<Vec<SpecialData>> {
    let mut special_data: Vec<SpecialData> = Vec::with_capacity(n_records);

    // todo figure out how to clean up this ugly mess
    match rad_type {
        RadType::Alpha => {
            for s in csv_reader(csv_text).deserialize::<Alpha>() {
                special_data.push(s?.into())
            }
        }
        RadType::BetaPlus => {
            for s in csv_reader(csv_text).deserialize::<BetaPlus>() {
                special_data.push(s?.into())
            }
        }
        RadType::BetaMinus => {
            for s in csv_reader(csv_text).deserialize::<BetaMinus>() {
                special_data.push(s?.into())
            }
        }
        RadType::Gamma => {
            for s in csv_reader(csv_text).deserialize::<Gamma>() {
                special_data.push(s?.into())
            }
        }
        RadType::Electron => {
            for s in csv_reader(csv_text).deserialize::<Electron>() {
                special_data.push(s?.into())
            }
        }
        RadType::Xray => {
            for s in csv_reader(csv_text).deserialize::<Xray>() {
                special_data.push(s?.into())
            }
        }
    }

    Ok(special_data)
}
