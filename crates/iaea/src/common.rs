//! Common data structures

// external crates
use serde::Deserialize;

// ntools modules
use ntools_format::{capitalise, f};

// internal modules
use crate::error::{Error, Result};
use crate::parsers::nuclide_from_str;

/// Type of decay radiation
///
/// The IAEA chart of nuclides contains data for the following types:
///
/// - Alpha (`a`)
/// - Beta+ or electron capture (`bp`)
/// - Beta- (`bm`)
/// - Gamma decay (`g`)
/// - Auger and conversion electron (`e`)
/// - X-ray (`x`)
///
/// This enum collects them together for simple and explicit requests.
///
/// The `FromStr` trait is implemented for all radiation types for easy
/// conversuion between the variants and their symbols required by the IAEA API.
///
/// ```rust
/// # use ntools_iaea::RadType;
/// # use std::str::FromStr;
/// // Get the variant from an IAEA symbol
/// assert_eq!(RadType::from_str("a").unwrap(), RadType::Alpha);
/// ```
///
/// Of course, the reverse is also available if the symbol is required:
///
/// ```rust
/// # use ntools_iaea::RadType;
/// # use std::str::FromStr;
/// // Get the IAEA symbol for a variant
/// assert_eq!(RadType::Alpha.query_symbol(), "a");
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RadType {
    /// Alpha decay (`a`)
    Alpha,
    /// Beta+ or electron capture (`bp`)
    BetaPlus,
    /// Beta- (`bm`)
    BetaMinus,
    /// Gamma decay (`g`)
    Gamma,
    /// Auger and conversion electron (`e`)
    Electron,
    /// X-ray (`x`)
    Xray,
}

impl RadType {
    /// Get corresponding API symbol for a variant
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_iaea::RadType;
    /// # use std::str::FromStr;
    /// // Get the IAEA symbol for a variant
    /// assert_eq!(RadType::Alpha.query_symbol(), "a");
    /// assert_eq!(RadType::BetaMinus.query_symbol(), "bm");
    /// assert_eq!(RadType::BetaPlus.query_symbol(), "bp");
    /// assert_eq!(RadType::Gamma.query_symbol(), "g");
    /// assert_eq!(RadType::Electron.query_symbol(), "e");
    /// assert_eq!(RadType::Xray.query_symbol(), "x");
    /// ```
    pub fn query_symbol(&self) -> &str {
        match self {
            RadType::Alpha => "a",
            RadType::BetaPlus => "bp",
            RadType::BetaMinus => "bm",
            RadType::Gamma => "g",
            RadType::Electron => "e",
            RadType::Xray => "x",
        }
    }
}

impl std::str::FromStr for RadType {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "a" => Ok(RadType::Alpha),
            "bp" => Ok(RadType::BetaPlus),
            "bm" => Ok(RadType::BetaMinus),
            "g" => Ok(RadType::Gamma),
            "e" => Ok(RadType::Electron),
            "x" => Ok(RadType::Xray),
            _ => Err(Error::CouldNotInferRadType {
                hint: s.to_string(),
            }),
        }
    }
}

/// Variants of excited states
///
/// A nuclide can either be in the ground state, or some excited state.
///
/// Excited state isomers use the slightly more standardised ENSDF notation,
/// where `m1` is the first excied state, `m2`, the second, and so on...
///
/// Note that this may be converted from anything ending with the FISPACT-II
/// notations of m, n, etc.. but it is not completely guaranteed that this is a
/// 1:1 mapping.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Default, Clone)]
pub enum IsomerState {
    #[default]
    Ground,
    Excited(u8),
}

impl std::fmt::Display for IsomerState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let state = match self {
            IsomerState::Ground => String::from(""),
            IsomerState::Excited(e) => f!("m{e}"),
        };
        write!(f, "{state}")
    }
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct BaseNuclide {
    pub symbol: String,
    pub z: u16,
    pub n: u16,
}

impl std::fmt::Display for BaseNuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", capitalise(&self.symbol), self.z + self.n)
    }
}

