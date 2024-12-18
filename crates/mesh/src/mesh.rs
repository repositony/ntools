//! Module for mesh-related data and implementations

// crate modules
use crate::error::{Error, Result};
use crate::format::Format;
use crate::geometry::Geometry;
use crate::group::Group;
use crate::particle::Particle;
use crate::point::{BoundaryTreatment, Point, PointKind};
use crate::voxel::{Voxel, VoxelCoordinate, VoxelSliceExt};

// ntools modules
use ntools_utils::{f, SliceExt, ValueExt};

// other crates
use log::warn;
use nalgebra::{Rotation, Vector3};

/// Common data structure representing a mesh tally
///
/// [Mesh] attributes correspond closely to MCNP input cards for consistency
/// and intuitive use. Units are unchanged from the MCNP defaults.
///
/// All parsed output formats are stored as a [Mesh] to provide a common
/// interface for all post-processing operations. For example: conversion to VTK
/// formats, weight window generation, data extraction, etc...
///
/// ## Terminology notes
///
/// #### I, J, K generics
///
/// Coordinate systems use different names i.e. (X,Y,Z) and (R,Z,Theta).
///
/// The generic (I,J,K) are used to represent all systems, in keeping with MCNP
/// user manuals.
///
/// #### Groups
///
/// A full set of (I,J,K) voxels are repeated for every time and energy bin in
/// the mesh.
///
/// It is also possible to have additional 'Total' groups if the `EMESH` or
/// `TMESH` cards contain multiple bins.
///
/// Time and energy bins are therefore often mapped to [Group::Value] and
/// [Group::Total] variants to be explicit when absolute values are handled
/// (See [Group] for details).
///
/// ## Examples
///
/// ### Reading meshtal files
///
/// Basic reading of files is very simple, regardless of format.
///
/// ```rust, no_run
/// # use ntools_mesh::{read, read_target, Mesh};
/// // Extract all meshes from a file into a Vec<Mesh>
/// let mesh_list = read("/path/to/meshtal.msht").unwrap();
///
/// // Extract just one target mesh from a file into a single Mesh
/// let mesh = read_target("/path/to/meshtal.msht", 104).unwrap();
/// ```
///
/// All the parsing and interpretation are done for you, and the data are in a
/// common [Mesh] type. This means that all [Mesh] methods are available for any
/// format mesh of any geometry type.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// Mesh tally number e.g fmesh104 => id = 104
    pub id: u32,
    /// Mesh geometry type, usually rectangular for MCNP default
    pub geometry: Geometry,
    /// Name of the particle type
    pub particle: Particle,
    /// i mesh boundaries
    pub imesh: Vec<f64>,
    /// Number of voxels in i
    pub iints: usize,
    /// j mesh boundaries
    pub jmesh: Vec<f64>,
    /// Number of voxels in j
    pub jints: usize,
    /// k mesh boundaries
    pub kmesh: Vec<f64>,
    /// Number of voxels in j
    pub kints: usize,
    /// Energy bins
    pub emesh: Vec<f64>,
    /// Number of energy bins, EXCLUDING 'total' group
    pub eints: usize,
    /// Time bins \[shakes\]
    pub tmesh: Vec<f64>,
    /// Number of time bins, EXCLUDING 'total' group
    pub tints: usize,
    /// ORIGIN card, [0.0, 0.0, 0.0] for MCNP default
    pub origin: [f64; 3],
    /// AXS card, [0.0, 0.0, 1.0] for MCNP default
    pub axs: [f64; 3],
    /// VEC card, [1.0, 0.0, 0.0] for MCNP default
    pub vec: [f64; 3],
    ///  List of every `Voxel` in the mesh
    pub voxels: Vec<Voxel>,
    /// Detected output format in MESHTAL file
    pub format: Format,
}

