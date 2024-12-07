// nom parser combinators
use nom::bytes::complete::tag;
use nom::character::complete::{self, digit1, space0};
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many1;
use nom::number::complete::double;
use nom::sequence::{preceded, terminated};
use nom::IResult;

// List of consecutive unsigned integer values
pub(crate) fn vector_of_u32(i: &str) -> IResult<&str, Vec<u32>> {
    many1(terminated(complete::u32, space0))(i.trim_start())
}

/// List of consecutive doubles as a vector of f64 values
pub(crate) fn vector_of_f64(i: &str) -> IResult<&str, Vec<f64>> {
    many1(terminated(double, space0))(i.trim_start())
}

/// Signed integer value, trimming the start and ignoring `-` signs
pub(in crate::parsers) fn iint8(i: &str) -> IResult<&str, i8> {
    let (i, value) = recognize(preceded(opt(tag("-")), digit1))(i.trim_start())?;
    let (_, v) = complete::i8(value)?;
    Ok((i, v))
}

/// Unsigned 32-bit integer value, trimming preceding whitespace
pub(in crate::parsers) fn uint32(i: &str) -> IResult<&str, u32> {
    let (i, value) = digit1(i.trim_start())?;
    let (_, v) = complete::u32(value)?;
    Ok((i, v))
}

/// Unsigned 64-bit integer value, trimming preceding whitespace
pub(in crate::parsers) fn uint64(i: &str) -> IResult<&str, u64> {
    let (i, value) = digit1(i.trim_start())?;
    let (_, v) = complete::u64(value)?;
    Ok((i, v))
}

/// Unsigned size value, trimming preceding whitespace
pub(in crate::parsers) fn uint(i: &str) -> IResult<&str, usize> {
    let (i, value) = digit1(i.trim_start())?;
    let (_, v) = complete::u128(value)?;
    Ok((i, v as usize))
}
