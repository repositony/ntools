use crate::{Particle, TallyResult};

/// TMESH tally type data
///
/// ### Overview
///
/// MCNP6 offers two mesh tallies. The `TMESH` tally was developed for MCNPX,
/// and `FMESH` for MCNP5. These are very similar and both were kept when
/// merging MCNP5 and MCNPX into MCNP6.
///
/// For whatever reason, the TMESH data are also written to the MCTAL file.
/// These are also known in user manuals as "Superimposed Mesh Tally Type A".
///
/// ### Supported types
///
/// For reference, below are the supported types:
///
/// | Type | Description                  |
/// | ---- | ---------------------------- |
/// | 1    | Track-Averaged Mesh Tally    |
/// | 2    | Source Mesh Tally            |
/// | 3    | Energy Deposition Mesh Tally |
/// | 4    | DXTRAN Mesh Tally            |
///
#[derive(Debug, Default)]
pub struct Tmesh {
    /// Tally number
    pub id: u32,
    /// Particle type
    pub particles: Vec<Particle>,
    /// Tally type (pinhole, ring, detector, etc...)
    pub geometry: Geometry,

    /// Number of i bins (i.e. fmesh `iints`)
    pub n_cora: usize,
    /// Number of j bins (i.e. fmesh `jints`)
    pub n_corb: usize,
    /// Number of k bins (i.e. fmesh `kints`)
    pub n_corc: usize,

    /// Bin edges for i coordinate (i.e. fmesh `imesh`)
    pub cora: Vec<f64>,
    /// Bin edges for j coordinate (i.e. fmesh `jmesh`)
    pub corb: Vec<f64>,
    /// Bin edges for k coordinate (i.e. fmesh `kmesh`)
    pub corc: Vec<f64>,

    /// Number of voxels in the mesh
    pub n_voxels: usize,
    /// Number of cell or surface numbers
    pub n_region_bins: usize,
    /// Number of flagged bins (total vs. direct or flagged vs. unflagged)
    pub n_flagged_bins: usize,
    /// Number and type of user bins
    pub n_user_bins: usize,
    /// Number of segment bins
    pub n_segment_bins: usize,
    /// Number of multiplier bins
    pub n_multiplier_bins: usize,
    /// Number and type of multiplier bins
    pub n_cosine_bins: usize,
    /// Number and type of energy bins
    pub n_energy_bins: usize,
    /// Number and type of time bins
    pub n_time_bins: usize,

    /// All tally values read directly
    pub results: Vec<TallyResult>,
}

impl Tmesh {
    /// Calculate expected number of results from MCTAL bin records
    pub fn n_expected_results(&self) -> usize {
        let values = [
            self.n_voxels,
            self.n_flagged_bins,
            self.n_user_bins,
            self.n_segment_bins,
            self.n_multiplier_bins,
            self.n_cosine_bins,
            self.n_energy_bins,
            self.n_time_bins,
        ];
        // 0=unbounded but should be considered 1x bin
        values.iter().filter(|v| **v > 0).product()
    }
}

/// Mesh geometry types
///
/// In MCNP there are three primary mesh geometries:
/// - `Rectangular`
/// - `Cylindrical`
/// - `Spherical`
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Geometry {
    #[default]
    /// Cartesian (rec, xyz) mesh type
    Rectangular = 1,
    /// Cylindrical (cyl, rzt) mesh type
    Cylindrical = 2,
    /// Spherical (sph, rpt) mesh type
    Spherical = 3,
}

impl Geometry {
    /// Full name i.e. 'Rectangular', 'Cylindrical', 'Spherical'
    pub fn long_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "Rectangular",
            Geometry::Cylindrical => "Cylindrical",
            Geometry::Spherical => "Spherical",
        }
    }

    /// Shortened name i.e. 'Rec', 'Cyl', 'Sph'
    pub fn short_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "Rec",
            Geometry::Cylindrical => "Cyl",
            Geometry::Spherical => "Sph",
        }
    }

    /// Coordinate system based name i.e. 'XYZ', 'RZT', 'RPT'
    pub fn geometry_name(&self) -> &str {
        match self {
            Geometry::Rectangular => "XYZ",
            Geometry::Cylindrical => "RZT",
            Geometry::Spherical => "RPT",
        }
    }
}

impl std::fmt::Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.geometry_name())
    }
}
