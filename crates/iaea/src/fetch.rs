// standard library
use std::format as f;
use std::iter::zip;
use std::str::FromStr;

// internal modules
use crate::common::{BaseNuclide, IsomerState, Nuclide, RadType};
use crate::error::Result;
use crate::record::{Record, RecordSet};
use crate::special::{Alpha, BetaMinus, BetaPlus, Electron, Gamma, SpecialData, Xray};

// external crates
use csv::Reader;
use csv::{self};
use kdam::par_tqdm;
use rayon::prelude::*;

// use bincode::serialize_into;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Base of the URL ised to query the IAEA API
const IAEA_API: &str = "https://nds.iaea.org/relnsd/v1/data?";

// ! Public API

/// Raw CSV data directly from IAEA API
///
/// This will return the raw, unaltered CSV received from the IAEA as a String.
///
/// For example:
/// ```rust, no_run
/// # use ntools_iaea::fetch_raw_csv;
/// # use ntools_iaea::RadType;
/// println!("{}", fetch_raw_csv("co60", RadType::Gamma).unwrap());
/// ```
///
/// will provide the headings followed by any data records
///
/// ```text
/// energy,unc_en,intensity,unc_i,start_level_energy,end_level_energy,multipolarity,
/// mixing_ratio,unc_mr,conversion_coeff,unc_cc,p_z,p_n,p_symbol,p_energy_shift,
/// p_energy,unc_pe,jp,half_life,operator_hl,unc_hl,unit_hl,half_life_sec,unc_hls,
/// decay,decay_%,unc_d,q,unc_q,d_z,d_n,d_symbol,ensdf_publication_cut-off,
/// ensdf_authors,Extraction_date
///
/// 58.603,0.007,2.07,0.03,58.603,0.0,M3+(E4),0.02,LT,47.3,,27,33,Co,,58.59,0.01,
/// 2+,10.467,,6,m,628.02,0.36,IT,99.75,0.03,,,27,33,Co,31-Dec-2012,E. BROWNE and  
/// J. K. TULI,2024-03-18
///
/// ...
///
/// 8.296,,0.0013453257951684461,0.000023104962776745224,,,,,,,,27,33,Co,,0,,5+,
/// 1925.28,,14,d,166344192,12096,B-,100,,2822.81,21,28,32,Ni,31-Dec-2012,E.
/// BROWNE and  J. K. TULI,2024-03-18
/// ```
///
/// For getting something usable, use [fetch_nuclide_records()] instead to
/// deserialise the rows into [Record]s.
pub fn fetch_raw_csv(nuclide: &str, rad_type: RadType) -> Result<String> {
    let nuclide = Nuclide::from_str(nuclide)?;

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

/// Record objects directly from IAEA API
///
/// The IAEA provides CSV data when queried, which can be seen using the
/// [fetch_raw_csv()] function.
///
/// This function deserialises the CSV data into [Record]s for the requested
/// nuclide and decay radiation type.
///
/// ```rust, no_run
/// # use ntools_iaea::fetch_nuclide_records;
/// # use ntools_iaea::RadType;
/// # use ntools_iaea::Record;
/// // Get all records for the Cobalt-60 nuclide
/// let co60_records: Vec<Record> = fetch_nuclide_records("co60", RadType::Gamma).unwrap();
/// ```
///
/// For details of the data structure and associated convenience methods see the
/// [Record] type.
pub fn fetch_nuclide_records(nuclide: &str, rad_type: RadType) -> Result<RecordSet> {
    // Fetch the raw csv data from webhook
    let csv_text = fetch_raw_csv(nuclide, rad_type)?;

    // deserialise the data into our own struct
    deserialise_records(&csv_text, rad_type)
}

/// Record objects directly from IAEA API for every single nuclide
///
/// This will request ~3500 nuclides and is parallelised, but will still take a
/// few minutes to complete.
///
/// It is strongly recommended that [fetch_nuclide_records()] is used instead
/// unless data for the entire chart of nuclides is required.   
///
/// ```rust, no_run
/// # use ntools_iaea::fetch_all_data;
/// # use ntools_iaea::RadType;
/// # use ntools_iaea::RecordSet;
/// // Get all records for all nuclides available in the chart of nuclides
/// let all_data: Vec<RecordSet> = fetch_all_data(RadType::Gamma);
/// ```
///
/// Failed requests result in a `panic!` for now, but this should perhaps be
/// a warning instead.
pub fn fetch_all_data(rad_type: RadType) -> Vec<RecordSet> {
    let nuclides = fetch_available_nuclides().expect("Unable to fetch list of available nuclides");

    // let nuclides = &nuclides[0..30];

    // todo, make this a filter map, no reason to panic really just raise a warning
    let records = par_tqdm!(
        nuclides.par_iter().map(|n| {
            fetch_nuclide_records(&n.name(), rad_type)
                .unwrap_or_else(|_| panic!("Unable to read records for \"{}\"", n.name()))
        }),
        bar_format = "Fetching nuclides: {count}/{total} [{rate:.2} nuc/s]  "
    )
    .collect();
    eprintln!();
    records
}

/// Collect list of every nuclide accessible via the IAEA API
///
/// ```rust, no_run
/// # use ntools_iaea::fetch_available_nuclides;
/// # use ntools_iaea::RadType;
/// // Get all records for the Cobalt-60 nuclide
/// let nuclides = fetch_available_nuclides().unwrap();
/// ```
///
/// which will give the names of all available ground state nuclides
///
/// ```text
/// H1, H2, H3, ... Ts293, Ts294, Og294
/// ```
pub fn fetch_available_nuclides() -> Result<Vec<Nuclide>> {
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

/// Generate a binary file of pre-fetched data
///
/// This will take up to a couple of minutes but there is a progress counter.
/// todo should parallelise this, the order does not matter
pub fn prefetch_binary<P: AsRef<Path>>(path: P, rad_type: RadType) -> Result<()> {
    let f = BufWriter::new(File::create(path)?);

    let a: Vec<RecordSet> = fetch_all_data(rad_type)
        .into_iter()
        .filter(|n| !n.is_empty())
        .collect();

    // write to binary file
    Ok(bincode::serialize_into(f, &a)?)
}

/// Generate a JSON file for pre-fetched data
pub fn prefetch_json<P: AsRef<Path>>(path: P, rad_type: RadType) -> Result<()> {
    let f = BufWriter::new(File::create(path)?);

    // generate a map and then write the lot?
    let a: Vec<RecordSet> = fetch_all_data(rad_type)
        .into_iter()
        .filter(|n| !n.is_empty())
        .collect();

    // write to json file
    Ok(serde_json::to_writer_pretty(f, &a)?)
}

// ! Private functions

fn csv_reader(csv_text: &str) -> Reader<&[u8]> {
    csv::ReaderBuilder::new()
        .quoting(false)
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes())
}

fn deserialise_records(csv_text: &str, rad_type: RadType) -> Result<Vec<crate::record::Record>> {
    // deserialise the data into our own struct
    let mut decay_data: Vec<crate::record::Record> = Vec::new();

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

// todo figure out how to clean up this ugly mess
fn deserialise_special_data(
    csv_text: &str,
    rad_type: RadType,
    n_records: usize,
) -> Result<Vec<SpecialData>> {
    let mut special_data: Vec<SpecialData> = Vec::with_capacity(n_records);

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
