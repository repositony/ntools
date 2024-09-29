/// Mesh geometry types, i.e. `Rectangular`, `Cylindrical`
///
/// Spherical is not currently implemented because everyone asked just questions
/// their existance in MCNP. This can be implemented if someone needs it.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Geometry {
    /// Cartesian (rec, xyz) mesh type
    Rectangular = 1,
    /// Cylindrical (cyl, rzt) mesh type
    Cylindrical = 2,
    // todo add spherical mesh type and implement into parsers etc...
    // Spherical (sph, rpt) mesh type
    // Spherical = 3
}

impl Geometry {
    /// Full name i.e. 'Rectangular', 'Cylindrical'
    pub fn long_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "Rectangular",
            Geometry::Cylindrical => "Cylindrical",
        }
    }

    /// Shortened name i.e. 'Rec', 'Cyl'
    pub fn short_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "Rec",
            Geometry::Cylindrical => "Cyl",
        }
    }

    /// Coordinate system based name i.e. 'XYZ', 'RZT'
    pub fn geometry_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "XYZ",
            Geometry::Cylindrical => "RZT",
        }
    }
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.geometry_name())
    }
}
