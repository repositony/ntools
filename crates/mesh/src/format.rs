/// Meshtal output formats, e.g. `COL`, `JK`, `CUV`...
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Format {
    /// Column data (MCNP default)
    ///
    /// The default MCNPv6.2 output format.
    ///
    /// Example:
    /// ```text
    ///  Energy      X      Y      Z     Result     Rel Error
    /// 1.111E+00 -0.500 -0.733 -2.625 7.25325E-03 1.20187E-02
    /// 1.111E+00 -0.500 -0.733 -0.875 3.43507E-02 4.71983E-03
    /// etc ...
    /// ```
    COL,
    /// Column data including voxel volume
    ///
    /// Same as column, but with extra information on voxel volumes. Since these
    /// data are derivable, volume information is descarded during parsing.
    ///
    /// Example:
    /// ```text
    ///  Energy      X      Y      Z     Result     Rel Error     Volume    Rslt * Vol
    /// 1.111E+00 -0.500 -0.733 -2.625 7.25325E-03 1.20187E-02 1.28333E+00 9.30834E-03
    /// 1.111E+00 -0.500 -0.733 -0.875 3.43507E-02 4.71983E-03 1.28333E+00 4.40834E-02
    /// etc ...
    /// ```
    CF,
    /// Cell-under-Voxel column data
    ///
    /// The UKAEA Cell under Voxel patch coerces all meshes, regradless of input
    /// deck specifications, to its own format. Multiple entries can correspond
    /// to the same voxel and voxels with only void cells can be omitted
    /// entirely. This is all handled in the background by the parsers.
    ///
    /// The cell number, matrial, and volume are parsed but not currently used
    /// while the main functionality is implemented.
    ///
    /// Example:
    /// ```text
    ///  Energy   Cell Mat  Density     Volume      X     Y       Z      Result   Rel Error
    /// 1.000E+35  76  6  8.00000E+00 4.47858E+02 0.697 9.000 -16.000 1.23957E-04 2.97900E-02
    /// 1.000E+35  84  6  8.00000E+00 5.06160E+00 0.697 9.000 -16.000 2.36108E-04 1.14448E-01
    /// etc ...
    /// ```
    CUV,
    /// 2D matrix of I (col) and J (row) data, grouped by K
    ///
    /// Matrix outputs separated into tables for two dimensions, and grouped by
    /// the third. For example, the IJ tables for
    /// [Geomtery::Rectangular](crate::geometry::Geometry) are X by Y, grouped by Z
    /// bins.
    ///
    /// ```text
    /// Energy Bin: 0.00E+00 - 1.11E+00 MeV
    /// Time Bin: -1.00E+36 - 0.00E+00 shakes
    ///   Z bin: -3.50  -  -1.75
    ///     Tally Results:  X (across) by Y (down)
    ///                   -0.50        0.50
    ///         -0.73   0.00000E+00 0.00000E+00
    ///          0.00   0.00000E+00 0.00000E+00
    ///          0.73   0.00000E+00 0.00000E+00
    ///     Relative Errors
    ///                   -0.50        0.50
    ///         -0.73   0.00000     0.00000
    ///          0.00   0.00000     0.00000
    ///          0.73   0.00000     0.00000
    /// ```
    IJ,
    /// 2D matrix of I (col) and K (row) data, grouped by J
    ///
    /// Matrix outputs separated into tables for two dimensions, and grouped by
    /// the third. For example, the IK tables for
    /// [Geomtery::Rectangular](crate::geometry::Geometry) are X by Z, grouped by Y
    /// bins.
    ///
    /// ```text
    /// Energy Bin: 0.00E+00 - 1.11E+00 MeV
    /// Time Bin: -1.00E+36 - 0.00E+00 shakes
    ///   Y bin: -3.50  -  -1.75
    ///     Tally Results:  X (across) by Z (down)
    ///                   -0.50        0.50
    ///         -0.73   0.00000E+00 0.00000E+00
    ///          0.00   0.00000E+00 0.00000E+00
    ///          0.73   0.00000E+00 0.00000E+00
    ///     Relative Errors
    ///                   -0.50        0.50
    ///         -0.73   0.00000     0.00000
    ///          0.00   0.00000     0.00000
    ///          0.73   0.00000     0.00000
    /// ```
    IK,
    /// 2D matrix of J (col) and K (row) data, grouped by I
    ///
    /// Matrix outputs separated into tables for two dimensions, and grouped by
    /// the third. For example, the JK tables for
    /// [Geomtery::Rectangular](crate::geometry::Geometry) are Z by Y, grouped by X
    /// bins.
    ///
    /// ```text
    /// Energy Bin: 0.00E+00 - 1.11E+00 MeV
    /// Time Bin: -1.00E+36 - 0.00E+00 shakes
    ///   X bin: -3.50  -  -1.75
    ///     Tally Results:  Y (across) by Z (down)
    ///                   -0.50        0.50
    ///         -0.73   0.00000E+00 0.00000E+00
    ///          0.00   0.00000E+00 0.00000E+00
    ///          0.73   0.00000E+00 0.00000E+00
    ///     Relative Errors
    ///                   -0.50        0.50
    ///         -0.73   0.00000     0.00000
    ///          0.00   0.00000     0.00000
    ///          0.73   0.00000     0.00000
    /// ```
    JK,
    #[default]
    /// Special case for unknown format or meshes with no output
    NONE,
}
