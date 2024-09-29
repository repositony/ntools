//! Library of parser functions

// crate modules
use crate::group::Group;
use crate::reader::{CellData, VoidRecord};
use crate::voxel::Voxel;

// ntools modules
use ntools_utils::f;

// external crates
use log::warn;

// nom parser combinators
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until1};
use nom::character::complete::{alpha1, char, digit1, one_of, space0, space1};
use nom::combinator::{map, map_parser, opt, recognize};
use nom::error::{Error, ErrorKind};
use nom::multi::{many1, many1_count};
use nom::number::complete::double;
use nom::sequence::{preceded, terminated, tuple};
use nom::{self, sequence, Err, IResult};

// ! Boolean checks
/// Check for a line ending with the `mesh tally.` tag
pub fn is_particle_type(i: &str) -> bool {
    i.trim_end().ends_with("mesh tally.")
}

/// Check for lines starting `origin at`
pub fn is_origin_axs_vec(i: &str) -> bool {
    i.starts_with("origin at")
}

/// Checks for a line starting `Energy/Time bin boundaries:`
pub fn is_group_bounds(i: &str) -> bool {
    group_bound_hint(i).is_ok()
}

/// Check for any coordiante tag (`R`, `Z`, etc...) followd by `direction`
pub fn is_geometry_bounds(i: &str) -> bool {
    geometry_bound_hint(i).is_ok()
}

/// Check for the `X` or `T` coordiante tags followd by `direction:`
pub fn is_meshtype_hint(i: &str) -> bool {
    meshtype_hint(i).is_ok()
}

/// Check for lines starting `Mesh Tally Number`
pub fn is_new_mesh(i: &str) -> bool {
    i.starts_with("Mesh Tally Number")
}

/// Check for `Cell` as part of the mesh header
pub fn is_cuv_hint(i: &str) -> bool {
    sequence::tuple((
        optional_energy_tag,
        space0,
        optional_time_tag,
        space0,
        cell_tag,
    ))(i)
    .is_ok()
}

/// Checks table headings for those expected in `COL` and `CF` style data  
pub fn is_col_hint(i: &str) -> bool {
    sequence::tuple((
        optional_energy_tag,
        space0,
        optional_time_tag,
        space0,
        one_of("XR"),
        space1,
        one_of("YZ"),
        space1,
        one_of("ZT"),
    ))(i)
    .is_ok()
}

/// Looks for any coordiante tag (`R`, `X`, etc...) followd by `bin`
///
/// Matrix table groups can be tagged `Theta bin (revolutions):`, but this is
/// not important. The types are ij, ik, jk, so the first hint the parser comes
/// across should never be the k-ordinate (theta).
pub fn is_matrix_hint(i: &str) -> bool {
    tuple((any_coordinate_tag, space1, tag("bin")))(i).is_ok()
}

/// Looks for any coordiante tag (`R`, `X`, etc...) followd by `bin`
/// todo remove one of these
pub fn is_new_table(i: &str) -> bool {
    tuple((any_coordinate_tag, space1, tag("bin")))(i).is_ok()
}

/// Checks for the next energy or time group
pub fn is_matrix_group(i: &str) -> bool {
    i.starts_with("Energy Bin:")
        | i.starts_with("Total Energy Bin")
        | i.starts_with("Time Bin:")
        | i.starts_with("Total Time Bin")
}

/// Checks for numbers in sequence
///
/// e.g.
/// ```text
/// -0.73 0.0
/// -0.73 0.00000E+00
/// ```
pub fn is_double_list(i: &str) -> bool {
    many1_count(terminated(double::<&str, ()>, space0))(i).is_ok()
}

/// Checks for lines starting with `Void_Record=`
pub fn is_voidoff_status(i: &str) -> bool {
    i.starts_with("Void_Record=")
}

/// Checks for an array of integers
pub fn is_material_array(i: &str) -> bool {
    !i.contains('.') && many1_count(terminated(digit1::<&str, ()>, space0))(i).is_ok()
}

/// Check for any a-z/A-Z character in the line
pub fn contains_alphabetic(i: &str) -> bool {
    i.chars().any(char::is_alphabetic)
}

