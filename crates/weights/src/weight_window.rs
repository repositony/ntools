// standard library
use std::fs::File;
use std::io::{BufWriter, Write};

// ntools modules
use ntools_utils::{f, ValueExt};

// internal modules
use crate::operations::track_newlines;

/// Mesh-based global weight window data for WWINP/WWOUT/WWONE
///
/// The [WeightWindow] data structure represents a set of weight windows for a
/// single mesh and particle type.
///
/// Writers are implemented to generate the standardised UTF-8 file format for
/// WWINP/WWOUT/WWONE files for direct use in MCNP simulations.
///
/// ### Memory use
///
/// Large weight window meshes are approximately **~8 bytes** per unique voxel
///
/// ### Field naming
///
/// While unreadable, the [WeightWindow] fields correspond closely to the
/// internal FORTRAN MCNP variables outlined in the Appendicies of the
/// [MCNPv6.2](https://mcnp.lanl.gov/pdf_files/TechReport_2017_LANL_LA-UR-17-29981_WernerArmstrongEtAl.pdf)
/// and [MCNPv6.3](https://mcnpx.lanl.gov/pdf_files/TechReport_2022_LANL_LA-UR-22-30006Rev.1_KuleszaAdamsEtAl.pdf)
/// user manuals.
///
/// This really helps to maintain consistency with the underlying data.
///
/// ### Formatting
///
/// All formats will match fortran formats in the table below, explicitly using
/// the scientific number representation for any genral 'g' format.
///
/// | Block | Format         | Variable List                                    |
/// | ----- | -------------- | ------------------------------------------------ |
/// | 1     | 4i10, 20x, a19 | if iv ni nr probid                               |
/// | 1     | 7i10           | nt(1) ... nt(ni) [if iv=2]                       |
/// | 1     | 7i10           | ne(1) ... ne(ni)                                 |
/// | 1     | 6g13.5         | nfx nfy nfz x0 y0 z0                             |
/// | 1     | 6g13.5         | ncx ncy ncz nwg [if nr=10]                       |
/// | 1     | 6g13.5         | ncx ncy ncz x1 y1 z1 [if nr=16]                  |
/// | 1     | 6g13.5         | x2 y2 z2 nwg [if nr=16]                          |
/// | 2     | 6g13.5         | x0 (qx(i), px(i), sx(i), i=1,ncx)                |
/// | 2     | 6g13.5         | y0 (qy(i), py(i), sy(i), i=1,ncy)                |
/// | 2     | 6g13.5         | z0 (qz(i), pz(i), sz(i), i=1,ncz)                |
/// | 3     | 6g13.5         | t(i,1) ... t(i,nt(i)) [if nt(i)>1]               |
/// | 3     | 6g13.5         | e(i,1) ... e(i,ne(i))                            |
/// | 3     | 6g13.5         | (((w(i,j,k,l,1) j=1,nft), k=1,ne(i)), l=1,nt(i)) |
///
/// Formatting is done in blocks for consistency with the specifications
/// provided in the user manual appendices.
#[derive(Debug, Clone)]
pub struct WeightWindow {
    // Basic header info
    /// File type, manual states unused, so always 1.
    pub f: u8,
    /// Time-dependent windows flag, 1=no 2=yes.
    pub iv: u8,
    /// Number of particle types
    pub ni: u8,
    /// Number of energy bins for each particle type
    pub ne: usize,
    /// Number of time bins for each particle type
    pub nt: usize,
    /// Number of 'words' to describe mesh. rec=10, cyl/sph=16
    pub nr: u8,
    /// Mesh type 1=rec, 2=cyl, 3=sph
    pub nwg: u8,
    /// Problem description, 19-char string, can be blank.
    pub probid: String,

    // Fine mesh
    /// Total number of fine mesh points in i
    pub nfx: usize,
    /// Total number of fine mesh points in j
    pub nfy: usize,
    /// Total number of fine mesh points in k
    pub nfz: usize,

    // Coarse mesh
    /// Total number of coarse mesh points in i
    pub ncx: usize,
    /// Total number of coarse mesh points in j
    pub ncy: usize,
    /// Total number of coarse mesh points in k
    pub ncz: usize,