/// Common methods
impl Mesh {
    /// Initialise new mesh with known id
    ///
    /// The `id` is the tally number used on the `FMESH` card in the input deck.
    /// For example, Fmesh204:n => id=204. This will initialise a [Mesh] with
    /// all of the default values.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Multiply all voxel results by a constant factor
    ///
    /// Uncertanties are relative and are therfore unaffected.
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh,Voxel};
    /// // Simple test mesh with three voxels
    /// let mut mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel {result: 12.0, ..Default::default()},
    ///         Voxel {result: 18.0, ..Default::default()},
    ///         Voxel {result:  4.0, ..Default::default()},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// // Scale results by a factor of x0.5 (errors relative, thus unchanged)
    /// mesh.scale(0.5);
    /// assert_eq!(mesh.voxels[0].result, 6.0);
    /// assert_eq!(mesh.voxels[1].result, 9.0);
    /// assert_eq!(mesh.voxels[2].result, 2.0);
    /// ```
    pub fn scale(&mut self, factor: f64) {
        self.voxels.iter_mut().for_each(|v| v.result *= factor);
    }

    /// Translate all coordinates by (x, y, z)
    ///
    /// Simply updates the relevant origin coordiantes and mesh geometry bounds
    /// using the cartesian values provided. For cylindrical meshes the voxel
    /// bounds will be unaffected.
    pub fn translate(&mut self, x: f64, y: f64, z: f64) {
        // origin always moves for rec, cyl
        self.origin[0] += x;
        self.origin[1] += y;
        self.origin[2] += z;

        if self.geometry == Geometry::Rectangular {
            // all corrdinates and boundaries change for rectangular
            self.imesh = self.imesh.iter().map(|c| c + x).collect();
            self.jmesh = self.jmesh.iter().map(|c| c + y).collect();
            self.kmesh = self.kmesh.iter().map(|c| c + z).collect();
        }
    }

    /// Returns the number of energy bins
    ///
    /// This will include the `Total` bin in the count for tallies with
    /// multiple energy bins defined on the EMESH card.
    ///
    /// | Card values   | #Groups               |
    /// | ------------- | --------------------- |
    /// | None          | 1                     |
    /// | 0.0 100       | 1                     |
    /// | 0.0 20 100    | 3 (2 + 'Total' bin)   |
    /// | 0.0 20 100 T  | 3 (2 + 'Total' bin)   |
    pub fn n_ebins(&self) -> usize {
        if self.eints > 1 {
            self.eints + 1
        } else {
            1
        }
    }

    /// Returns the number of time bins
    ///
    /// This will include the 'Total' bin in the count for tallies with
    /// multiple time bins defined on the TMESH card. Defaults to 1 `Total` bin
    /// if none are explicitly defined.
    ///
    /// | Card values       | #Groups               |
    /// | ----------------- | --------------------- |
    /// | None              | 1                     |
    /// | 0.0 1e16          | 1                     |
    /// | 0.0 1e16 1e36     | 3 (2 + 'Total' bin)   |
    /// | 0.0 1e16 1e36 T   | 3 (2 + 'Total' bin)   |
    pub fn n_tbins(&self) -> usize {
        if self.tints > 1 {
            self.tints + 1
        } else {
            1
        }
    }

    /// Find the maximum (`value`, `error`) in the mesh
    ///
    /// Will `panic!` when the mesh contains no voxel data. Use
    /// [try_maximum()](Mesh::try_maximum) to handle this case explicitly.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(mesh.maximum(), (3.0, 0.3));
    /// ```
    ///
    /// To find the maximum of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.maximum_voxel().unwrap()       , &voxels[2]);
    /// assert_eq!(voxels.maximum_result_error().unwrap(), (3.0, 0.3));
    /// ```
    pub fn maximum(&self) -> (f64, f64) {
        self.try_maximum().expect("The mesh contains no voxels")
    }

    /// Find the maximum (`value`, `error`) in the mesh
    ///
    /// Will fail if `mesh.voxels` is empty. The [maximum()](Mesh::maximum) will
    /// simply allow this case to `panic!`.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mut mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// // Successful example
    /// assert_eq!(mesh.try_maximum().unwrap(), (3.0, 0.3));
    ///
    /// // Failure example for empty mesh.voxels
    /// mesh.voxels.clear();
    /// assert!(mesh.try_maximum().is_err());
    /// ```
    ///
    /// To find the maximum of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.maximum_voxel().unwrap()       , &voxels[2]);
    /// assert_eq!(voxels.maximum_result_error().unwrap(), (3.0, 0.3));
    /// ```
    pub fn try_maximum(&self) -> Result<(f64, f64)> {
        self.voxels.maximum_result_error()
    }

    /// Find the minimum (`value`, `error`) in the mesh
    ///
    /// Will `panic!` when the mesh contains no voxel data. Use
    /// [try_minimum()](Mesh::try_minimum) to handle this case explicitly.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(mesh.minimum(), (1.0, 0.1));
    /// ```
    ///
    /// To find the minimum of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.minimum_voxel().unwrap()       , &voxels[0]);
    /// assert_eq!(voxels.minimum_result_error().unwrap(), (1.0, 0.1));
    /// ```
    pub fn minimum(&self) -> (f64, f64) {
        self.try_minimum().expect("The mesh contains no voxels")
    }

    /// Find the minimum (`value`, `error`) in the mesh
    ///
    /// Will fail if `mesh.voxels` is empty. The [minimum()](Mesh::minimum) will
    /// simply allow this case to `panic!`.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mut mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// // Successful example
    /// assert_eq!(mesh.try_minimum().unwrap(), (1.0, 0.1));
    ///
    /// // Failure example for empty mesh.voxels
    /// mesh.voxels.clear();
    /// assert!(mesh.try_minimum().is_err());
    /// ```
    ///
    /// To find the minimum of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.minimum_voxel().unwrap()       , &voxels[0]);
    /// assert_eq!(voxels.minimum_result_error().unwrap(), (1.0, 0.1));
    /// ```
    pub fn try_minimum(&self) -> Result<(f64, f64)> {
        self.voxels.minimum_result_error()
    }

    /// Find the averege (`value`, `error`) in the mesh
    ///
    /// Will `panic!` when the mesh contains no voxel data. Use
    /// [try_average()](Mesh::try_average) to handle this case explicitly.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(mesh.average(), (2.0, 0.49497474683058323));
    /// ```
    ///
    /// To find the average of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.average_result_error().unwrap(), (2.0, 0.49497474683058323));
    /// ```
    pub fn average(&self) -> (f64, f64) {
        self.try_average().expect("The mesh contains no voxels")
    }

    /// Find the average (`value`, `error`) in the mesh
    ///
    /// Will fail if `mesh.voxels` is empty. The [average()](Mesh::average) will
    /// simply allow this case to `panic!`.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mut mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// // Successful example
    /// assert_eq!(mesh.try_average().unwrap(), (2.0, 0.49497474683058323));
    ///
    /// // Failure example for empty mesh.voxels
    /// mesh.voxels.clear();
    /// assert!(mesh.try_average().is_err());
    /// ```
    ///
    /// To find the average of any collection of voxels, the [VoxelSliceExt]
    /// trait is implemented for convenience.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Voxel, VoxelSliceExt};
    /// let voxels = vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3} ];
    ///
    /// assert_eq!(voxels.average_result_error().unwrap(), (2.0, 0.49497474683058323));
    /// ```
    pub fn try_average(&self) -> Result<(f64, f64)> {
        self.voxels.average_result_error()
    }
}

