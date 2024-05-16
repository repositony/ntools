//! Set of useful parser combinators

// internal modules
use crate::common::{IsomerState, Nuclide};
use ntools_format::capitalise;

// external crates
use log::warn;

// nom parser combinators
use nom::branch::alt;
use nom::character::complete::{alpha1, one_of};
use nom::combinator::opt;
use nom::error::{Error, ErrorKind};
use nom::{self, Err, IResult};

/// Parse string into a usable Nuclide as a common type
///
/// Can be:
///     - Element only Co, C
///     - Isotope Co60, C12
///     - Metastable Co60m1 Co60m2 Co60m3 ...
///     - Fispact Co60m Co60n Co60mo
///
/// Full is <element><separator><isotope><metastable>
///
/// Assume fispact == std metastable but warn
/// Allow separators between <element> and <isotope>
/// Unknown states set to ground with warning
/// Must enforce because things like 104mn is ambiguous -> Mn-104 or N-104m?
/// No guarentee fispact aligns with the m1 m2 m3 of the IAEA data
pub(crate) fn nuclide_from_str(i: &str) -> IResult<&str, Nuclide> {
    let (i, element) = element(i)?;
    let (i, _) = opt(separator)(i)?;
    let (i, isotope) = opt(isotope)(i)?;

    // Only look for a metastable tag if it follows an isotope number
    let isomer_state = if isotope.is_some() {
        let (_, m) = metastable(i)?;
        match m {
            Some(isomer) => isomer,
            None => IsomerState::Ground,
        }
    } else {
        IsomerState::Ground
    };

    Ok((
        i,
        Nuclide {
            symbol: capitalise(element),
            isotope: isotope.unwrap_or(000),
            state: isomer_state,
        },
    ))
}

/// Get the element symbol
fn element(i: &str) -> IResult<&str, &str> {
    let (i, element) = alpha1(i)?;

    if element.len() > 2 {
        Err(Err::Error(Error::new(i, ErrorKind::Fail)))
    } else {
        Ok((i, element))
    }
}

/// Get an unsigned integer value
fn isotope(i: &str) -> IResult<&str, u16> {
    nom::character::complete::u16(i)
}

/// Get the stability from a range of possible formats and conventions
fn metastable(i: &str) -> IResult<&str, Option<IsomerState>> {
    opt(alt((numbered_isomer, symbol_isomer)))(i)
}

/// List of possible separators people may use
fn separator(i: &str) -> IResult<&str, char> {
    nom::character::complete::one_of("_-")(i)
}

/// Get the isomer from the usual IAEA formats m1, m2, etc...
fn numbered_isomer(i: &str) -> IResult<&str, IsomerState> {
    let (i, _) = one_of("mM")(i)?;
    let (i, number) = nom::character::complete::u8(i)?;

    if number == 0 {
        Ok((i, IsomerState::Ground))
    } else {
        Ok((i, IsomerState::Excited(number)))
    }
}

/// Get the isomer type from known fispact/common use symbols
fn symbol_isomer(i: &str) -> IResult<&str, IsomerState> {
    match i {
        "g" | "" => Ok((i, IsomerState::Ground)),
        "m" | "*" => Ok((i, IsomerState::Excited(1))),
        "n" => Ok((i, IsomerState::Excited(2))),
        "o" => Ok((i, IsomerState::Excited(3))),
        _ => {
            warn!("Unable to infer isomer from \"{i}\", set to ground");
            Err(Err::Error(Error::new(i, ErrorKind::Fail)))
        }
    }
}
