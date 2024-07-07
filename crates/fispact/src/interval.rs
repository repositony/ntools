use crate::{Nuclide, SortProperty};
use ntools_format::f;

use serde::{de::Error, Deserialize, Deserializer, Serialize};

/// Data for a time step in the calculation
///
/// Traditionally this is all data written when the `ATOMS` keyword is used on a
/// irradiation or cooling step.
///  
/// # Important differences to the JSON structure
///
/// The following choices were made during deserialisation:
/// - All masses are automatically converted to grams
/// - All `"total_*"` keys are shortened for brevity
///     - e.g. `total_activity` is shortened to `activity`
/// - The dose rate dictionaries are converted to more concise [Dose] structures
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
    /// List of names for all nuclides in the interval
    pub fn nuclide_names(&self) -> Vec<String> {
        let mut nuclides: Vec<String> =
            self.nuclides.iter().map(|nuclide| nuclide.name()).collect();
        nuclides.sort();
        nuclides.dedup();
        nuclides
    }

    /// List of names for all unique elements in the interval
    pub fn element_names(&self) -> Vec<String> {
        let mut elements: Vec<String> = self
            .nuclides
            .iter()
            .map(|nuclide| nuclide.element.clone())
            .collect();
        elements.sort();
        elements.dedup();
        elements
    }

    /// Collection of only the stable nuclides
    pub fn stable_nuclides(&self) -> Vec<&Nuclide> {
        self.nuclides
            .iter()
            .filter(|n| n.half_life == 0.0)
            .collect()
    }

    /// Collection of only the unstable nuclides
    pub fn unstable_nuclides(&self) -> Vec<&Nuclide> {
        self.nuclides.iter().filter(|n| n.half_life > 0.0).collect()
    }

    /// Basic search for a nuclide in the interval by name
    pub fn find_nuclide(&self, target: &str) -> Option<&Nuclide> {
        self.nuclides
            .iter()
            .find(|n| n.name().to_lowercase().starts_with(&target.to_lowercase()))
    }

    /// Sort nuclides in ascending order by property
    pub fn sort_ascending(&mut self, property: SortProperty) {
        match property {
            SortProperty::Activity => self
                .nuclides
                .sort_by(|a, b| a.activity.partial_cmp(&b.activity).unwrap()),
            SortProperty::Mass => self
                .nuclides
                .sort_by(|a, b| a.mass.partial_cmp(&b.mass).unwrap()),
            SortProperty::Dose => self
                .nuclides
                .sort_by(|a, b| a.dose.partial_cmp(&b.dose).unwrap()),
            SortProperty::Atoms => self
                .nuclides
                .sort_by(|a, b| a.atoms.partial_cmp(&b.atoms).unwrap()),
            SortProperty::Heat => self
                .nuclides
                .sort_by(|a, b| a.heat.partial_cmp(&b.heat).unwrap()),
        }
    }

    /// Sort nuclides in decending order by property
    pub fn sort_descending(&mut self, property: SortProperty) {
        self.sort_ascending(property);
        self.nuclides.reverse()
    }

    /// Filter nuclides by some predicate
    ///
    /// Returns references to the interval nuclides after filtering by the given
    /// condition. This can be any function that operates on the nuclide that
    /// returns a boolean.
    ///
    /// ```rust, ignore
    /// let inventory = read_json("path/to/data.json")?;
    /// let interval  = inventory.intervals[2];
    ///
    /// // Filter out any nuclides with activity below 1e8 Bq
    /// let nuclides = intervals[2].filter(|n| n.activity > 1e8);
    /// for nuclide in nuclides {
    ///     println!("{} {}", nuclide.name(), nuclide.activity)
    /// }
    /// ```
    pub fn filter<P>(&self, predicate: P) -> Vec<&Nuclide>
    where
        P: FnMut(&&Nuclide) -> bool,
    {
        self.nuclides.iter().filter(predicate).collect()
    }

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
/// Note that this is not directly translated from the original JSON structure.
/// Instead, the deserialiser was written to make this more ergonomic and
/// properly leverage the type system.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