// ! Parser combinators

/// Scientific number format e.g. -1.0e+03
pub fn scientific(i: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(one_of("-+")),
        digit1,
        opt(preceded(char('.'), digit1)),
        one_of("Ee"),
        opt(one_of("-+")),
        digit1,
    )))(i)
}

/// Parse values that should be scientific but are formatted incorrectly
///
/// This is generally just something dumb that fortran does for the CuV ouput
///
/// ```text
/// Energy     ...     Result   Rel Error
/// 1.047E-11  ...  8.15942-132 1.00000E+00
/// ```
///
/// If the exponent goes into triple digits the `E` will be dropped to make
/// room for the exponent and break the result/error values for parsers. This
/// alternative parser will do what it can to fix this when the issue is
/// detected but defaults to 0.0 with a warning if completely unsalvageable.
pub fn broken_scientific_f64(i: &str) -> IResult<&str, f64> {
    warn!("Fixing formatting for: \"{i}\"");
    let (i, value) = double(i)?;
    let (i, sign) = recognize(one_of("-+"))(i)?;
    let (i, exponent) = digit1(i)?;

    let number: f64 = f!("{value}e{sign}{exponent}").parse().unwrap_or_else(|_| {
        warn!("  Attempt failed, value set to 0.00000e+00");
        0.0
    });
    Ok((i, number))
}

/// Parse scientific numbers into an f64
fn scientific_as_f64(i: &str) -> IResult<&str, f64> {
    map_parser(scientific, double)(i)
}

/// Sequence of one or more a-z/A-Z characters (word)
pub fn first_word(i: &str) -> IResult<&str, &str> {
    alpha1(i)
}

/// Parse energy/time bounds into a vector of f64 values
pub fn group_bounds(i: &str) -> IResult<&str, Vec<f64>> {
    let (i, _) = take_until1(":")(i)?;
    let (i, _) = space1(&i[1..])?;
    vector_of_f64(i)
}

/// Parse mesh geometry bounds into a vector of f64 values
pub fn geometry_bounds(i: &str) -> IResult<&str, Vec<f64>> {
    let (i, _) = take_until1(":")(i)?;
    let (i, _) = space1(&i[1..])?;
    vector_of_f64(i)
}

/// Parse three numerical values following the `origin at` tag
pub fn origin(i: &str) -> IResult<&str, [f64; 3]> {
    let (i, _) = tag("origin at")(i.trim_start())?;
    coordinate_array(i)
}

/// Parse three numerical values following the `axis in` tag
pub fn axis(i: &str) -> IResult<&str, [f64; 3]> {
    let (i, _) = tag("axis in")(i.trim_start())?;
    coordinate_array(i)
}

/// Parse three numerical values following the `direction, VEC direction` tag
pub fn vec(i: &str) -> IResult<&str, [f64; 3]> {
    let (i, _) = tag("direction, VEC direction")(i.trim_start())?;
    coordinate_array(i)
}

/// Parse line of column data into a [Voxel]
pub fn column_type_voxel(i: &str) -> IResult<&str, Voxel> {
    let (i, _energy) = group(i)?;
    let (i, _) = space0(i)?;
    let (i, _time) = group(i)?;
    let (i, _) = space0(i)?;
    let (i, _i_coord) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, _j_coord) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, _k_coord) = double(i)?;
    let (i, _) = space0(i)?;
    let (i, result) = scientific_as_f64(i)?;
    let (i, _) = space0(i)?;
    let (i, error) = scientific_as_f64(i)?;

    Ok((
        i,
        Voxel {
            index: 0,
            result,
            error,
        },
    ))
}

