use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::SpecialData;
use ntools_format::f;
use ntools_format::OptionFormat;

/// Type alias for `Vec<Record>`
pub type RecordSet = Vec<Record>;

/// Data for a single record from an IAEA fetch request
///
/// This is almost a mirror of the `decay_rads` data from the IAEA chart of
/// nuclides API (see the
/// [guide](https://www-nds.iaea.org/relnsd/vcharthtml/api_v0_guide.html)).
///
/// ## Why Option?
///
/// The guide is frustratingly inconsistent with the data you actually recieve
/// and unfortunately every CSV field appears optional.
///
/// For absolute transparency the data remain the standard [Option] type
/// indicating the presence of either `Some` where an entry is valid, or `None`
/// for blank empty fields.
///
/// ## Special decay data
///
/// Any `decay_rads` request to the IAEA chart of nuclides returns the common
/// fields seen in a [Record] regardless of radiation type.
///
/// Every decay radiation type will also have additional specialised data. To
/// keep everything under a common and simple interface, every [Record] stores
/// these data undert the `special_data` field.
///
/// The table below is a reference of all supported types but should be fairly
/// intuitive.
///
/// | Decay radiation type | IAEA symbol | [SpecialData](crate::special::SpecialData) variant |
/// | ---------- | ----------- | -------------------------------------- |
/// | alpha      | a           | [Alpha](crate::special::Alpha)         |
/// | beta plus  | bp          | [BetaPlus](crate::special::BetaPlus)   |
/// | beta minus | bm          | [BetaMinus](crate::special::BetaMinus) |
/// | gamma      | g           | [Gamma](crate::special::Gamma)         |
/// | electron   | e           | [Electron](crate::special::Electron)   |
/// | x-ray      | x           | [Xray](crate::special::Xray)           |
///
/// ## Examples
///
/// ### Basic use
///
/// Fetching the data directly from the IAEA API will always use a `fetch_*`
/// function.
///
/// For example, the cobalt-60 decay data for gamma emissions:  
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_nuclide_records, RadType};
/// // Get all records for the Cobalt-60 nuclide
/// let co60_records = fetch_nuclide_records("co60", RadType::Gamma).unwrap();
///
/// // Find the 1173 keV emission as an example
/// let example = co60_records
///         .iter()
///         .find(|record| record.energy.unwrap() == 1173.228)
///         .unwrap();
///
/// // Print a summary of the record
/// println!("{example}");
/// ```
///
/// The output of which will be the following data. Note the use of an explicit
/// `None` for any blank entires recieved from the IAEA data.
///
/// ```text
/// Record
///    --- Common decay_rads data ---
///    Energy           1173.228 +/- 0.003 keV
///    Intensity        99.85 +/- 0.03 %
///    Half life        166344200 +/- 12096 s
///    Decay mode       B-
///    Branching        100 +/- None %
///    State            5+
///    Q value          2822.81 +/- 21 keV
///    Parent symbol    Co
///    Parent z         27
///    Parent n         33
///    Parent energy    0 +/- None keV
///    Daughter symbol  Ni
///    Daughter z       28
///    Daughter n       32
///    --- Gamma special data ---
///    Start level      2505.748 keV
///    End level        1332.508 keV
///    Multipolarity    E2(+M3)
///    Mixing ratio     -0.0025 +/- 22
///    Conversion coef. 0.0001722 +/- None
/// ```
///
/// ### Other formatting options
///
/// This can be written to JSON formats fairly easily, continuing on from the
/// previous example:
///
/// ```rust, no_run
/// # use ntools_iaea::{fetch_nuclide_records, RadType};
/// # let co60_records = fetch_nuclide_records("co60", RadType::Gamma).unwrap();
/// # let example = co60_records
/// #        .iter()
/// #        .find(|record| record.energy.unwrap() == 1173.228)
/// #        .unwrap();
/// // Print a JSON string representation of the record
/// println!("{}", example.to_json().unwrap());
/// ```
///
/// which will result in
///
/// ```json
/// {
///   "energy": 1173.228,
///   "unc_en": 0.003,
///   "intensity": 99.85,
///   "unc_i": 0.03,
///   "half_life_sec": 166344200.0,
///   "unc_hls": 12096.0,
///   "decay": "B-",
///   "decay_%": 100.0,
///   "unc_d": null,
///   "jp": "5+",
///   "q": "2822.81",
///   "unc_q": "21",
///   "p_symbol": "Co",
///   "p_z": 27,
///   "p_n": 33,
///   "p_energy": 0.0,
///   "unc_pe": null,
///   "d_symbol": "Ni",
///   "d_z": 28,
///   "d_n": 32
///   "Gamma": {
///     "start_level_energy": 2505.748,
///     "end_level_energy": 1332.508,
///     "multipolarity": "E2(+M3)",
///     "mixing_ratio": -0.0025,
///     "unc_mr": 22.0,
///     "conversion_coeff": 0.0001722,
///     "unc_cc": null
///   }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct Record {
    /// Radiation energy (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(alias = "mean_energy")]
    pub energy: Option<f32>,

    /// Uncertainty in radiation energy (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(alias = "unc_mean")] // this is changed from unc_me for beta-/+
    pub unc_en: Option<f32>,

    /// Radiation intensity (%)
    #[serde(alias = "intensity_beta")]
    #[serde(deserialize_with = "csv::invalid_option")]
    pub intensity: Option<f32>,

    /// Uncertainty in radiation intensity (%)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(alias = "unc_ib")]
    pub unc_i: Option<f32>,

    /// Parent half-life (s)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "half_life_sec")]
    pub half_life: Option<f32>,

    // todo make sure this gets the right value
    /// Uncertainty in parent half-life (s)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_hls: Option<f32>,

    /// Decay mechanism
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "decay")]
    pub decay_mode: Option<String>,

    /// Decay mechanism branching ratio (%)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "decay_%")]
    pub branching: Option<f32>,

    /// Decay mechanism branching ratio uncertainty
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "unc_d")]
    pub unc_b: Option<f32>,

    /// Nuclear state of the parent nuclide
    #[serde(deserialize_with = "csv::invalid_option")]
    pub jp: Option<String>,

    /// Q-value (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub q: Option<String>,

    /// Q-value uncertainty (%)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_q: Option<String>,

    // * Parent nuclide
    /// Element of the parent
    #[serde(deserialize_with = "csv::invalid_option")]
    pub p_symbol: Option<String>,

    /// Parent proton number
    #[serde(deserialize_with = "csv::invalid_option")]
    pub p_z: Option<u8>,

    /// Parent neutron number
    #[serde(deserialize_with = "csv::invalid_option")]
    pub p_n: Option<u8>,

    /// Parent energy state (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub p_energy: Option<f32>,

    /// Uncertainty in parent energy state (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_pe: Option<String>,

    // * Daughter nuclide
    /// Element of the Daughter
    #[serde(deserialize_with = "csv::invalid_option")]
    pub d_symbol: Option<String>,

    /// Daughter proton number
    #[serde(deserialize_with = "csv::invalid_option")]
    pub d_z: Option<u8>,

    /// Daughter neutron number
    #[serde(deserialize_with = "csv::invalid_option")]
    pub d_n: Option<u8>,

    /// Data specific to the radiation type requested
    // #[serde(skip)]
    pub special_data: SpecialData,
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Record\n".to_string();
        s += &f!(
            "  Energy           {} +/- {} keV\n",
            self.energy.display(),
            self.unc_en.display()
        );
        s += &f!(
            "  Intensity        {} +/- {} %\n",
            self.intensity.display(),
            self.unc_i.display()
        );
        s += &f!(
            "  Half life        {} +/- {} s\n",
            self.half_life.display(),
            self.unc_hls.display()
        );
        s += &f!("  Decay mode       {}\n", self.decay_mode.display());
        s += &f!(
            "  Branching        {} +/- {} %\n",
            self.branching.display(),
            self.unc_b.display()
        );
        s += &f!("  State            {}\n", self.jp.display());
        s += &f!(
            "  Q value          {} +/- {} keV\n",
            self.q.display(),
            self.unc_q.display()
        );
        s += &f!("  Parent symbol    {}\n", self.p_symbol.display());
        s += &f!("  Parent z         {}\n", self.p_z.display());
        s += &f!("  Parent n         {}\n", self.p_n.display());
        s += &f!(
            "  Parent energy    {} +/- {} keV\n",
            self.p_energy.display(),
            self.unc_pe.display()
        );
        s += &f!("  Daughter symbol  {}\n", self.d_symbol.display());
        s += &f!("  Daughter z       {}\n", self.d_z.display());
        s += &f!("  Daughter n       {}\n", self.d_n.display());
        s += &f!("{}\n", self.special_data);
        write!(f, "{s}")
    }
}

