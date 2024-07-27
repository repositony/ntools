//! Simple read operations for plot_fmesh_xxx.bin binary files
//!
//! For generating fine cell-under-voxel plot meshes from the coarse mesh data.
//! The file is binary with a sequence of i32 signed integers assuming the
//! fortran default is used and little endian byte-ordering.

// standard library
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// crate modules
use crate::error::{Error, Result};
use crate::posvol::{Dimensions, Posvol};

// external crates
use bincode::deserialize;

/// Deserialise binary posvol file
///
/// Returns a Result containing a [Posvol] struct with all the information
/// extracted from a CuV posvol file at `path`.
///
/// ```rust
/// # use ntools_posvol::read_posvol_file;
/// // Read the example file
/// let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
///
/// // Print a summary of the data
/// println!("{posvol}");
/// ```  
pub fn read_posvol_file<P: AsRef<Path>>(path: P) -> Result<Posvol> {
    let mut reader = init_reader(path)?;

    let dimensions = parse_dimensions(&mut reader)?;
    let cells = parse_cell_data(&mut reader, &dimensions)?;

    Ok(Posvol { dimensions, cells })
}

/// Initialise a reader from anything that can be turned into a path
fn init_reader(path: impl AsRef<Path>) -> Result<BufReader<File>> {
    let file = File::open(path)?;
    Ok(BufReader::new(file))
}

/// Deserialise header to get the posvol dimensions
fn parse_dimensions(reader: &mut BufReader<File>) -> Result<Dimensions> {
    // `size_of` is less error prone but could just be 4
    let mut buffer = [0u8; std::mem::size_of::<i32>()];

    // read the first value, should be 24
    reader.read_exact(&mut buffer)?;
    if i32::from_ne_bytes(buffer) != 24 {
        return Err(Error::UnexpectedByteLength {
            expected: 24,
            found: i32::from_ne_bytes(buffer),
        });
    }

    // get the actual useful values, should be 6 of them
    let mut dim_buffer = [0u8; 6 * std::mem::size_of::<i32>()];
    reader.read_exact(&mut dim_buffer)?;
    let dimensions = deserialize(&dim_buffer)?;

    // skip the bookend '24'
    reader.read_exact(&mut buffer)?;
    Ok(dimensions)
}

/// Deserialise the data into a vector of cell values
fn parse_cell_data(reader: &mut BufReader<File>, dimensions: &Dimensions) -> Result<Vec<i32>> {
    let mut buffer = [0u8; std::mem::size_of::<i32>()];
    // next value will be the bytes to follow, use to check
    reader.read_exact(&mut buffer)?;

    // check to make sure it is the expected value
    let expected_length = dimensions.cell_array_byte_length() as i32;
    if i32::from_ne_bytes(buffer) != expected_length {
        return Err(Error::UnexpectedByteLength {
            expected: expected_length,
            found: i32::from_ne_bytes(buffer),
        });
    }

    // Collect the cell data together
    let mut cell_data = Vec::with_capacity(dimensions.number_of_subvoxels());
    for _ in 0..dimensions.number_of_cells() {
        reader.read_exact(&mut buffer)?;
        cell_data.push(i32::from_ne_bytes(buffer));
    }

    Ok(cell_data)
}
