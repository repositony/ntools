use ntools_support::{f, OptionExt};
use serde::{Deserialize, Serialize};

/// Collection of specialised data fields under one type
///
/// Every `rad_type` request to the IAEA chart of nuclides returns a set of CSV
/// data with ~25 common columns, along with up to 10 that are unique to the
/// type of decay radiation.
///
/// For simplicity any unique fields are collected as variant under the single
/// type [SpecialData].  
///
/// No matter what the request, the user will always get the same
/// [Record](crate::Record) no matter the radiation type with all the common
/// information and expected functionality. The `special_data` field of a
/// [Record](crate::Record) then contains any information specific to the
/// radiation type requested.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[repr(C)]
pub enum SpecialData {
    #[default]
    None,
    Alpha(Alpha),
    BetaPlus(BetaPlus),
    BetaMinus(BetaMinus),
    Gamma(Gamma),
    Electron(Electron),
    Xray(Xray),
}

impl std::fmt::Display for SpecialData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::None => "None".to_string(),
            Self::Alpha(inner) => f!("{inner}"),
            Self::BetaPlus(inner) => f!("{inner}"),
            Self::BetaMinus(inner) => f!("{inner}"),
            Self::Gamma(inner) => f!("{inner}"),
            Self::Electron(inner) => f!("{inner}"),
            Self::Xray(inner) => f!("{inner}"),
        };
        write!(f, "{s}")
    }
}

impl From<Alpha> for SpecialData {
    fn from(data: Alpha) -> Self {
        SpecialData::Alpha(data)
    }
}

impl From<BetaPlus> for SpecialData {
    fn from(data: BetaPlus) -> Self {
        SpecialData::BetaPlus(data)
    }
}

impl From<BetaMinus> for SpecialData {
    fn from(data: BetaMinus) -> Self {
        SpecialData::BetaMinus(data)
    }
}

impl From<Gamma> for SpecialData {
    fn from(data: Gamma) -> Self {
        SpecialData::Gamma(data)
    }
}

impl From<Electron> for SpecialData {
    fn from(data: Electron) -> Self {
        SpecialData::Electron(data)
    }
}

impl From<Xray> for SpecialData {
    fn from(data: Xray) -> Self {
        SpecialData::Xray(data)
    }
}

/// Special data for alpha decay
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct Alpha {
    /// Energy of fed level (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub daughter_level_energy: Option<f32>,

    /// Alpha transition hindrance factor
    #[serde(deserialize_with = "csv::invalid_option")]
    pub hindrance_factor: Option<f32>,

    /// Uncertainty in alpha transition hindrance factor
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_hf: Option<f32>,
}

impl std::fmt::Display for Alpha {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Alpha\n".to_string();
        s += &f!(
            "  Daugher level    {} keV\n",
            self.daughter_level_energy.display()
        );
        s += &f!(
            "  Hindrance factor {} +/- {}\n",
            self.hindrance_factor.display(),
            self.unc_hf.display()
        );
        write!(f, "{s}")
    }
}

/// Special data for beta-plus decay
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct BetaPlus {
    /// End-point energy for B- (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub daughter_level_energy: Option<f32>,

    /// Energy of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub energy_ec: Option<f32>,

    /// Uncertainty in energy of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_eec: Option<f32>,

    /// Intensity of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub intensity_ec: Option<f32>,

    /// Uncertainty in intensity of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_ie: Option<f32>,

    /// Log-f table value  
    #[serde(deserialize_with = "csv::invalid_option")]
    pub log_ft: Option<f32>,

    /// Log-f table value uncertainty
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_lf: Option<f32>,

    /// Transition type (A for antineutrino, xNU for x neutrinos)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub transition_type: Option<String>,

    /// Neutrino mean energy (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub nu_mean_energy: Option<f32>,

    /// Neutrino mean energy uncertainty (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_nme: Option<f32>,
}

impl std::fmt::Display for BetaPlus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "BetaPlus\n".to_string();
        s += &f!(
            "  Daughter level   {} keV\n",
            self.daughter_level_energy.display()
        );
        s += &f!(
            "  Energy EC        {} +/- {} keV\n",
            self.energy_ec.display(),
            self.unc_eec.display()
        );
        s += &f!(
            "  Intensity EC     {} +/- {} %\n",
            self.intensity_ec.display(),
            self.unc_ie.display()
        );
        s += &f!(
            "  Log-f table      {} +/- {} \n",
            self.log_ft.display(),
            self.unc_lf.display()
        );
        s += &f!("  Transition type  {}\n", self.transition_type.display());
        s += &f!(
            "  Nu mean energy   {} +/- {} keV",
            self.nu_mean_energy.display(),
            self.unc_nme.display()
        );
        write!(f, "{s}")
    }
}

