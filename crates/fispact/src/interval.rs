use crate::nuclide::Nuclide;
use ntools_format::f;

use serde::{de::Error, Deserialize, Deserializer, Serialize};

/// Interval/time step data
///
/// Modifications
/// - mass automatically converted to grams for consistency
/// - totals renamed for brevity
/// - dose converted to more concise [Dose] structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Interval {
    /// Irradiation time (s)
    pub irradiation_time: f64,
    /// Cooling time (s)
    pub cooling_time: f64,
    /// Particle flux (#/cm2/s)
    pub flux: f64,
    /// Total number of atoms in sample
    #[serde(rename = "total_atoms")]
    pub atoms: f64,
    /// Total sample activity (Bq)
    #[serde(rename = "total_activity")]
    pub activity: f64,
    /// Total sample alpha activity (Bq)
    pub alpha_activity: f64,
    /// Total sample beta activity (Bq)
    pub beta_activity: f64,
    /// Total sample gamma activity (Bq)
    pub gamma_activity: f64,
    /// Total mass of the sample (g)
    #[serde(rename = "total_mass")]
    #[serde(deserialize_with = "from_mass_kg")]
    pub mass: f64,
    /// Combined total heating (kW)
    #[serde(rename = "total_heat")]
    pub heat: f64,
    /// Total heat from alpha emissions (kW)
    pub alpha_heat: f64,
    /// Total heat from beta emissions (kW)
    pub beta_heat: f64,
    /// Total heat from gamma emissions (kW)
    pub gamma_heat: f64,
    /// Ingestion dose (Sv/kg)
    #[serde(rename = "ingestion_dose")]
    pub ingestion: f64,
    /// Inhalation dose (Sv/kg)
    #[serde(rename = "inhalation_dose")]
    pub inhalation: f64,
    /// Dose rate information
    #[serde(rename = "dose_rate")]
    #[serde(deserialize_with = "from_dose_rate")]
    pub dose: Dose,
    /// Gamma spectrum 'boundaries' (MeV) and 'values' lists
    #[serde(rename = "gamma_spectrum")]
    pub spectrum: Spectrum,
    /// Nuclides list
    pub nuclides: Vec<Nuclide>,
}

impl Interval {
    /// Apply a flux normalisation factor to the appropriate fields
    pub fn apply_normalisation(&mut self, norm: f64) {
        self.flux *= norm;
        self.dose.rate *= norm;
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

        for n in &mut self.nuclides {
            n.apply_normalisation(norm);
        }
    }
}

/// Total sample dose rate and type
///
/// Note this is not the same as the original to make this more ergonomic and
/// usable for programming.
#[derive(Serialize, Deserialize, Debug)]
pub struct Dose {
    /// Dose rate (Sv/hr)
    #[serde(rename = "dose")]
    pub rate: f64,
    /// Type of dose
    #[serde(rename = "type")]
    pub kind: DoseKind,
}

/// Type of dose rate
///
/// Either Contact or dose, where the point source contains the distance. This
/// must be >0.3m.
#[derive(Serialize, Deserialize, Debug)]
pub enum DoseKind {
    /// Semi-infinite slab approximation
    Contact,
    /// Point source approximation at contained distance (m)
    Point(f64),
}

/// Predicted gamma spectrum intensities
///
/// Good indicator that can have arbitrary bin structures, but do not expect it
/// to look like anything you would get out of a detector.
#[derive(Serialize, Deserialize, Debug)]
pub struct Spectrum {
    /// Bin edges (MeV)
    #[serde(rename = "boundaries")]
    pub edges: Vec<f64>,
    /// Intensity (MeV/s)
    pub values: Vec<f64>,
}

/// Custom deserialiser for a mass in units of kg
fn from_mass_kg<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    // return the converted mass from kg -> g
    let mass: f64 = Deserialize::deserialize(deserializer)?;
    Ok(mass * 1.0e+03)
}

/// Custom deserialiser for the "dose_rate" dictionary
fn from_dose_rate<'de, D>(deserializer: D) -> Result<Dose, D::Error>
where
    D: Deserializer<'de>,
{
    // deserialise into an intermediate for convenience
    let cd: CoreDose = Deserialize::deserialize(deserializer)?;

    // try to infer the kind of dose rate
    let kind = match cd.kind.to_lowercase().as_str() {
        "contact" => DoseKind::Contact,
        "point source" => DoseKind::Point(cd.distance),
        _ => {
            return Err(D::Error::custom(f!(
                "Dose rate type not recognised: '{}'",
                cd.kind
            )))
        }
    };

    // return a more sensible structure
    Ok(Dose {
        rate: cd.dose,
        kind,
    })
}

/// Intermediate type for custom "dose_rate" deserialiser
#[derive(Serialize, Deserialize, Debug)]
struct CoreDose {
    #[serde(rename = "type")]
    kind: String,
    distance: f64,
    dose: f64,
}