/// Point method implementations for the Mesh type
impl Mesh {
    /// Find the result at a [Point]
    ///
    /// Results are averaged between adjacent voxels when the point is on a
    /// boundary. Points outside the mesh return `None`.
    ///
    /// A small tolerance is granted for detecting points on boundaries. By
    /// default this is set to within 0.01% of the voxel length in each axis.
    ///
    /// For example, for a voxel spanning 0.0 - 1.0 in the x-axis, a Point with
    /// x = 0.999 is considered to be on the boundary. The result will therefore
    /// be the avaerage of this and the appropriate adjacent voxel.
    pub fn find_point_data(&self, point: Point, boundary: BoundaryTreatment) -> Option<(f64, f64)> {
        match self.find_point_voxels(point, boundary) {
            Ok(voxels) => {
                // average the voxels if multiple
                let result = voxels.iter().map(|v| v.result).sum::<f64>() / (voxels.len() as f64);

                // sum errors in quadrature
                let relative_error = voxels
                    .iter()
                    .map(|v| v.absolute_error().powi(2))
                    .sum::<f64>()
                    .sqrt()
                    / result;

                Some((result, relative_error))
            }
            _ => None,
        }
    }

    /// Find the results for a list of [Point]s
    ///
    /// Equivalent to looping over
    /// [find_point_data()](crate::mesh::Mesh::find_point_data) in a loop for
    /// multiple points, collecting the results to a vector.
    ///
    /// Results are averaged between adjacent voxels when the point is on a
    /// boundary. Points outside the mesh return `None`.
    ///
    /// See [find_point_data()](crate::mesh::Mesh::find_point_data) for details.
    pub fn find_points_data(
        &self,
        points: &[Point],
        boundary: BoundaryTreatment,
    ) -> Vec<Option<(f64, f64)>> {
        points
            .iter()
            .map(|p| self.find_point_data(p.clone(), boundary))
            .collect()
    }

    /// Find all relevant voxels corresponding to a [Point]
    ///
    /// Try to find all adjacent voxels that a [Point] touches.
    ///
    /// A small tolerance may be granted for detecting points on boundaries. For
    /// example, `tol=0.01` would consider points to be on a boudary if within
    /// 1% of the total voxel length for each axis.
    pub fn find_point_voxels(
        &self,
        point: Point,
        boundary: BoundaryTreatment,
    ) -> Result<Vec<Voxel>> {
        // convert into the correct geometry
        let point = self.coerce_point_kind(&point);

        // check if point valid
        self.is_point_valid(&point)?;

        let e = self.energy_index_from_group(point.e)?;
        let t = self.time_index_from_group(point.t)?;
        let mut voxels = Vec::with_capacity(8);

        // at this point we know the point is valid and in the right geometry
        match point.kind {
            PointKind::Index => {
                let index = self.voxel_index_from_etijk(
                    e,
                    t,
                    point.i as usize,
                    point.j as usize,
                    point.k as usize,
                );
                voxels.push(self.voxels[index])
            }
            _ => match boundary {
                BoundaryTreatment::Average(tol) => {
                    for i in &self.imesh.find_bin_average(point.i, tol)? {
                        for j in &self.jmesh.find_bin_average(point.j, tol)? {
                            for k in &self.kmesh.find_bin_average(point.k, tol)? {
                                let index = self.voxel_index_from_etijk(e, t, *i, *j, *k);
                                voxels.push(self.voxels[index])
                            }
                        }
                    }
                }
                BoundaryTreatment::Lower => {
                    let i = &self.imesh.find_bin_exclusive(point.i)?;
                    let j = &self.jmesh.find_bin_exclusive(point.j)?;
                    let k = &self.kmesh.find_bin_exclusive(point.k)?;
                    let index = self.voxel_index_from_etijk(e, t, *i, *j, *k);
                    voxels.push(self.voxels[index])
                }
                BoundaryTreatment::Upper => {
                    let i = &self.imesh.find_bin_inclusive(point.i)?;
                    let j = &self.jmesh.find_bin_inclusive(point.j)?;
                    let k = &self.kmesh.find_bin_inclusive(point.k)?;
                    let index = self.voxel_index_from_etijk(e, t, *i, *j, *k);
                    voxels.push(self.voxels[index])
                }
            },
        }

        Ok(voxels)
    }
}

/// Voxels and voxel slicing
impl Mesh {
    /// Returns the number of voxels
    pub fn n_voxels(&self) -> usize {
        self.voxels.len()
    }