/// Parse line of UKAEA Cell-under-Voxel data into a [Voxel]
/// and CellData struct
///
/// For now a lot of the data are thrown away to reduce memory requirements
/// significantly. However, the CuV is still a bit of a pain due to all the
/// cell and volume information required.
pub fn cuv_type_voxel(i: &str) -> IResult<&str, (Voxel, CellData)> {
    let (i, energy) = group(i)?;
    let (i, _) = space0(i)?;
    let (i, time) = group(i)?;
    let (i, _) = space0(i)?;
    let (i, cell) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, material) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, density) = scientific_as_f64(i)?; // guards against parsing material array lines as CuV data
    let (i, _) = space1(i)?;
    let (i, volume) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, i_coord) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, j_coord) = double(i)?;
    let (i, _) = space1(i)?;
    let (i, k_coord) = double(i)?;
    let (i, _) = space0(i)?;
    let (i, result) = alt((scientific_as_f64, broken_scientific_f64))(i)?;
    let (i, _) = space0(i)?;
    let (i, error) = alt((scientific_as_f64, broken_scientific_f64))(i)?;

    Ok((
        i,
        (
            Voxel {
                index: 0,
                result,
                error,
            },
            CellData {
                energy,
                time,
                i_coord,
                j_coord,
                k_coord,
                cell: cell as u32,         // cell number
                material: material as u32, // material number
                density,                   // material density
                volume,                    // cell volume
            },
        ),
    ))
}

/// Parse the number following a `Mesh Tally Number` tag to a u32
pub fn mesh_id(i: &str) -> IResult<&str, u32> {
    let (_, tally_id) = preceded(tuple((tag("Mesh Tally Number"), space1)), digit1)(i)?;
    nom::character::complete::u32(tally_id)
}

/// Parse void record `on` or `off` to a VoidRecord variant
pub fn void_record_status(i: &str) -> IResult<&str, VoidRecord> {
    let (i, _) = take_until1("=")(i)?;
    let (i, status) = on_or_off(&i[1..])?;

    match status.to_lowercase().as_str() {
        "on" => Ok((i, VoidRecord::On)),
        "off" => Ok((i, VoidRecord::Off)),
        _ => Err(Err::Error(Error::new("Not 'on' or 'off'", ErrorKind::Tag))),
    }
}

/// Recognise case-insensitive `on` or `off` tags
fn on_or_off(i: &str) -> IResult<&str, &str> {
    alt((tag_no_case("on"), tag_no_case("off")))(i)
}

/// Parse the `Total` time or energy to a [Group::Total](Group::Total)
fn total_group(i: &str) -> IResult<&str, Group> {
    map(tag_no_case("Total"), |_| Group::Total)(i)
}

/// Parse a scientific value to a [Group::Value(f64)](Group::Value(f64))
fn value_group(i: &str) -> IResult<&str, Group> {
    map(scientific_as_f64, Group::Value)(i)
}

#[allow(dead_code)]
/// Parse a decimal number to a [Group::Value(f64)](Group::Value(f64))
fn double_group(i: &str) -> IResult<&str, Group> {
    map(double, Group::Value)(i)
}

/// Parse scientific time or energy group data to the appropriate
/// [Group] variant
fn group(i: &str) -> IResult<&str, Group> {
    let (i, group) = opt(alt((total_group, value_group)))(i)?;
    match group {
        Some(g) => Ok((i, g)),
        None => Ok((i, Group::Total)),
    }
}

/// Optionally recognises `Energy` tag if it exists
fn optional_energy_tag(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag("Energy"))(i)
}

/// Optionally recognises `Time` tag if it exists
fn optional_time_tag(i: &str) -> IResult<&str, Option<&str>> {
    opt(tag("Time"))(i)
}

/// Optionally recognises `Cell` tag if it exists
fn cell_tag(i: &str) -> IResult<&str, &str> {
    tag("Cell")(i)
}

/// Recognizes `X`, `R`, `Y`, `Z`, or `Theta` tags
fn any_coordinate_tag(i: &str) -> IResult<&str, &str> {
    // one_of returns a char, tag a string which is needed for theta
    alt((tag("X"), tag("R"), tag("Y"), tag("Z"), tag("Theta")))(i)
}