impl Record {
    /// Check if the parent is in an excited state
    pub fn is_isomer(&self) -> bool {
        matches!(self.p_energy, Some(e) if e > 0.0)
    }

    /// Common formatting for the parent nuclide
    pub fn parent_name(&self) -> String {
        let n = self.p_n.unwrap() as u16;
        let z = self.p_z.unwrap() as u16;
        f!("{}{}", self.p_symbol.as_ref().unwrap(), n + z,)
    }

    /// Common formatting for the daughter nuclide
    pub fn daughter_name(&self) -> String {
        f!(
            "{}{}",
            self.d_symbol.as_ref().unwrap(),
            self.d_n.unwrap() + self.d_z.unwrap(),
        )
    }

    /// Serailize to JSON format string
    ///
    /// ```rust
    /// # use ntools_iaea::{fetch_nuclide_records, RadType};
    /// // Get all records for the Cobalt-60 nuclide
    /// let co60_records = fetch_nuclide_records("co60", RadType::Gamma).unwrap();
    ///
    /// // Find the 1173 keV emission as an example
    /// let example = co60_records
    ///         .iter()
    ///         .find(|record| record.energy.unwrap() == 1173.228).unwrap();
    ///
    /// // Print a summary of the record
    /// println!("{}", example.to_json().unwrap());
    /// ```
    pub fn to_json(&self) -> Result<String> {
        let mut s = serde_json::to_string_pretty(self)?;
        let special = serde_json::to_string_pretty(&self.special_data)?;
        s.insert_str(s.len() - 2, &special[1..special.len() - 2]);
        Ok(s)
    }
}