    /// Calculate how many voxels there should be
    ///
    /// Useful for common sense checking against the value returned by a
    /// `mesh.voxels.len()` call. This is the product of the number of energy
    /// bins, time bins, and fine mesh bins in each dimension.
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh};
    /// // Generate a simple test mesh
    /// let mesh = Mesh {
    ///     eints: 1,   // 1 energy group (total only)
    ///     tints: 2,   // 3 time groups (2+total)
    ///     iints: 2,
    ///     jints: 3,
    ///     kints: 4,
    ///     ..Default::default()
    /// };
    /// // Calculate the coordinates for the voxel
    /// assert_eq!(mesh.n_voxels_expected(), 72);
    pub fn n_voxels_expected(&self) -> usize {
        self.n_ebins() * self.n_tbins() * self.iints * self.jints * self.kints
    }

    /// Expected number of voxels per group set
    ///
    /// Returns the product of the number of fine mesh bins in each dimension.
    /// This is equivalent to the total number of voxels in the mesh geometry
    /// itself. For example, a 5x5x5 rectangular mesh should have 100 voxels
    /// for every unique energy/time group set.
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh};
    /// // Generate a simple test mesh
    /// let mesh = Mesh {
    ///     iints: 2,
    ///     jints: 3,
    ///     kints: 4,
    ///     ..Default::default()
    /// };
    /// // Calculate the coordinates for the voxel
    /// assert_eq!(mesh.n_voxels_per_group(), 24);
    /// ```
    pub fn n_voxels_per_group(&self) -> usize {
        self.iints * self.jints * self.kints
    }

    /// Get a full set of coordinates for a voxel
    ///
    ///
    /// ```rust
    /// # use ntools_mesh::{read_target, Mesh, VoxelCoordinate, Group};
    /// // Get some voxel from an example file
    /// let mesh = read_target("./data/meshes/fmesh_114.msht", 114).unwrap();
    /// // Calculate the coordinates for the voxel
    /// assert_eq!( mesh.voxel_coordinates(63).unwrap(),
    ///             VoxelCoordinate {
    ///                 energy: Group::Value(1.0),
    ///                 time: Group::Value(1.0E+30),
    ///                 i: 9.375,
    ///                 j: 4.500,
    ///                 k: 2.500} );
    /// ```
    pub fn voxel_coordinates(&self, index: usize) -> Result<VoxelCoordinate> {
        if index > self.voxels.len() {
            return Err(Error::IndexOutOfBounds {
                minimum: 0,
                maximum: self.voxels.len(),
                actual: index,
            });
        }

        // get the indicies
        let (e_idx, t_idx, i_idx, j_idx, k_idx) = self.etijk_from_voxel_index(index);

        Ok(VoxelCoordinate {
            energy: self.energy_group_from_index(e_idx)?,
            time: self.time_group_from_index(t_idx)?,
            i: (self.imesh[i_idx + 1] + self.imesh[i_idx]) / 2.0,
            j: (self.jmesh[j_idx + 1] + self.jmesh[j_idx]) / 2.0,
            k: (self.kmesh[k_idx + 1] + self.kmesh[k_idx]) / 2.0,
        })
    }

    /// Collect (`value`, `error`) pairs for all voxels in the mesh
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Voxel};
    /// let mesh = Mesh {
    ///     voxels: vec![
    ///         Voxel{index: 0, result: 1.0, error: 0.1},
    ///         Voxel{index: 1, result: 2.0, error: 0.2},
    ///         Voxel{index: 2, result: 3.0, error: 0.3},
    ///     ],
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(mesh.voxel_data(), vec![(1.0, 0.1), (2.0, 0.2), (3.0, 0.3)]);
    /// ```
    pub fn voxel_data(&self) -> Vec<(f64, f64)> {
        self.voxels.collect_result_error()
    }

    /// Slice the full list of mesh Voxels by energy/time index
    ///
    /// Very fast, but operates on indicies and therefore relies on the voxels
    /// being sorted. For an arbitrary list of [Voxel](crate::mesh::Voxel)s,
    /// use the filter methods which explicitly check the groups of every voxel
    /// provided.
    pub fn voxels_by_group_index(&self, e_idx: usize, t_idx: usize) -> Result<&[Voxel]> {
        // Just quickly make sure the values given are reasonable
        if e_idx > self.n_ebins() {
            return Err(Error::IndexOutOfBounds {
                minimum: 0,
                maximum: self.n_ebins(),
                actual: e_idx,
            });
        } else if t_idx > self.n_tbins() {
            return Err(Error::IndexOutOfBounds {
                minimum: 0,
                maximum: self.n_tbins(),
                actual: t_idx,
            });
        };

        // slice voxels down to the full energy group
        let group_size = self.n_tbins() * self.iints * self.jints * self.kints;
        let start = e_idx * group_size;
        let end = start + group_size;
        let voxels = &self.voxels[start..end];

        // slice again down the right time group
        let group_size = self.iints * self.jints * self.kints;
        let start = t_idx * group_size;
        let end = start + group_size;

        // is there a more concise way? probably, but it's late and this makes sense
        Ok(&voxels[start..end])
    }

    /// Slice the full list of mesh Voxels by both energy/time groups
    ///
    /// Very fast, but operates on indicies and therefore relies on the voxels
    /// being sorted. For an arbitrary list of [Voxel](crate::mesh::Voxel)s,
    /// use the filter methods which explicitly check the groups of every voxel
    /// provided.
    pub fn voxels_by_group_value(
        &self,
        energy_group: Group,
        time_group: Group,
    ) -> Result<&[Voxel]> {
        let e_idx = self.energy_index_from_group(energy_group)?;
        let t_idx = self.time_index_from_group(time_group)?;

        let group_size = self.n_tbins() * self.iints * self.jints * self.kints;
        let start = e_idx * group_size;
        let end = start + group_size;
        let voxels = &self.voxels[start..end];

        let group_size = self.iints * self.jints * self.kints;
        let start = t_idx * group_size;
        let end = start + group_size;
        Ok(&voxels[start..end])
    }
}

impl Mesh {
    /// Returns slice of `emesh` for upper energy bin edges
    ///
    /// ```rust
    /// # use ntools_mesh::Mesh;
    /// let mesh = Mesh {
    ///     emesh: vec![0.0, 1.0, 2.0, 3.0, 4.0],
    ///     ..Default::default()
    /// };
    /// // Get a slice of the upper edges of energy bins
    /// assert_eq!(mesh.energy_bins_upper(), vec![1.0, 2.0, 3.0, 4.0]);
    /// ```
    pub fn energy_bins_upper(&self) -> &[f64] {
        &self.emesh[1..]
    }

