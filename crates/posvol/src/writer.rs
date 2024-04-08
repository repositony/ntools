//! Write operations for Posvol data

// standard library
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// crate modules
use crate::error::Result;
use crate::posvol::Posvol;
use ntools_format::f;

/// Write raw [Posvol] data to an ascii text file
///
/// Any posvol file read into a [Posvol] type may be written to an ASCII file
/// for inspection or analysis.
///
/// This is a raw conversion with every value converted to ascii and written
/// directly to a text file with no formatting. For a more readable text file
/// use [write_ascii_pretty()] instead.
///
/// ```no_run
/// # use ntools_posvol::write_ascii;
/// # use ntools_posvol::read_posvol_file;
/// // Read the example file
/// let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
///
/// // Write a direct translation of the binary data to ASCII
/// write_ascii(&posvol, "./posvol.txt");
/// ```
pub fn write_ascii<P: AsRef<Path>>(posvol: &Posvol, path: P) -> Result<()> {
    let mut writer = init_writer(path)?;

    // write the block 1 information
    write!(writer, "24 ")?;
    write!(writer, "{} ", posvol.dimensions.res_x)?;
    write!(writer, "{} ", posvol.dimensions.res_y)?;
    write!(writer, "{} ", posvol.dimensions.res_z)?;
    write!(writer, "{} ", posvol.dimensions.n_x)?;
    write!(writer, "{} ", posvol.dimensions.n_y)?;
    write!(writer, "{} ", posvol.dimensions.n_z)?;
    write!(writer, "24 ")?;

    // write the block 2 information
    write!(writer, "{} ", posvol.number_of_cells())?;
    for cell in &posvol.cells {
        write!(writer, "{cell} ")?;
    }
    write!(writer, "{}", posvol.number_of_cells())?;

    Ok(())
}

/// Write [Posvol] data to a human readable text file
///
/// Any posvol file read into a [Posvol] type may be written to an ASCII file
/// for inspection or analysis.
///
/// This outputs the content of the [Posvol] to a human readable text format
/// with metadata for useful overall values to check at a glance. For a direct
/// conversion use [write_ascii()] instead.
///
/// ```no_run
/// # use ntools_posvol::write_ascii_pretty;
/// # use ntools_posvol::read_posvol_file;
/// // Read the example file
/// let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
///
/// // Write a human readable ascii text file
/// write_ascii_pretty(&posvol, "./posvol_pretty.txt");
/// ```
pub fn write_ascii_pretty<P: AsRef<Path>>(posvol: &Posvol, path: P) -> Result<()> {
    let mut writer = init_writer(path)?;

    // write the block 1 information
    writeln!(writer, "Total voxels: {}", posvol.number_of_voxels())?;
    writeln!(writer, "Total cells : {}", posvol.number_of_cells())?;
    writeln!(writer, "Mesh bounds in i: {}", posvol.dimensions.n_x)?;
    writeln!(writer, "Mesh bounds in j: {}", posvol.dimensions.n_y)?;
    writeln!(writer, "Mesh bounds in k: {}", posvol.dimensions.n_z)?;
    writeln!(writer, "Sample resolution i: {}", posvol.dimensions.res_x)?;
    writeln!(writer, "Sample resolution j: {}", posvol.dimensions.res_y)?;
    writeln!(writer, "Sample resolution k: {}", posvol.dimensions.res_z)?;

    // write the block 2 information
    for (i, subset) in posvol.subvoxels().iter().enumerate() {
        writeln!(writer, "\nVoxel[{i}] cells:")?;

        let s = subset
            .iter()
            .map(|cell| f!("{cell}"))
            .collect::<Vec<String>>()
            .join(" ");

        writeln!(writer, "{}", textwrap::fill(&s, 80))?;
    }

    Ok(())
}

/// Write [Posvol] data to a JSON file
///
/// Any posvol file read into a [Posvol] type may be written to JSON formats
/// for inspection or analysis.
///
/// This is a direct serialization to a JSON string of the dimensions extracted
/// from the file header, and a list of every cell for every sub-voxel region.
///
/// For a human readable text version see [write_ascii_pretty()], or use for a
/// direct conversion see [write_ascii()].
///
/// ```no_run
/// # use ntools_posvol::write_json;
/// # use ntools_posvol::read_posvol_file;
/// // Read the example file
/// let posvol = read_posvol_file("./data/posvol_example.bin").unwrap();
///
/// // Write a direct translation of the binary data to ASCII
/// write_json(&posvol, "./posvol.json");
/// ```
pub fn write_json<P: AsRef<Path>>(posvol: &Posvol, path: P) -> Result<()> {
    let writer = init_writer(path)?;
    serde_json::to_writer_pretty(writer, posvol)?;
    Ok(())
}

/// Initialise a reader from anything that can be turned into a path
fn init_writer<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>> {
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}
