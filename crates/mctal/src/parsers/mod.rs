// All nom parsers split amoung files for organisation
mod header;
mod kcode;
mod number;
mod tally;
mod tmesh;

// Internal re-exports for convenience
pub(crate) use header::*;
pub(crate) use kcode::*;
pub(crate) use number::*;
pub(crate) use tally::*;
pub(crate) use tmesh::*;

use log::{error, trace};

/// Data block types
pub(crate) enum Block {
    Header,
    Tally,
    Tmesh,
    Kcode,
    Blank,
}

/// Split a string slice at a specific index
pub(in crate::parsers) fn split_index(i: &str, n: usize) -> nom::IResult<&str, &str> {
    if n > i.len() {
        Err(cause("String slice not long enough to split on"))
    } else {
        Ok((&i[n..], &i[..n]))
    }
}

/// More convenient error creation for nom
use nom::error::{Error, ErrorKind};
pub(in crate::parsers) fn cause(s: &str) -> nom::Err<Error<&str>> {
    nom::Err::Error(Error::new(s, ErrorKind::Fail))
}

/// Find out if the line indicates a new data block
pub(crate) fn data_block(i: &str) -> nom::IResult<&str, Block> {
    if i.trim().is_empty() {
        Ok((i, Block::Blank))
    } else if tally::is_new_tally(i) {
        Ok((i, Block::Tally))
    } else if tmesh::is_new_tmesh(i) {
        Ok((i, Block::Tmesh))
    } else if kcode::is_new_kcode(i) {
        Ok((i, Block::Kcode))
    } else if header::is_new_header(i) {
        Ok((i, Block::Header))
    } else {
        Err(cause("line does not identify a new data block"))
    }
}