    /// Returns slice of `emesh` for lower energy bin edges
    ///
    /// ```rust
    /// # use ntools_mesh::Mesh;
    /// let mesh = Mesh {
    ///     emesh: vec![0.0, 1.0, 2.0, 3.0, 4.0],
    ///     ..Default::default()
    /// };
    /// // Get a slice of the lower edges of energy bins
    /// assert_eq!(mesh.energy_bins_lower(), vec![0.0, 1.0, 2.0, 3.0]);
    /// ```
    pub fn energy_bins_lower(&self) -> &[f64] {
        let end = self.emesh.len() - 1;
        &self.emesh[0..end]
    }

    /// Returns a collection of all energy groups, including total
    ///
    /// Builds a full list of the energy groups in the mesh, which can include
    /// both [Group](crate::mesh::Group) variants.
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh,Group};
    /// let mesh = Mesh {
    ///     eints: 2,
    ///     emesh: vec![0.0, 1.0, 2.0],
    ///     ..Default::default()
    /// };
    /// // See what energy groups the voxels may be split into
    /// assert_eq!(mesh.energy_groups(), vec![Group::Value(1.0), // bin 0.0 - 1.0
    ///                                       Group::Value(2.0), // bin 1.0 - 2.0
    ///                                       Group::Total]);
    /// ```
    pub fn energy_groups(&self) -> Vec<Group> {
        if self.n_ebins() > 1 {
            let mut groups = self
                .energy_bins_upper()
                .iter()
                .map(|energy| Group::Value(*energy))
                .collect::<Vec<Group>>();
            groups.push(Group::Total);
            groups
        } else {
            vec![Group::Total]
        }
    }

    // /// Returns a collection of `emesh` Value() groups, ignoring any 'Total'
    // ///
    // /// Builds a list of only the energy groups with a value from `emesh`, and
    // /// will only include the [Group::Value](crate::mesh::Group) variant.
    // ///
    // /// ```rust
    // /// # use ntools_mesh::{Mesh,Group};
    // /// let mesh = Mesh {
    // ///     eints: 2,
    // ///     emesh: vec![0.0, 1.0, 2.0],
    // ///     ..Default::default()
    // /// };
    // /// // List of emesh groups
    // /// assert_eq!(mesh.emesh_groups(), vec![Group::Value(1.0),     // bin 0.0 - 1.0
    // ///                                      Group::Value(2.0)]);   // bin 1.0 - 2.0
    // /// ```
    // pub fn emesh_groups(&self) -> Vec<Group> {
    //     self.energy_bins_upper()
    //         .iter()
    //         .map(|energy| Group::Value(*energy))
    //         .collect::<Vec<Group>>()
    // }

    /// Returns slice of `tmesh` for upper time bin edges
    ///
    /// ```rust
    /// # use ntools_mesh::Mesh;
    /// let mesh = Mesh {
    ///     tmesh: vec![0.0, 1e12, 1e16, 1e20],
    ///     ..Default::default()
    /// };
    /// // Get a slice of the upper edges of time bins
    /// assert_eq!(mesh.time_bins_upper(), vec![1e12, 1e16, 1e20]);
    /// ```
    pub fn time_bins_upper(&self) -> &[f64] {
        if let Some((_, elements)) = self.tmesh.split_first() {
            elements
        } else {
            &self.tmesh
        }
    }

    /// Returns slice of `emesh` for lower time bin edges
    ///
    /// ```rust
    /// # use ntools_mesh::Mesh;
    /// let mesh = Mesh {
    ///     tmesh: vec![0.0, 1e12, 1e16, 1e20],
    ///     ..Default::default()
    /// };
    /// // Get a slice of the lower edges of time bins
    /// assert_eq!(mesh.time_bins_lower(), vec![0.0, 1e12, 1e16]);
    /// ```
    pub fn time_bins_lower(&self) -> &[f64] {
        if let Some((_, elements)) = self.tmesh.split_last() {
            elements
        } else {
            &self.tmesh
        }
    }

    /// Returns a collection of all time groups, including total
    ///
    /// Builds a full list of the time groups in the mesh, which can include
    /// both [Group](crate::mesh::Group) variants.
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh,Group};
    /// let mesh = Mesh {
    ///     tints: 2,
    ///     tmesh: vec![0.0, 1e12, 1e16],
    ///     ..Default::default()
    /// };
    /// // See what time groups the voxels may be split into
    /// assert_eq!(mesh.time_groups(), vec![Group::Value(1e12), // bin 0.0  - 1e12
    ///                                     Group::Value(1e16), // bin 1e12 - 1e16
    ///                                     Group::Total]);     // 'total' group
    /// ```
    pub fn time_groups(&self) -> Vec<Group> {
        if self.n_tbins() > 1 {
            let mut groups = self
                .time_bins_upper()
                .iter()
                .map(|time| Group::Value(*time))
                .collect::<Vec<Group>>();
            groups.push(Group::Total);
            groups
        } else {
            vec![Group::Total]
        }
    }

    // /// Returns a collection of `tmesh` Value() groups, ignoring any 'Total'
    // ///
    // /// Builds a list of only the time groups with a value from `tmesh`, and
    // /// will only include the [Group::Value](crate::mesh::Group) variant.
    // ///
    // /// ```rust
    // /// # use ntools_mesh::{Mesh,Group};
    // /// let mesh = Mesh {
    // ///     tints: 2,
    // ///     tmesh: vec![0.0, 1e12, 1e16],
    // ///     ..Default::default()
    // /// };
    // /// // List of tmesh groups
    // /// assert_eq!(mesh.tmesh_groups(), vec![Group::Value(1e12),    // bin 0.0  - 1e12
    // ///                                      Group::Value(1e16)]);  // bin 1e12 - 1e16
    // /// ```
    // pub fn tmesh_groups(&self) -> Vec<Group> {
    //     self.time_bins_upper()
    //         .iter()
    //         .map(|time| Group::Value(*time))
    //         .collect::<Vec<Group>>()
    // }
}