/// Definition for a particular nuclide
///
/// The `TryFrom` trait is implemented for core string types and will try to
/// parse them into a nuclide.
///
/// Expects `<element><separator><isotope><metastable>` at most but only the
/// element is required. e.g.
///
/// - Element only => Co, C
/// - Isotope => Co60, C12
/// - Metastable => Co60m1 Co60m2 Co60m3 ...
/// - Fispact => Co60m Co60n Co60mo
///
/// This order must be enforced because something like "104mn" is ambiguous.
/// i.e. should it be interpreted as Mn-104 or N-104m?
///
/// Note that the metastable state should be the ENSDF notation (m1, m2, m3,
/// etc...).
///
/// The metastable symbol can be converted from anything ending with the
/// FISPACT-II notation of m, n, etc..., but it can not be guaranteed that
/// this is a 1:1 mapping.
///
/// ```rust
/// # use ntools_iaea::{Nuclide, IsomerState};
/// // Get the variant from an IAEA symbol
/// assert_eq!(
///     Nuclide::try_from("eu-152m2").unwrap(),
///     Nuclide {
///         symbol: "eu".to_string(),
///         isotope: 152,
///         state: IsomerState::Excited(2)
///     }
/// );
/// ```
#[derive(Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Nuclide {
    /// Element
    pub symbol: String,
    /// Isotope number (Z+N, total nucleons)
    pub isotope: u16,
    /// Excited state status
    pub state: IsomerState,
}

impl Nuclide {
    /// A basic name for the nuclide
    ///
    /// The nuclide name will be formatted as `<element><isotope number>`
    /// to provide a display name with consistent formatting.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_iaea::{Nuclide, IsomerState};
    /// let mut nuclide = Nuclide {
    ///     symbol: "eu".to_string(),
    ///     isotope: 152,
    ///     state: IsomerState::Ground
    /// };
    ///
    /// // Get a display name for the nuclide
    /// assert_eq!(nuclide.name(), "Eu152");
    /// ```
    pub fn name(&self) -> String {
        // special case for elements
        let isotope = if self.isotope == 0 {
            "".to_string()
        } else {
            self.isotope.to_string()
        };

        f!("{}{}", capitalise(&self.symbol), isotope)
    }

    /// A name for the nuclide including isomer state
    ///
    /// The nuclide name will be formatted as `<element><isotope number><state>`
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_iaea::{Nuclide, IsomerState};
    /// let nuclide = Nuclide {
    ///     symbol: "eu".to_string(),
    ///     isotope: 152,
    ///     state: IsomerState::Excited(1)
    /// };
    ///
    /// // Get a display name for the excited nuclide
    /// assert_eq!(nuclide.name_with_state(), "Eu152m1");
    /// ```
    pub fn name_with_state(&self) -> String {
        f!("{}{}", self.name(), self.state)
    }

    /// Name formatted for IAEA queries
    ///
    /// The IAEA API expects the nuclide as `<mass><element>` with no state
    /// information.
    ///
    /// Calls to this method for elements (i.e. mass set to 0) will return an
    /// error.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_iaea::{Nuclide, IsomerState};
    /// let nuclide = Nuclide {
    ///     symbol: "eu".to_string(),
    ///     isotope: 152,
    ///     state: IsomerState::Excited(1)
    /// };
    ///
    /// // Get the format expected for the IAEA API query
    /// assert_eq!(nuclide.query_name().unwrap(), "152eu");
    /// ```
    pub fn query_name(&self) -> Result<String> {
        // special case for elements
        if self.isotope == 0 {
            return Err(Error::InvalidNuclideQuery);
        }

        Ok(f!("{}{}", self.isotope, self.symbol.to_lowercase()))
    }
}

impl std::fmt::Display for Nuclide {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name_with_state())
    }
}

impl TryFrom<&str> for Nuclide {
    type Error = crate::Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let (_, nuclide) = nuclide_from_str(s).map_err(|_| crate::Error::FailedParseToNuclide {
            text: s.to_string(),
        })?;

        Ok(nuclide)
    }
}

impl TryFrom<String> for Nuclide {
    type Error = crate::Error;
    fn try_from(s: String) -> Result<Self> {
        s.as_str().try_into()
    }
}

impl TryFrom<&String> for Nuclide {
    type Error = crate::Error;
    fn try_from(s: &String) -> Result<Self> {
        s.as_str().try_into()
    }
}
