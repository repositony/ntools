// nom parser combinators
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, space1};
use nom::sequence::preceded;
use nom::{self, IResult};

use crate::parsers::cause;
use crate::parsers::number::{iint8, uint, uint32};
use crate::Geometry;

#[derive(Debug)]
pub(crate) struct TmeshHeader {
    /// Tally number
    pub id: u32,
    /// Tally type
    pub geometry: Geometry,
}

#[derive(Debug)]
pub(crate) struct TmeshDimensions {
    /// mxgc is the total number of spatial bins (or “voxels”) in the TMESH tally.
    pub n_voxels: usize,
    /// ng1 is the number of bins on the CORA card
    pub n_cora: usize,
    /// ng2 is the number of bins on the CORB card
    pub n_corb: usize,
    /// ng3 is the number of bins on the CORC card
    pub n_corc: usize,
}

/// Checks if the line can be parsed into the TmeshHeader struct
pub(crate) fn is_new_tmesh(i: &str) -> bool {
    tmesh_header(i).is_ok()
}

/// Parse whole line into a TmeshHeader struct
pub(crate) fn tmesh_header(i: &str) -> IResult<&str, TmeshHeader> {
    let (i, id) = tmesh_number(i)?;
    let (i, _) = iint8(i)?;
    let (i, geometry) = preceded(space1, tmesh_geometry)(i)?;
    Ok((i, TmeshHeader { id, geometry }))
}

/// Parse a geometry flag into an explicit enum variant
fn tmesh_geometry(i: &str) -> IResult<&str, Geometry> {
    let (i, number) = iint8(i)?;

    match number.abs() {
        1 => Ok((i, Geometry::Rectangular)),
        2 => Ok((i, Geometry::Cylindrical)),
        3 => Ok((i, Geometry::Spherical)),
        _ => Err(cause("unrecognised TMESH geometry flag")),
    }
}

/// Parse the tmesh id following the "tally" tag
fn tmesh_number(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("tally"), uint32)(i.trim_start())
}

/// Parse line to dimensions for the tmesh geometry
pub(crate) fn tmesh_coordinates(i: &str) -> IResult<&str, TmeshDimensions> {
    let (i, voxels) = preceded(tag_no_case("f"), uint)(i)?;
    let (i, _) = uint(i)?;
    let (i, cora_bins) = uint(i)?;
    let (i, corb_bins) = uint(i)?;
    let (i, corc_bins) = uint(i)?;

    Ok((
        i,
        TmeshDimensions {
            n_voxels: voxels,
            n_cora: cora_bins,
            n_corb: corb_bins,
            n_corc: corc_bins,
        },
    ))
}

pub(crate) fn basic_bin(i: &str, token: char) -> IResult<&str, usize> {
    let (i, _) = char(token)(i.trim_start())?;
    uint(i)
}