/// Indexing and conversion helpers
impl Mesh {
    /// Find the global voxel index from (e,t,i,j,k) indicies
    ///
    /// The voxel index corresponds to **the order seen in the column format
    /// output files**. All formats are coerced into this sorting for
    /// consistency.
    ///
    /// MCNP writes voxels as (k,j,i,t,e) loops, i.e:
    ///
    /// ```rust, ignore, no_run
    /// for e in energy_groups
    ///     for t in time_groups
    ///         for i in imesh
    ///             for j in jmesh
    ///                 for k in kmesh
    ///                     // ... write voxel
    /// ```
    ///
    /// Frustratingly, MCNP internally uses the "cell index" ordering which
    /// loops in the reverse for (i,j,k). See
    /// [cell_index_from_etijk()](Mesh.cell_index_from_etijk) for details.
    pub fn voxel_index_from_etijk(
        &self,
        e_idx: usize,
        t_idx: usize,
        i_idx: usize,
        j_idx: usize,
        k_idx: usize,
    ) -> usize {
        let mut idx: usize = e_idx * (self.n_tbins() * self.iints * self.jints * self.kints);
        idx += t_idx * (self.iints * self.jints * self.kints);
        idx += i_idx * (self.jints * self.kints);
        idx += j_idx * (self.kints);
        idx += k_idx;
        idx
    }

    /// Find the global cell index from (e,t,i,j,k) indicies
    ///
    /// The cell index corresponds to **the internal ordering used by MCNP**,
    /// not what is seen in output files. The cell index number loops energy and
    /// time groups as expected, but (i,j,k) indicies are calculated as:
    ///
    /// ```text
    ///     # FORTAN indexing
    ///     cell index number = i + (j − 1) x I + (k − 1) x I x J
    /// ```
    ///
    /// where `I` and `J` are the total number of `imesh` and `jmesh` bins.
    ///
    /// In other words, this loops as:
    ///
    /// ```rust, ignore, no_run
    /// for e in energy_groups
    ///     for t in time_groups
    ///         for k in kmesh
    ///             for j in jmesh
    ///                 for i in imesh
    ///                     // ...some voxel
    /// ```
    ///
    /// This is the order needed by VTK formats and weight window files.
    pub fn cell_index_from_etijk(
        &self,
        e_idx: usize,
        t_idx: usize,
        i_idx: usize,
        j_idx: usize,
        k_idx: usize,
    ) -> usize {
        let mut idx: usize = e_idx * self.n_tbins() * self.iints * self.jints * self.kints;
        idx += t_idx * (self.iints * self.jints * self.kints);
        idx += k_idx * (self.iints * self.jints);
        idx += j_idx * self.iints;
        idx += i_idx;
        idx
    }

    /// Find the (e,t,i,j,k) indicies for a given voxel index
    ///
    /// The reverse of [voxel_index_to_etijk()](Mesh.voxel_index_to_etijk).
    pub fn etijk_from_voxel_index(&self, idx: usize) -> (usize, usize, usize, usize, usize) {
        // convenient values for readability
        let a: usize = self.n_tbins() * self.kints * self.jints * self.iints;
        let b: usize = self.kints * self.jints * self.iints;
        let c: usize = self.kints * self.jints;
        let d: usize = self.kints;

        // find indicies in reverse (integer division floors in Rust)
        let e: usize = idx / a;
        let t: usize = (idx - e * a) / b;
        let i: usize = (idx - e * a - t * b) / c;
        let j: usize = (idx - e * a - t * b - i * c) / d;
        let k: usize = idx - e * a - t * b - i * c - j * d;

        (e, t, i, j, k)
    }

    /// Convert voxel index to a cell index
    ///
    /// Besides the obvious convenience, this is needed for UKAEA CuV formats.
    /// For some reason it was thought hilarious to order the
    /// `Number_of_material_cells_per_voxel` array by cell index while ordering
    /// the output column data by voxel index.
    pub fn cell_index_from_voxel_index(&self, idx: usize) -> usize {
        let (e, t, i, j, k) = self.etijk_from_voxel_index(idx);
        self.cell_index_from_etijk(e, t, i, j, k)
    }

    // todo: depnds on etijk_from_cell_index()
    /// Convert a cell index to a voxel index
    pub fn voxel_index_from_cell_index(&self, idx: usize) -> usize {
        let (e, t, i, j, k) = self.etijk_from_cell_index(idx);
        self.voxel_index_from_etijk(e, t, i, j, k)
    }

    // todo: figure out a clean way of doing this one
    /// Find the (e,t,i,j,k) indicies for a given cell index
    pub fn etijk_from_cell_index(&self, _idx: usize) -> (usize, usize, usize, usize, usize) {
        todo!()
    }

    /// For a given energy, find what group the results are under
    ///
    /// Error returned for handling if the energy is outside of the emesh bounds
    /// and should be handled by the caller.
    ///
    /// Important: the group an energy is under is determined by
    /// [find_bin_inclusive()](ntools_utils::SliceExt::find_bin_inclusive) rules
    /// for MCNP output, even though internally it follows the
    /// [find_bin_exclusive()](ntools_utils::SliceExt::find_bin_exclusive) rules.
    pub fn energy_group_from_value(&self, energy: f64) -> Result<Group> {
        if self.n_ebins() == 1 {
            Ok(Group::Total)
        } else {
            let idx = self.emesh.find_bin_inclusive(energy)?;
            Ok(Group::Value(self.emesh[idx]))
        }
    }