/// Special data for beta-minus decay
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct BetaMinus {
    /// End-point energy for B- (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub daughter_level_energy: Option<f32>,

    /// Energy of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub energy_ec: Option<f32>,

    /// Uncertainty in energy of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_eec: Option<f32>,

    /// Intensity of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub intensity_ec: Option<f32>,

    /// Uncertainty in intensity of electron capture for B+
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_ie: Option<f32>,

    /// Log-f table value  
    #[serde(deserialize_with = "csv::invalid_option")]
    pub log_ft: Option<f32>,

    /// Log-f table value uncertainty
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_lf: Option<f32>,

    /// Transition type (A for antineutrino, xNU for x neutrinos)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub transition_type: Option<String>,

    /// Anti-neutrino mean energy (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub anti_nu_mean_energy: Option<f32>,

    /// Anti-neutrino mean energy uncertainty (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_ame: Option<f32>,
}

impl std::fmt::Display for BetaMinus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "BetaMinus\n".to_string();
        s += &f!(
            "  Daughter level   {} keV\n",
            self.daughter_level_energy.display()
        );
        s += &f!(
            "  Energy EC        {} +/- {} keV\n",
            self.energy_ec.display(),
            self.unc_eec.display()
        );
        s += &f!(
            "  Intensity EC     {} +/- {} %\n",
            self.intensity_ec.display(),
            self.unc_ie.display()
        );
        s += &f!(
            "  Log-f table      {} +/- {} \n",
            self.log_ft.display(),
            self.unc_lf.display()
        );
        s += &f!("  Transition type  {}\n", self.transition_type.display());
        s += &f!(
            "  Anti-Nu energy   {} +/- {} keV",
            self.anti_nu_mean_energy.display(),
            self.unc_ame.display()
        );
        write!(f, "{s}")
    }
}

/// Special data for gamma decay
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct Gamma {
    /// Energy of the initial level (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub start_level_energy: Option<f32>,

    /// Energy of the final level (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub end_level_energy: Option<f32>,

    /// Multipolarity of the transition
    #[serde(deserialize_with = "csv::invalid_option")]
    pub multipolarity: Option<String>,

    /// Multipole mixing ratio
    #[serde(deserialize_with = "csv::invalid_option")]
    pub mixing_ratio: Option<f32>,

    /// Multipole mixing ratio uncertainty (%)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_mr: Option<f32>,

    /// Total internal conversion coefficient
    #[serde(deserialize_with = "csv::invalid_option")]
    pub conversion_coeff: Option<f32>,

    /// Total internal conversion coefficient uncertainty
    #[serde(deserialize_with = "csv::invalid_option")]
    pub unc_cc: Option<f32>,
}

impl std::fmt::Display for Gamma {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Gamma \n".to_string();
        s += &f!(
            "  Start level      {} keV\n",
            self.start_level_energy.display()
        );
        s += &f!(
            "  End level        {} keV\n",
            self.end_level_energy.display()
        );
        s += &f!("  Multipolarity    {}\n", self.multipolarity.display());
        s += &f!(
            "  Mixing ratio     {} +/- {}\n",
            self.mixing_ratio.display(),
            self.unc_mr.display()
        );
        s += &f!(
            "  Conversion coef. {} +/- {}",
            self.conversion_coeff.display(),
            self.unc_cc.display()
        );
        write!(f, "{s}")
    }
}

impl Gamma {
    pub fn table(&self) -> String {
        f!(
            "{} {} {} {} +/- {} {} +/- {}",
            self.start_level_energy.display(),
            self.end_level_energy.display(),
            self.multipolarity.display(),
            self.mixing_ratio.display(),
            self.unc_mr.display(),
            self.conversion_coeff.display(),
            self.unc_cc.display()
        )
    }
}

/// Special data for Auger and Conversion electrons
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
#[repr(C)]
pub struct Electron {
    /// Energy of the initial level (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "type")]
    pub electron_type: Option<String>,

    /// Siegbahn notation for shell (K, L, etc...)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub shell: Option<String>,
}

impl std::fmt::Display for Electron {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Electron \n".to_string();
        s += &f!("  Electron type    {}\n", self.electron_type.display());
        s += &f!("  Shell (Siegbahn) {}", self.shell.display());
        write!(f, "{s}")
    }
}

/// Special data for Xray emissions
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct Xray {
    /// Energy of the initial level (keV)
    #[serde(deserialize_with = "csv::invalid_option")]
    #[serde(rename = "type")]
    pub xray_type: Option<String>,

    /// Siegbahn notation for shell (K, L, etc...)
    #[serde(deserialize_with = "csv::invalid_option")]
    pub shell: Option<String>,
}

impl std::fmt::Display for Xray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Xray\n".to_string();
        s += &f!("  X-ray type       {}\n", self.xray_type.display());
        s += &f!("  Shell (Siegbahn) {}", self.shell.display());
        write!(f, "{s}")
    }
}