    // [ORIGIN] Corner of (x,y,z) rec, bottom center of (r,z,t) cyl, or center of (r,p,t) sph
    /// Origin i coordinate
    pub x0: f64,
    /// Origin j coordinate
    pub y0: f64,
    /// Origin k coordinate
    pub z0: f64,

    // [AXS] Vector from x0 y0 z0 to x1 y1 z1 defines (r,z,t) cylinder, or (r,p,t) polar axis
    /// Axis i coordinate
    pub x1: f64,
    /// Axis j coordinate
    pub y1: f64,
    /// Axis k coordinate
    pub z1: f64,

    // [VEC] Vector from x0 y0 z0 to x2 y2 z2 defines (r,z,t) cylinder, or (r,p,t) azimuthal axis
    /// Vec i coordinate
    pub x2: f64,
    /// Vec j coordinate
    pub y2: f64,
    /// Vec k coordinate
    pub z2: f64,

    // Energy and time bin boundaries
    /// Upper energy bounds for each particle type
    pub e: Vec<f64>,
    /// Upper time bounds for each particle type if nt(i)>1
    pub t: Vec<f64>,

    // Block 2 nonsense:
    // q = Fine mesh ratio (1 always) in each coarse mesh
    // p = Coarse mesh coordinates for (x,y,z), (r,z,t), or (r,p,t)
    // s = Number of fine meshes in each coarse mesh for (x,y,z), (r,z,t), or (r,p,t)
    /// List of (qx(i), px(i), sx(i)) tuples for i=1,ncx
    pub qps_x: Vec<[f64; 3]>,
    /// List of (qy(i), py(i), sy(i)) tuples for i=1,ncy
    pub qps_y: Vec<[f64; 3]>,
    /// List of (qz(i), pz(i), sz(i)) tuples for i=1,ncz
    pub qps_z: Vec<[f64; 3]>,

    // Actual weights
    /// Flattened vector of lower weights for each voxel, for each energy bin,
    /// for each time bin
    pub weights: Vec<f64>,

    /// Retain particle type for use in multi-particle sets
    pub particle: u8,
}

// Public API of useful stuff
impl WeightWindow {
    /// Write the weight window to the standard fromatted UTF-8 file
    ///
    /// Generates the WWINP/WWOUT/WWONE formatted files for direct input to MCNP
    /// simulations.
    ///
    /// Tools to combine weight window sets for multiple particles are provided
    /// see [write_multi_particle()](crate::write_multi_particle).
    pub fn write(&self, path: &str) {
        // assume fine >2 meshes for now
        let f = File::create(path).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        f.write_all(self.block_1_header().as_bytes()).unwrap();
        f.write_all(self.block_1().as_bytes()).unwrap();
        f.write_all(self.block_2().as_bytes()).unwrap();
        f.write_all(self.block_3().as_bytes()).unwrap();
    }

    /// Multiply all weights by a constant factor
    ///
    /// This is just a blanket multiplication that applies to every weight
    /// across all energy/time groups and particle types.
    ///
    /// ```rust
    /// # use ntools_weights::WeightWindow;
    /// let mut wwout = WeightWindow {
    ///     weights: vec![0.2, 0.15, 0.4],
    ///     ..Default::default()
    /// };
    ///
    /// // Double every weight
    /// wwout.scale(2.0);
    ///
    /// assert_eq!(wwout.weights, vec![0.4, 0.3, 0.8])
    /// ```
    pub fn scale(&mut self, factor: f64) {
        self.weights = self
            .weights
            .iter_mut()
            .map(|w| *w * factor)
            .collect::<Vec<f64>>();
    }

    /// Calculate number of non-zero weights
    ///
    /// Useful common sense value for checking the conversion to weights and
    /// that successive weight windows are imporving. Do not expect it to
    /// reach 100% if the mesh geometry covers any areas of zero importance for
    /// a given particle type.
    ///
    /// ```rust
    /// # use ntools_weights::WeightWindow;
    /// let wwout = WeightWindow {
    ///     weights: vec![0.2, 0.15, 0.0, 0.0],
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(wwout.non_analogue_percentage(), 50.0)
    /// ```
    pub fn non_analogue_percentage(&self) -> f64 {
        let non_zero = self.weights.iter().filter(|&v| *v != 0.0).count();
        100.0 * (non_zero as f64) / (self.weights.len() as f64)
    }

