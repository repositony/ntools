// nom parser combinators
use nom::bytes::complete::tag_no_case;
use nom::IResult;

use crate::parsers::uint32;
use crate::Kcode;

/// Checks if the line begins with the "kcose" keyword for the new block
pub(in crate::parsers) fn is_new_kcode(i: &str) -> bool {
    i.trim_start().starts_with("kcode")
}

/// Parse whole line into a Kcode struct, leaving the results empty
pub(crate) fn kcode_header(i: &str) -> IResult<&str, Kcode> {
    let (i, _) = tag_no_case("kcode")(i)?;
    let (i, recorded_cycles) = uint32(i)?;
    let (i, settle_cycles) = uint32(i)?;
    let (i, variables_provided) = uint32(i)?;

    Ok((
        i,
        Kcode {
            recorded_cycles,
            settle_cycles,
            variables_provided,
            results: Vec::with_capacity(recorded_cycles as usize),
        },
    ))
}