/// Recognizes `Energy` or `Time` followed by `bin boundaries:`
fn group_bound_hint(i: &str) -> IResult<&str, &str> {
    let (i, _) = alt((tag("Energy"), tag("Time")))(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("bin boundaries:")(i)?;
    Ok((i, ""))
}

/// Parse any number of consecutive doubles into a vector of f64 values
fn vector_of_f64(i: &str) -> IResult<&str, Vec<f64>> {
    many1(terminated(double, space0))(i)
}

/// Parse any number of consecutive integers into a vector of f64 values
pub fn vector_of_u32(i: &str) -> IResult<&str, Vec<u32>> {
    many1(terminated(nom::character::complete::u32, space0))(i)
}

/// Parse any three numbers into an array
fn coordinate_array(i: &str) -> IResult<&str, [f64; 3]> {
    let (i, a) = double(i.trim_start())?;
    let (i, b) = double(i.trim_start())?;
    let (i, c) = double(i.trim_start())?;

    println!("i = {i}\n  -> [a, b, c] = {a} {b} {c}");

    Ok((i, [a, b, c]))
}

/// Recognise geometry bound data by any geometry tag `X`, `R`, etc... followed
/// by `direction`
fn geometry_bound_hint(i: &str) -> IResult<&str, &str> {
    let (i, (coordinate_tag, _, _)) = tuple((any_coordinate_tag, space1, tag("direction")))(i)?;
    Ok((i, coordinate_tag))
}

/// Recognise specifically the `X` or `R` tage followed by `direction:`
fn meshtype_hint(i: &str) -> IResult<&str, char> {
    terminated(one_of("XR"), tuple((space1, tag("direction:"))))(i)
}

#[allow(dead_code)]
/// Takes the u8 value terminated by an `e` tag
fn value_before_e(i: &str) -> IResult<&str, u8> {
    let (_, o) = take_until1("e")(i)?;
    nom::character::complete::u8(&o[o.len() - 1..o.len()])
}

#[cfg(test)]
mod boolean_tests {
    use super::*;

    #[test]
    fn test_geometry_bound_hint() {
        assert_eq!(is_geometry_bounds("X direction:     -1.00      0.00"), true);
        assert_eq!(is_geometry_bounds("Y direction:     -1.10     -0.37"), true);
        assert_eq!(is_geometry_bounds("Z direction:     -3.50      0.00"), true);
    }

    #[test]
    fn any_coordinate_tag_test() {
        // Uppercase for these values should all be true
        assert_eq!(any_coordinate_tag("X"), Ok(("", "X")));
        assert_eq!(any_coordinate_tag("R"), Ok(("", "R")));
        assert_eq!(any_coordinate_tag("Y"), Ok(("", "Y")));
        assert_eq!(any_coordinate_tag("Z"), Ok(("", "Z")));
        assert_eq!(any_coordinate_tag("Theta"), Ok(("", "Theta")));
        // the lowercase should fail
        assert!(any_coordinate_tag("x").is_err());
        assert!(any_coordinate_tag("r").is_err());
        assert!(any_coordinate_tag("y").is_err());
        assert!(any_coordinate_tag("z").is_err());
        assert!(any_coordinate_tag("theta").is_err());
        // 'T' alone should fail
        assert!(any_coordinate_tag("T").is_err());
    }

    #[test]
    fn geometry_bound_hint_test() {
        assert!(geometry_bound_hint("X direction: ").is_ok());
    }

    #[test]
    fn test_group_boundary_hints() {
        assert!(group_bound_hint("Energy  bin boundaries:").is_ok());
        assert!(group_bound_hint("Time      bin boundaries:").is_ok());

        // need the ':' to be sure this is a group bounds hint
        assert!(group_bound_hint("Energy bin boundaries").is_err());
        assert!(group_bound_hint("Time bin boundaries").is_err());

        assert!(group_bound_hint("Energybin boundaries:").is_err());
        assert!(group_bound_hint("time bin boundaries:").is_err());
    }

    #[test]
    fn test_broken_f64() {
        assert_eq!(
            broken_scientific_f64("1.111+001 2.222E+02"),
            Ok((" 2.222E+02", 11.11))
        );
        assert_eq!(
            broken_scientific_f64("1.111-001 2.222E+02"),
            Ok((" 2.222E+02", 0.1111))
        );
        assert_eq!(
            broken_scientific_f64("1.111-001-2.222E+02"),
            Ok(("-2.222E+02", 0.1111))
        );
        assert_eq!(
            broken_scientific_f64("+1.111-001+2.222+002"),
            Ok(("+2.222+002", 0.1111))
        );
    }
}
