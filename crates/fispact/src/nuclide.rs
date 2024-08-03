use ntools_utils::{f, StringExt};
use serde::{Deserialize, Serialize};

/// Basic satbility variants
pub enum Stability {
    /// Stable nuclides with no halflife
    Stable,
    /// Unstable nuclides defined by any non-zero halflife
    Unstable,
    /// Non preference in stability, use both
    Any,
}

/// Nuclide data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nuclide {
    /// Element symbol e.g. 'He', 'Co'
    pub element: String,
    /// Atomic mass number
    pub isotope: u32,
    // State in FISPACT notation i.e. m, n, etc...
    pub state: String,
    /// Nuclide decay half-life (s)
    pub half_life: f64,
    /// ZZZAAAI identifier
    pub zai: u32,
    /// Number of atoms
    pub atoms: f64,
    /// Mass (g)
    #[serde(rename = "grams")]
    pub mass: f64,
    /// Activity (Bq)
    pub activity: f64,
    /// Alpha activity (Bq)
    pub alpha_activity: f64,
    /// Beta activity (Bq)
    pub beta_activity: f64,
    /// Gamma activity (Bq)
    pub gamma_activity: f64,
    /// Total heating from nuclide (kW)
    pub heat: f64,
    /// Heating from alpha emissions (kW)
    pub alpha_heat: f64,
    /// Heating from beta emissions (kW)
    pub beta_heat: f64,
    /// Heating from gamma emissions (kW)
    pub gamma_heat: f64,
    /// Dose rate (Sv/hr)
    pub dose: f64,
    /// Ingestion dose (Sv)
    pub ingestion: f64,
    /// Inhalation dose (Sv)
    pub inhalation: f64,
}

impl Nuclide {
    /// Simple formatted string to identify the nuclide
    pub fn name(&self) -> String {
        f!(
            "{}{}{}",
            &self.element.capitalise(),
            self.isotope,
            self.state
        )
    }

    /// Apply a flux normalisation factor to the appropriate fields
    pub fn apply_normalisation(&mut self, norm: f64) {
        self.dose *= norm;
        self.ingestion *= norm;
        self.inhalation *= norm;
        self.heat *= norm;
        self.alpha_heat *= norm;
        self.beta_heat *= norm;
        self.gamma_heat *= norm;
        self.activity *= norm;
        self.alpha_activity *= norm;
        self.beta_activity *= norm;
        self.gamma_activity *= norm;
    }
}
