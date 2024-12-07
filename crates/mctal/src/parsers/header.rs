// nom parser combinators
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::opt;
use nom::sequence::preceded;
use nom::{self, IResult};

use crate::parsers::number::{uint32, uint64};
use crate::parsers::split_index;

#[derive(Debug)]
pub(crate) struct FirstLine {
    /// Name of the code, "MCNP6"
    pub code_name: String,
    /// Code version, i.e. "6.3"
    pub version: String,
    /// Date and time run, and pc designator if available
    pub problem_id: String,
    /// dump number
    pub dump: u32,
    /// number of histories that were run
    pub nps: u64,
    /// number of pseudorandom numbers used
    pub random_numbers: u64,
}

/// Checks if the line can be parsed into the FirstLine struct
pub fn is_new_header(i: &str) -> bool {
    first_line(i).is_ok()
}

/// Parse whole line into a FirstLine struct
pub fn first_line(i: &str) -> IResult<&str, FirstLine> {
    let (i, code_name) = code_name(i)?;
    let (i, version) = version(i)?;
    let (i, probid) = problem_id(i)?;
    let (i, dump) = preceded(space1, uint32)(i)?;
    let (i, nps) = preceded(space1, uint64)(i)?;
    let (i, rnd) = preceded(space1, uint64)(i)?;

    Ok((
        i,
        FirstLine {
            code_name: code_name.trim().to_string(),
            version: version.trim().to_string(),
            problem_id: probid.trim().to_string(),
            dump,
            nps,
            random_numbers: rnd,
        },
    ))
}

/// Parse the number of tallies and perturbations
pub fn ntal_npert(i: &str) -> IResult<&str, (u32, u32)> {
    let (i, ntal) = ntal(i)?;
    let (i, npert) = opt(npert)(i)?;
    Ok((i, (ntal, npert.unwrap_or_default())))
}

/// Parse the number of tallies following the "ntal" tag
fn ntal(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("ntal"), uint32)(i)
}

/// Parse the number of perturbations following the "npert" tag
fn npert(i: &str) -> IResult<&str, u32> {
    preceded(tag_no_case("npert"), uint32)(i)
}

/// Parse the name of the code within 8 characters
fn code_name(i: &str) -> IResult<&str, &str> {
    split_index(i, 8)
}

/// Parse the version of the code within 8 characters
fn version(i: &str) -> IResult<&str, &str> {
    split_index(i, 8)
}

/// Parse the problem description within 19 characters
fn problem_id(i: &str) -> IResult<&str, &str> {
    split_index(i, 19)
}