    /// Get the energy [Group](crate::mesh::Group) for the energy index `e_idx`
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Group};
    /// // Generate a simple test mesh
    /// let mesh = Mesh {
    ///     eints: 2,
    ///     emesh: vec![0.0, 1.0, 20.0],
    ///     ..Default::default()
    /// };
    /// // Find the group for an energy bin index
    /// assert_eq!(mesh.energy_group_from_index(0).unwrap(), Group::Value(1.0));
    /// assert_eq!(mesh.energy_group_from_index(1).unwrap(), Group::Value(20.0));
    /// assert_eq!(mesh.energy_group_from_index(2).unwrap(), Group::Total);
    /// assert!(mesh.energy_group_from_index(3).is_err());
    /// ```
    pub fn energy_group_from_index(&self, e_idx: usize) -> Result<Group> {
        if e_idx < self.n_ebins() {
            Ok(self.energy_groups()[e_idx])
        } else {
            Err(Error::IndexOutOfBounds {
                minimum: 0,
                maximum: self.n_ebins(),
                actual: e_idx,
            })
        }
    }

    /// For a given energy group, find the corresponding `emesh` bin index
    ///
    /// Important: the group an energy is under is determined by
    /// [find_bin_inclusive()](ntools_utils::SliceExt::find_bin_inclusive) rules
    /// for MCNP output, even though internally it follows the
    /// [find_bin_exclusive()](ntools_utils::SliceExt::find_bin_exclusive) rules.
    pub fn energy_index_from_group(&self, energy_group: Group) -> Result<usize> {
        match energy_group {
            Group::Total => Ok(self.n_ebins() - 1),
            Group::Value(energy) => Ok(self.emesh.find_bin_inclusive(energy)?),
        }
    }

    /// For a given energy, find the corresponding `emesh` bin index
    pub fn energy_index_from_value(&self, energy: f64) -> Result<usize> {
        Ok(self.emesh.find_bin_inclusive(energy)?)
    }

    /// For a given time, find what group the results are under
    ///
    /// Error returned for handling if the time is outside of the emesh bounds
    /// and should be handled by the caller.
    ///
    /// Important: the group a time is under is determined by
    /// [find_bin_inclusive()](ntools_utils::SliceExt::find_bin_inclusive) rules
    /// for MCNP output, even though internally it follows the
    /// [find_bin_exclusive()](ntools_utils::SliceExt::find_bin_exclusive) rules.
    pub fn time_group_from_value(&self, time: f64) -> Result<Group> {
        // tmesh can be empty, have just total, or values + total
        if self.n_tbins() == 1 {
            Ok(Group::Total)
        } else {
            let idx = self.tmesh.find_bin_inclusive(time)?;
            Ok(Group::Value(self.tmesh[idx]))
        }
    }

    /// Get the time [Group](crate::mesh::Group) for the time index `t_idx`
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::{Mesh, Group};
    /// // Generate a simple test mesh
    /// let mesh = Mesh {
    ///     tints: 2,
    ///     tmesh: vec![0.0, 1e16, 1e30],
    ///     ..Default::default()
    /// };
    /// // Find the group for an energy bin index
    /// assert_eq!(mesh.time_group_from_index(0).unwrap(), Group::Value(1e16));
    /// assert_eq!(mesh.time_group_from_index(1).unwrap(), Group::Value(1e30));
    /// assert_eq!(mesh.time_group_from_index(2).unwrap(), Group::Total);
    /// assert!(mesh.time_group_from_index(3).is_err());
    /// ```
    pub fn time_group_from_index(&self, t_idx: usize) -> Result<Group> {
        if t_idx < self.n_tbins() {
            Ok(self.time_groups()[t_idx])
        } else {
            Err(Error::IndexOutOfBounds {
                minimum: 0,
                maximum: self.n_tbins(),
                actual: t_idx,
            })
        }
    }

    /// For a given time group, find the corresponding `tmesh` bin index
    ///
    /// Important: the group a time is under is determined by
    /// [find_bin_inclusive()](ntools_utils::SliceExt::find_bin_inclusive) rules
    /// for MCNP output, even though internally it follows the
    /// [find_bin_exclusive()](ntools_utils::SliceExt::find_bin_exclusive) rules.
    pub fn time_index_from_group(&self, time_group: Group) -> Result<usize> {
        match time_group {
            Group::Total => Ok(self.n_tbins() - 1),
            Group::Value(time) => Ok(self.tmesh.find_bin_inclusive(time)?),
        }
    }

    /// For a given time, find the corresponding `tmesh` bin index
    pub fn time_index_from_value(&self, time: f64) -> Result<usize> {
        Ok(self.tmesh.find_bin_inclusive(time)?)
    }
}

// Private point methods
impl Mesh {
    /// Checks if [Point] coordinate and groups are all within the mesh bounds
    ///
    /// Points exactly on the boundaries are considered within the self. It is
    /// assumed that the point has the correct coordinate system.
    fn is_point_valid(&self, point: &Point) -> Result<bool> {
        if self.is_valid_group(point)? && self.is_valid_coordinate(point)? {
            Ok(true)
        } else {
            Err(Error::PointNotFound {
                point: point.clone(),
            })
        }
    }

    /// Checks if (i,j,k) coordinate is within the mesh bounds
    ///
    /// Points exactly on the boundaries are considered within the self. It is
    /// assumed that the point has the correct coordinate system.
    fn is_valid_coordinate(&self, point: &Point) -> Result<bool> {
        // todo should be in the correct coordinate system, but can add a check
        Ok(match point.kind {
            PointKind::Index => {
                // for ijk only need to check the max
                (point.i as usize) < self.iints
                    && (point.j as usize) < self.jints
                    && (point.k as usize) < self.kints
            }
            _ => {
                // for coordinate types need to check all the mesh boundaries
                point.i >= self.imesh.try_min()?
                    && point.i <= self.imesh.try_max()?
                    && point.j >= self.jmesh.try_min()?
                    && point.j <= self.jmesh.try_max()?
                    && point.k >= self.kmesh.try_min()?
                    && point.k <= self.kmesh.try_max()?
            }
        })
    }