    /// Generate file content as a string (not for large files)
    ///
    /// Build a string for the full wwout file. Can be useful for small files
    /// and quick checks. However, this can end up duplicating a lot of data and
    /// the memory usage could be large for a significant number of weights.
    pub fn file_content(&self) -> String {
        let mut s = self.block_1_header();
        s += &self.block_1();
        s += &self.block_2();
        s += &self.block_3();
        s
    }

    /// Find the (e,t,i,j,k) indicies for a given cell index
    pub fn cell_index_to_etijk(&self, idx: usize) -> (usize, usize, usize, usize, usize) {
        // convenient values for readability
        let a: usize = self.nt * self.ncz * self.ncy * self.ncx;
        let b: usize = self.ncz * self.ncy * self.ncx;
        let c: usize = self.ncx * self.ncy;
        let d: usize = self.ncx;

        // find indicies in reverse (integer division floors in Rust)
        let e: usize = idx / a;
        let t: usize = (idx - e * a) / b;
        let k: usize = (idx - e * a - t * b) / c;
        let j: usize = (idx - e * a - t * b - k * c) / d;
        let i: usize = idx - e * a - t * b - k * c - j * d;

        (e, t, i, j, k)
    }

    /// Convert indexed bins to a voxel index
    pub fn etijk_to_voxel_index(
        &self,
        e_idx: usize,
        t_idx: usize,
        i_idx: usize,
        j_idx: usize,
        k_idx: usize,
    ) -> usize {
        let mut idx: usize = e_idx * (self.nt * self.ncx * self.ncy * self.ncz);
        idx += t_idx * (self.ncx * self.ncy * self.ncz);
        idx += i_idx * (self.ncy * self.ncz);
        idx += j_idx * (self.ncz);
        idx += k_idx;
        idx
    }

    /// Convert from a cell index to a voxel index
    ///
    /// Generally useful for weight windows to vtk plotting orders.
    pub fn cell_index_to_voxel_index(&self, idx: usize) -> usize {
        let (e, t, i, j, k) = self.cell_index_to_etijk(idx);
        self.etijk_to_voxel_index(e, t, i, j, k)
    }
}

impl WeightWindow {
    /// Only the formatted header String
    pub fn block_1_header(&self) -> String {
        // if iv ni nr probid
        let mut s = f!(
            "{:>10}{:>10}{:>10}{:>10}",
            self.f,
            self.iv,
            self.ni,
            self.nr,
        );

        // a19 so no longer than 19 characters
        let mut comment = self.probid.clone();
        comment.truncate(19);
        s += &f!("{comment}\n");

        // nt(1) ... nt(ni) [if iv=2]
        if self.iv == 2 {
            s += &f!("{:>10}\n", &self.nt);
        }

        // ne(1) ... ne(ni)
        s += &f!("{:>10}\n", &self.ne);
        s
    }

    /// Block 1 formatted String of common data for all combined window sets
    pub fn block_1(&self) -> String {
        // nfx nfy nfz x0 y0 z0
        let mut s = f!(
            "{:>13}{:>13}{:>13}",
            self.nfx.sci(5, 2),
            self.nfy.sci(5, 2),
            self.nfz.sci(5, 2)
        );
        s += &f!(
            "{:>13}{:>13}{:>13}\n",
            self.x0.sci(5, 2),
            self.y0.sci(5, 2),
            self.z0.sci(5, 2)
        );

        // ncx ncy ncz nwg          [if nr=10]
        // --------------- OR ----------------
        // ncx ncy ncz x1 y1 z1     [if nr=16]
        // x2 y2 z2 nwg             [if nr=16]
        s += &f!(
            "{:>13}{:>13}{:>13}",
            self.ncx.sci(5, 2),
            self.ncy.sci(5, 2),
            self.ncz.sci(5, 2)
        );

        match self.nwg {
            1 => s += &f!("{:>13}", self.nwg.sci(5, 2)),
            _ => {
                s += &f!(
                    "{:>13}{:>13}{:>13}\n",
                    self.x1.sci(5, 2),
                    self.y1.sci(5, 2),
                    self.z1.sci(5, 2)
                );
                s += &f!(
                    "{:>13}{:>13}{:>13}{:>13}",
                    self.x2.sci(5, 2),
                    self.y2.sci(5, 2),
                    self.z2.sci(5, 2),
                    self.nwg.sci(5, 2)
                )
            }
        }

        s += "\n";
        s
    }

