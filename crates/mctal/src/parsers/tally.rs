// nom parser combinators
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{anychar, char, space0, space1};
use nom::combinator::opt;
use nom::multi::many1;
use nom::number::complete::double;
use nom::sequence::{preceded, terminated};
use nom::{self, IResult};

use crate::parsers::cause;
use crate::parsers::number::*;
use crate::{BinData, BinFlag, BinKind, Modifier, TallyKind, TallyResult, Tfc, TfcResult};

#[derive(Debug)]
pub(crate) struct TallyHeader {
    /// Tally number
    pub id: u32,
    /// Particle type(s)
    pub particle_flag: i8,
    /// Tally type
    pub kind: TallyKind,
    /// Tally modifier (none, *, +)
    pub modifier: Modifier,
}

/// Checks if the line can be parsed into the TallyHeader struct
pub(crate) fn is_new_tally(i: &str) -> bool {
    tally_header(i).is_ok()
}

/// Parse whole line into a TallyHeader struct
pub(crate) fn tally_header(i: &str) -> IResult<&str, TallyHeader> {
    let (i, id) = tally_number(i)?;
    let (i, particles) = iint8(i)?;
    let (i, kind) = preceded(space1, tally_kind)(i)?;
    let (i, modifier) = preceded(space1, modifier)(i)?;

    Ok((
        i,
        TallyHeader {
            id,
            particle_flag: particles,
            kind,
            modifier,
        },
    ))
}

/// Parse the tally id following the "tally" tag
fn tally_number(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("tally"), uint32)(i.trim_start())
}

/// Parse a tally identifier into a proper variant
fn tally_kind(i: &str) -> IResult<&str, TallyKind> {
    let (i, kind) = uint32(i)?;
    match kind {
        0 => Ok((i, TallyKind::None)),
        1 => Ok((i, TallyKind::Point)),
        2 => Ok((i, TallyKind::Ring)),
        3 => Ok((i, TallyKind::Pinhole)),
        4 => Ok((i, TallyKind::TransmittedRectangular)),
        5 => Ok((i, TallyKind::TransmittedCylindrical)),
        _ => Err(cause("invalid tally type")),
    }
}

/// Parse a tally modifier into an explicit type
fn modifier(i: &str) -> IResult<&str, Modifier> {
    let (i, modifier) = uint32(i)?;

    match modifier {
        0 => Ok((i, Modifier::None)),
        1 => Ok((i, Modifier::Star)),
        2 => Ok((i, Modifier::Plus)),
        _ => Err(cause("invalid tally modifier type")),
    }
}

/// f_bins value
pub(crate) fn region_bins(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("f"), uint32)(i)
}

/// d_bins value
pub(crate) fn detector_bins(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("d"), uint32)(i)
}

pub(crate) fn bin_data(i: &str, token: char) -> IResult<&str, BinData> {
    let (i, token) = char(token)(i.trim_start())?;
    let (i, bin_tag) = opt(alt((char('t'), char('c'))))(i)?;
    let (i, number) = uint(i)?;
    let (i, bin_flag) = opt(uint)(i)?;

    Ok((
        i,
        BinData {
            token,
            number,
            unbound: number == 0,
            kind: BinKind::from(bin_tag),
            flag: BinFlag::from(bin_flag),
            values: Vec::with_capacity(number),
        },
    ))
}

/// Check for the results section by the "vals" tag
pub(crate) fn is_vals(i: &str) -> bool {
    i.trim_start().starts_with("vals")
}

/// Collect a list of TallyResults
pub(crate) fn vector_of_tally_results(i: &str) -> IResult<&str, Vec<TallyResult>> {
    many1(terminated(result_pair, space0))(i.trim_start())
}

/// Collect pairs of values into a TallyResult
fn result_pair(i: &str) -> IResult<&str, TallyResult> {
    let (i, (value, error)) =
        nom::sequence::separated_pair(double, space1, double)(i.trim_start())?;

    Ok((i, TallyResult { value, error }))
}

/// Parse the tally fluctuation chart into a TallyFluctuationChart, leaving the
/// records empty for now
pub(crate) fn tfc(i: &str) -> IResult<&str, Tfc> {
    let (i, _) = tag_no_case("tfc")(i)?;
    let (i, n_sets) = uint32(i)?;
    let (i, f_bin) = uint32(i)?;
    let (i, d_bin) = uint32(i)?;
    let (i, u_bin) = uint32(i)?;
    let (i, s_bin) = uint32(i)?;
    let (i, m_bin) = uint32(i)?;
    let (i, c_bin) = uint32(i)?;
    let (i, e_bin) = uint32(i)?;
    let (i, t_bin) = uint32(i)?;

    Ok((
        i,
        Tfc {
            n_records: n_sets,
            n_flagged_bins: f_bin,
            n_region_bins: d_bin,
            n_user_bins: u_bin,
            n_segment_bins: s_bin,
            n_multiplier_bins: m_bin,
            n_cosine_bins: c_bin,
            n_energy_bins: e_bin,
            n_time_bins: t_bin,
            ..Default::default()
        },
    ))
}

/// Parse a record of the tally fluctuation chart into a TfResult
pub(crate) fn tfc_result(i: &str) -> IResult<&str, TfcResult> {
    let (i, nps) = uint64(i.trim_start())?;
    let (i, value) = double(i.trim_start())?;
    let (i, error) = double(i.trim_start())?;
    let (i, fom) = double(i.trim_start())?;

    Ok((
        i,
        TfcResult {
            nps,
            value,
            error,
            fom,
        },
    ))
}