    fn is_valid_group(&self, point: &Point) -> Result<bool> {
        // Make sure the energy group is valid
        if let Group::Value(e) = point.e {
            if e < self.emesh.try_min()? || e > self.emesh.try_max()? {
                return Ok(false);
            }
        }

        // Make sure the energy group is valid
        if let Group::Value(t) = point.t {
            if t < self.tmesh.try_min()? || t > self.tmesh.try_max()? {
                return Ok(false);
            }
        }

        // should be good otherwise
        Ok(true)
    }

    /// Convert tuple of (r,z,t) to cartesian (x,y,z)
    fn convert_rzt_to_xyz(&self, r: f64, z: f64, t: f64) -> (f64, f64, f64) {
        (r * t.cos(), r * t.sin(), z)
    }

    /// Convert tuple of (x,y,z) to cylindrical (r,z,t)
    fn convert_xyz_to_rzt(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        // invert the translation
        let mut x = x - self.origin[0];
        let mut y = y - self.origin[1];
        let mut z = z - self.origin[2];

        // invert the rotation
        if let Some(r) = self.rotation_matrix() {
            let a = r.inverse_transform_vector(&Vector3::from([x, y, z]));
            x = a[0];
            y = a[1];
            z = a[2];
        };

        // now just point as if in default rotation/origin/axis

        // convert to 0-360 range, TAU = 2*PI
        let mut t = y.atan2(x);
        t = if t.is_sign_negative() {
            std::f64::consts::TAU + t
        } else {
            t
        };
        (x.hypot(y), z, t)
    }

    /// Initialise the rotation matrix from AXS if required
    fn rotation_matrix(&self) -> Option<Rotation<f64, 3>> {
        // the mcnp default axis
        let axs_default = [0.0, 0.0, 1.0];

        if axs_default == self.axs {
            None
        } else {
            let axs_default = Vector3::from(axs_default);
            let axs_user = Vector3::from([self.axs[0], self.axs[1], self.axs[2]]);
            Some(Rotation::face_towards(&axs_user, &axs_default))
        }
    }

    fn coerce_point_kind(&self, point: &Point) -> Point {
        match point.kind {
            PointKind::Index => point.clone(),
            PointKind::Rectangular => match self.geometry {
                Geometry::Rectangular => point.clone(),
                Geometry::Cylindrical => {
                    warn!("Automatic Point conversion to mesh geometry may not be exact");
                    let (r, z, t) = self.convert_xyz_to_rzt(point.i, point.j, point.k);
                    Point {
                        e: point.e,
                        t: point.t,
                        i: r,
                        j: z,
                        k: t,
                        kind: PointKind::Cylindrical,
                    }
                }
            },
            PointKind::Cylindrical => match self.geometry {
                Geometry::Cylindrical => point.clone(),
                Geometry::Rectangular => {
                    warn!("Automatic Point conversion to mesh geometry may not be exact");
                    let (x, y, z) = self.convert_rzt_to_xyz(point.i, point.j, point.k);
                    Point {
                        e: point.e,
                        t: point.t,
                        i: x,
                        j: y,
                        k: z,
                        kind: PointKind::Rectangular,
                    }
                }
            },
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            id: 0,
            geometry: Geometry::Rectangular,
            particle: Particle::Unknown,
            imesh: Vec::new(),
            iints: 0,
            jmesh: Vec::new(),
            jints: 0,
            kmesh: Vec::new(),
            kints: 0,
            emesh: Vec::new(),
            eints: 0,
            tmesh: Vec::new(),
            tints: 0,
            origin: [0.0, 0.0, 0.0],
            axs: [0.0, 0.0, 1.0],
            vec: [1.0, 0.0, 0.0],
            voxels: Vec::new(),
            format: Format::NONE,
        }
    }
}

impl std::fmt::Display for Mesh {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let head: String = f!(
            " > Mesh {} [{:?}, {:?}]",
            self.id,
            self.particle,
            self.geometry
        );

        let mut s = f!("{}\n{}\n{}\n", "-".repeat(40), head, "-".repeat(40));

        s += &f!("origin: {:?}\n", self.origin);
        s += &f!("axs   : {:?}\n", self.axs);
        s += &f!("vec   : {:?}\n", self.vec);

        s += &f!(
            "imesh : {:>10} - {:>8} cm ({} bins)\n",
            self.imesh[0].sci(2, 2),
            self.imesh.last().unwrap().sci(2, 2),
            self.iints
        );
        s += &f!(
            "jmesh : {:>10} - {:>8} cm ({} bins)\n",
            self.jmesh[0].sci(2, 2),
            self.jmesh.last().unwrap().sci(2, 2),
            self.jints
        );
        s += &f!(
            "kmesh : {:>10} - {:>8} cm ({} bins)\n",
            self.kmesh[0].sci(2, 2),
            self.kmesh.last().unwrap().sci(2, 2),
            self.kints
        );
        s += &f!(
            "emesh : {:>10} - {:>8} MeV ({} bins)\n",
            self.emesh[0].sci(2, 2),
            self.emesh.last().unwrap().sci(2, 2),
            self.n_ebins()
        );
        if self.tints > 1 {
            s += &f!(
                "tmesh : {:>10} - {:>8} shakes ({} bins)\n",
                self.tmesh[0].sci(2, 2),
                self.tmesh.last().unwrap().sci(2, 2),
                self.n_tbins()
            );
        }
        write!(f, "{}", s)
    }
}