    /// Block 2 formatted String containing all the coarse mesh bounds
    pub fn block_2(&self) -> String {
        // x0 ( qx(i), px(i), sx(i) ) from i=1,ncx
        let mut s = f!("{:>13}", self.x0.sci(5, 2));
        let mut count: u8 = 1;
        for x in &self.qps_x {
            s += &f!(
                "{:>13}{:>13}{}{:>13}",
                x[0].sci(5, 2),
                x[1].sci(5, 2),
                track_newlines(&mut count, 2),
                x[2].sci(5, 2)
            );
        }

        // y0 ( qy(i), py(i), sy(i) ) from i=1,ncy
        count = 1;
        s += &f!("\n{:>13}", self.y0.sci(5, 2));
        for y in &self.qps_y {
            s += &f!(
                "{:>13}{:>13}{}{:>13}",
                y[0].sci(5, 2),
                y[1].sci(5, 2),
                track_newlines(&mut count, 2),
                y[2].sci(5, 2)
            );
        }

        // z0 ( qz(i), pz(i), sz(i) ) from i=1,ncz
        count = 1;
        s += &f!("\n{:>13}", self.z0.sci(5, 2));
        for z in &self.qps_z {
            s += &f!(
                "{:>13}{:>13}{}{:>13}",
                z[0].sci(5, 2),
                z[1].sci(5, 2),
                track_newlines(&mut count, 2),
                z[2].sci(5, 2)
            );
        }

        // don't add a newline if it happens to end in one already
        if !s.ends_with('\n') {
            s += "\n";
        }
        s
    }

    /// Block 3 formatted String of weight windows for each voxel group
    pub fn block_3(&self) -> String {
        let mut s = String::new();
        let mut count: u8 = 1;

        // t(i,1) ... t(i,nt(i)) [if nt(i)>1]
        if self.t.len() > 1 {
            for t in &self.t {
                s += &f!("{:>13}", t.sci(5, 2));
                s += track_newlines(&mut count, 6);
            }
            s += "\n";
        }

        // e(i,1) ... e(i,ne(i))
        count = 1;
        for e in &self.e {
            s += &f!("{:>13}", e.sci(5, 2));
            s += track_newlines(&mut count, 6);
        }
        s += "\n";

        // w(i,j,k,l,1) where j=1,nft k=1,ne(i) l=1,nt(i)
        // textwrap extremely slow on large lines, just split as we go
        count = 1;
        for w in &self.weights {
            s += &f!("{:>13}", w.sci(5, 2));
            s += track_newlines(&mut count, 6);
        }

        // don't add a newline if it happens to end in one already
        if !s.ends_with('\n') {
            s += "\n";
        }
        s
    }
}

impl Default for WeightWindow {
    fn default() -> Self {
        Self {
            f: 1,
            iv: 1,
            ni: 1,
            ne: 1,
            nt: 1,
            nr: 10,
            nwg: 1,
            probid: String::new(),
            nfx: 0,
            nfy: 0,
            nfz: 0,
            ncx: 0,
            ncy: 0,
            ncz: 0,
            x0: 0.0,
            y0: 0.0,
            z0: 0.0,
            x1: 0.0,
            y1: 0.0,
            z1: 1.0,
            x2: 1.0,
            y2: 0.0,
            z2: 0.0,
            e: Vec::new(),
            t: Vec::new(),
            qps_x: Vec::new(),
            qps_y: Vec::new(),
            qps_z: Vec::new(),
            weights: Vec::new(),
            particle: 0,
        }
    }
}
