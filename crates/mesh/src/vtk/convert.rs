// standard library
use std::ops::RangeInclusive;

// ntools modules
use crate::{Geometry, Group, Mesh};
use ntools_utils::f;

// internal modules
use crate::vtk::MeshToVtkBuilder;
use crate::vtk::Vertex;

// extrenal crates
use log::warn;
use nalgebra::{Rotation, Vector3};
use vtkio::model::{
    Attribute, Attributes, ByteOrder, CellType, Cells, Coordinates, DataArray, DataSet,
    ElementType, Extent, IOBuffer, RangeExtent, RectilinearGridPiece, UnstructuredGridPiece,
    Version, VertexNumbers, Vtk,
};
use vtkio::xml::Compressor;

/// Convert mesh tallies to vtk formats for plotting
///
/// All of the of logic for converting voxel data into the right VTK types and
/// formats is implemented here. This includes calculating verticies for
/// cylindrical cases as an unstructured mesh.
///
/// The fields remain public for direct use, but for convenience and style
/// preference a builder pattern is also implemented and recommended.
///
/// # General properties
///
/// ## Error meshes
///
/// Error meshes omitted by default to save space.
///
/// If enabled, every mesh will have a corresponding relative uncertainty
/// dataset. Of course, this ~doubles file size, but that is fine in most cases.
///
/// ```rust
/// # use ntools_mesh::vtk::{MeshToVtk};
/// // Include error meshes for each result
/// let converter = MeshToVtk::builder()
///     .include_errors(true)
///     .build();
/// ```
///
/// ## Target specific voxel groups
///
/// **Important: By default all energy groups are included in the vtk**.
///
/// Specific energy groups can be provided to reduce file sizes. This is also
/// especially useful if only certain groups are of interest.
///
/// For specific energy/time groups:
///
/// ```rust
/// # use ntools_mesh::vtk::{MeshToVtk};
/// # use ntools_mesh::{Group};
/// // Choose specific energy and time groups by group index
/// let converter = MeshToVtk::builder()
///     .energy_groups(vec![0, 1, 2, 6])
///     .build();
/// ```
///
/// If the groups indicies are unknown the index may be found with:
///  
/// ```rust, no_run
/// # use ntools_mesh::{Group, Mesh};
/// # let mesh = Mesh::default();
/// // Find the group index of a 20 MeV particle and index of the "total" group
/// let e_idx = vec![
///     mesh.find_energy_group_index(Group::Value(20.0)).unwrap(),
///     mesh.find_energy_group_index(Group::Total).unwrap()
/// ];
/// ```
///
/// ## Vtk formatting
///
/// Included are a couple of more advanced options for VTK preferences.
///
/// Most useful is the byte ordering, which is important for binary file
/// compatability with plotting software. ParaView does not care, but something
/// like Visit only likes big endian. This is the default for convenience but is
/// completely up to the user.
///
/// ```rust
/// # use ntools_mesh::vtk::{MeshToVtk};
/// # use vtkio::model::ByteOrder;
/// // Change the byte ordering to little endian
/// let converter = MeshToVtk::builder()
///     .byte_order(ByteOrder::LittleEndian)
///     .build();
/// ```
///
/// Perhaps less useful is the compression method for XML file formats, but it
/// is included for completeness anyway.
///
/// ```rust
/// # use ntools_mesh::vtk::{MeshToVtk};
/// # use vtkio::xml::Compressor;
/// // Select the LZMA compression method
/// let converter = MeshToVtk::builder()
///     .compressor(Compressor::LZMA)
///     .build();
/// ```
///
/// Generally just use LZMA but other options are available:
/// - lzma (default)
/// - lz4
/// - zlib
/// - none
///
/// # A note on Cylindrical meshes
///
/// There is no VTK representation of cylindrical meshes, so an unstructured
/// mesh is generated from verticies based on the RZT bounds.
///
/// Unfortunately, this can result in "low-resolution" plots for meshes with
/// few theta bins. The number of theta bins can be increased to round off these
/// edges. This simply subdivides the voxels by an integer number of theta bins.
///
/// ![Cylindrical mesh resolution option](https://github.com/repositony/meshtal/blob/main/data/assets/cylindrical_mesh_resolution.png)
///
/// For example:
///
/// ```rust
/// # use ntools_mesh::vtk::{MeshToVtk};
/// // Split every theta bin into 3 to round off the edges
/// let converter = MeshToVtk::builder()
///     .resolution(3)
///     .build();
/// ```
///
/// Setting the `resolution` to 3 will subbdivide the theta bins into 3, thereby
/// tripling the number of edges plotted from 8 to 24 for a more rounded look.
///
/// Note that this can increase memory usage and file size significantly but is
/// a nice feature for generating more accurate cylinders.  
///
#[derive(Debug, PartialEq)]
pub struct MeshToVtk {
    /// Target energy group(s)
    pub energy_groups: Vec<usize>,
    /// Target energy group(s)
    pub time_groups: Vec<usize>,
    /// Include errors mesh in output files
    pub include_errors: bool,
    /// Byte ordering as big or little endian
    pub byte_order: ByteOrder,
    /// compression method for xml file formats
    pub compressor: Compressor,
    /// Cylindrical mesh resolution
    pub resolution: u8,
}

// Public API
impl MeshToVtk {
    /// Start with the default configuration
    pub fn new() -> MeshToVtk {
        Default::default()
    }

    /// Get an instance of the [MeshToVtkBuilder]
    pub fn builder() -> MeshToVtkBuilder {
        MeshToVtkBuilder::default()
    }

    /// Convert a [Mesh] to vtkio::Vtk object
    ///
    /// Once the configuration is set through either the builder or changing the
    /// fields directly, convert any [Mesh] into a Vtk ready for writing or
    /// futher processing.
    pub fn convert(&self, mesh: &Mesh) -> Vtk {
        match mesh.geometry {
            Geometry::Rectangular => self.rectangular_vtk(mesh),
            Geometry::Cylindrical => self.cylindrical_vtk(mesh),
        }
    }
}

impl Default for MeshToVtk {
    fn default() -> Self {
        MeshToVtkBuilder::default().build()
    }
}

/// Common use implementations
impl MeshToVtk {
    /// Collect energy groups, and if none are given fallback to using all groups
    fn collect_energy_group_idx(&self, mesh: &Mesh) -> Vec<usize> {
        // none defined? convert everything
        if self.energy_groups.is_empty() {
            return (0..mesh.ebins()).collect::<Vec<usize>>();
        }

        // filter out anything not valid, usize means < 0 inherently checked
        let mut indicies = self
            .energy_groups
            .iter()
            .copied()
            .filter(|e_idx| e_idx < &mesh.ebins())
            .collect::<Vec<usize>>();

        // clean up the list or just default to all if none of the indicies were
        // valid
        if !indicies.is_empty() {
            indicies.sort();
            indicies.dedup();
            indicies
        } else {
            warn!("Warning: No valid energy index provided, defaulting to all");
            (0..mesh.ebins()).collect::<Vec<usize>>()
        }
    }

    /// Collect time groups, and if none are given fallback to using all groups
    fn collect_time_group_idx(&self, mesh: &Mesh) -> Vec<usize> {
        // none defined? convert everything
        if self.time_groups.is_empty() {
            return (0..mesh.tbins()).collect::<Vec<usize>>();
        }

        // filter out anything not valid, usize means < 0 inherently checked
        let mut indicies = self
            .time_groups
            .iter()
            .copied()
            .filter(|t_idx| t_idx < &mesh.tbins())
            .collect::<Vec<usize>>();

        // clean up the list or just default to all if none of the indicies were
        // valid
        if !indicies.is_empty() {
            indicies.sort();
            indicies.dedup();
            indicies
        } else {
            warn!("Warning: No valid time index provided, defaulting to all");
            (0..mesh.tbins()).collect::<Vec<usize>>()
        }
    }

    /// Create a name to display in the output mesh data
    fn group_name(&self, mesh: &Mesh, e_idx: usize, t_idx: usize) -> String {
        // "Energy[0] 2.00E+01 MeV, Time[0] 1.00E+12 shakes, error"
        // ok to use indexing as already checked by this point

        let energy_prefix = match mesh.energy_groups()[e_idx] {
            Group::Value(e) => f!("Energy[{e_idx}] {e:.2E} MeV"),
            Group::Total => f!("Energy[{e_idx}] Total"),
        };

        let time_prefix = match mesh.time_groups()[t_idx] {
            Group::Value(t) => f!(", Time[{t_idx}] {t:.2E} shakes"),
            Group::Total => {
                if mesh.tbins() > 1 {
                    f!(", Time[{t_idx}] Total")
                } else {
                    "".to_string()
                }
            }
        };

        energy_prefix + &time_prefix
    }

    /// Create a Visit-friendly name to display in the output mesh data
    ///
    /// The rules are no whitespace, no brackets, basically nothing nice for
    /// formatting.
    fn group_name_visit(&self, mesh: &Mesh, e_idx: usize, t_idx: usize) -> String {
        let energy_prefix = f!("Energy-{e_idx}");

        let time_prefix = if mesh.tbins() > 1 {
            f!("_Time-{t_idx}")
        } else {
            "".to_string()
        };

        energy_prefix + &time_prefix
    }
}

/// Implementations for proecessing Rectangular mesh types
impl MeshToVtk {
    /// Convert mesh voxel data to vtkio types for writing
    fn rectangular_vtk(&self, mesh: &Mesh) -> Vtk {
        Vtk {
            version: Version::Auto,
            title: f!("Fmesh{} results", mesh.id),
            byte_order: self.byte_order,
            file_path: None,
            data: DataSet::inline(RectilinearGridPiece {
                extent: Self::extent(mesh),
                coords: Self::coordinates(mesh),
                data: self.collect_attributes(mesh),
            }),
        }
    }

    /// Defines number of mesh voxels in each extent for the rectilinear grid
    fn extent(mesh: &Mesh) -> Extent {
        let range_ext: RangeExtent = [
            RangeInclusive::new(0, mesh.iints as i32),
            RangeInclusive::new(0, mesh.jints as i32),
            RangeInclusive::new(0, mesh.kints as i32),
        ];
        Extent::Ranges(range_ext)
    }

    /// Defines coordiantes for rectilinear grid from mesh bounds
    fn coordinates(mesh: &Mesh) -> Coordinates {
        Coordinates {
            x: IOBuffer::F64(mesh.imesh.clone()),
            y: IOBuffer::F64(mesh.jmesh.clone()),
            z: IOBuffer::F64(mesh.kmesh.clone()),
        }
    }

    /// Collect rectilinear cell results into appropriate order/format
    fn collect_attributes(&self, mesh: &Mesh) -> Attributes {
        let mut attributes: Attributes = Attributes::new();

        let energy_groups = self.collect_energy_group_idx(mesh);
        let time_groups = self.collect_time_group_idx(mesh);

        for e_idx in &energy_groups {
            for t_idx in &time_groups {
                let voxels = mesh.slice_voxels_by_idx(*e_idx, *t_idx).unwrap();

                let (results, errors): (Vec<f64>, Vec<f64>) = voxels
                    .iter()
                    .map(|v| (v.result, v.error))
                    .collect::<Vec<(f64, f64)>>()
                    .into_iter()
                    .unzip();

                let cell_data = DataArray {
                    name: self.group_name(mesh, *e_idx, *t_idx),
                    elem: ElementType::Scalars {
                        num_comp: 1,
                        lookup_table: None,
                    },
                    data: IOBuffer::F64(Self::sort_by_cell_index(mesh, results)),
                };
                attributes.cell.push(Attribute::DataArray(cell_data));

                // do the same for the errors if they are to be included
                if self.include_errors {
                    let cell_data = DataArray {
                        name: self.group_name(mesh, *e_idx, *t_idx) + ", error",
                        elem: ElementType::Scalars {
                            num_comp: 1,
                            lookup_table: None,
                        },
                        data: IOBuffer::F64(Self::sort_by_cell_index(mesh, errors)),
                    };
                    attributes.cell.push(Attribute::DataArray(cell_data));
                }
            }
        }

        attributes
    }

    /// Sort a list of results for the rectilinear grid cell ordering
    fn sort_by_cell_index(mesh: &Mesh, values: Vec<f64>) -> Vec<f64> {
        let idx = (0..values.len())
            .map(|i| mesh.voxel_index_to_cell_index(i))
            .collect::<Vec<usize>>();

        let mut result = idx.iter().zip(values.iter()).collect::<Vec<_>>();

        result.sort_by(|a, b| a.0.cmp(b.0));
        result.into_iter().map(|r| *r.1).collect()
    }
}

/// Implementations for proecessing Cylindrical mesh types
impl MeshToVtk {
    /// Convert mesh voxel data to vtkio types for writing
    fn cylindrical_vtk(&self, mesh: &Mesh) -> Vtk {
        // generate cell verticies from mesh bounds
        let (points, offset, cell_types) = self.cell_verticies(mesh);
        let connect = (0..*offset.last().unwrap()).collect::<Vec<u64>>();

        Vtk {
            version: Version::Auto,
            title: f!("Fmesh{} results", mesh.id),
            byte_order: self.byte_order,
            file_path: None,
            data: DataSet::inline(UnstructuredGridPiece {
                points: points.into(),
                cells: Cells {
                    cell_verts: VertexNumbers::XML {
                        connectivity: connect,
                        offsets: offset,
                    },
                    types: cell_types,
                },
                data: self.collect_cyl_attributes(mesh),
            }),
        }
    }

    /// Cylinders need to be built explicitly from vertex points
    fn cell_verticies(&self, mesh: &Mesh) -> (Vec<f64>, Vec<u64>, Vec<CellType>) {
        let mut points: Vec<f64> = Vec::new();
        let mut offsets: Vec<u64> = Vec::new();
        let mut cell_types: Vec<CellType> = Vec::new();
        let rotation_axs = Self::init_rotation(&mesh.axs);
        let rotation_vec = mesh.vec[1].atan2(mesh.vec[0]);

        // go layer-by-layer up from z
        for layer in 0..mesh.jints {
            // first inner segments always CellType::Wedge
            self.wedge_segments(
                mesh,
                layer,
                &mut points,
                &mut offsets,
                &mut cell_types,
                &rotation_axs,
                rotation_vec,
            );

            // any additional ring segments use CellType::Voxel
            if mesh.iints > 1 {
                // start from 1, the first ring is already made from wedges
                for ring in 1..mesh.iints {
                    self.voxel_segments(
                        mesh,
                        ring,
                        layer,
                        &mut points,
                        &mut offsets,
                        &mut cell_types,
                        &rotation_axs,
                        rotation_vec,
                    );
                }
            }
        }

        (points, offsets, cell_types)
    }

    #[allow(clippy::too_many_arguments)]
    /// For the central voxels where r=0
    fn wedge_segments(
        &self,
        mesh: &Mesh,
        z_idx: usize,
        points: &mut Vec<f64>,
        offsets: &mut Vec<u64>,
        cell_types: &mut Vec<CellType>,
        rotation_axs: &Option<Rotation<f64, 3>>,
        rotation_vec: f64,
    ) {
        let mut step = 2.0 * std::f64::consts::PI / (mesh.kints as f64);
        step /= self.get_resolution(&mesh.kints) as f64;
        let r = mesh.imesh[1];

        // wedge type has 6 verticies
        // only need to find three and then repeat for the lower layer
        for i in 0..(mesh.kints * self.get_resolution(&mesh.kints) as usize) {
            let t0 = step * (i as f64) + rotation_vec;
            let t1 = step * (i as f64 + 1.0) + rotation_vec;

            let x0 = r * t0.cos();
            let y0 = r * t0.sin();

            let x1 = r * t1.cos();
            let y1 = r * t1.sin();

            for idx in z_idx..=(z_idx + 1) {
                let z = mesh.jmesh[idx];
                points.extend(
                    Vertex { x: 0.0, y: 0.0, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x0, y: y0, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x1, y: y1, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
            }

            Self::update_offsets(offsets, 6);
            cell_types.push(CellType::Wedge);
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// For anything beyond the first inside ring
    fn voxel_segments(
        &self,
        mesh: &Mesh,
        r_idx: usize,
        z_idx: usize,
        points: &mut Vec<f64>,
        offsets: &mut Vec<u64>,
        cell_types: &mut Vec<CellType>,
        rotation_axs: &Option<Rotation<f64, 3>>,
        rotation_vec: f64,
    ) {
        let mut step = 2.0 * std::f64::consts::PI / (mesh.kints as f64);
        step /= self.get_resolution(&mesh.kints) as f64;
        let r0 = mesh.imesh[r_idx];
        let r1 = mesh.imesh[r_idx + 1];

        // voxel type has 8 verticies
        // only need to find 4 and then repeat at lower layer
        for i in 0..(mesh.kints * self.get_resolution(&mesh.kints) as usize) {
            let t0 = step * (i as f64) + rotation_vec;
            let t1 = step * (i as f64 + 1.0) + rotation_vec;

            let x00: f64 = r0 * t0.cos();
            let y00: f64 = r0 * t0.sin();

            let x01: f64 = r0 * t1.cos();
            let y01: f64 = r0 * t1.sin();

            let x10: f64 = r1 * t0.cos();
            let y10: f64 = r1 * t0.sin();

            let x11: f64 = r1 * t1.cos();
            let y11: f64 = r1 * t1.sin();

            for idx in z_idx..=(z_idx + 1) {
                let z = mesh.jmesh[idx];
                points.extend(
                    Vertex { x: x00, y: y00, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x01, y: y01, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x10, y: y10, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x11, y: y11, z }
                        .rotate(rotation_axs)
                        .translate(&mesh.origin)
                        .as_array(),
                );
            }

            Self::update_offsets(offsets, 8);
            cell_types.push(CellType::Voxel);
        }
    }

    /// Bring all of the cell data together
    fn collect_cyl_attributes(&self, mesh: &Mesh) -> Attributes {
        let mut attributes: Attributes = Attributes::new();
        let energy_groups = self.collect_energy_group_idx(mesh);
        let time_groups = self.collect_time_group_idx(mesh);
        let cyl_cell_order = self.cylinder_cell_order(mesh);

        for e_idx in &energy_groups {
            for t_idx in &time_groups {
                let voxels = mesh.slice_voxels_by_idx(*e_idx, *t_idx).unwrap();

                let (mut results, mut errors): (Vec<f64>, Vec<f64>) = cyl_cell_order
                    .iter()
                    .map(|i| (voxels[*i].result, voxels[*i].error))
                    .collect::<Vec<(f64, f64)>>()
                    .into_iter()
                    .unzip();

                results = Self::repeat_values(results, self.get_resolution(&mesh.kints));

                errors = Self::repeat_values(errors, self.get_resolution(&mesh.kints));

                let cell_data = DataArray {
                    name: self.group_name_visit(mesh, *e_idx, *t_idx),
                    elem: ElementType::Scalars {
                        num_comp: 1,
                        lookup_table: None,
                    },
                    data: IOBuffer::F64(results),
                };
                attributes.cell.push(Attribute::DataArray(cell_data));

                // do the same for the errors if they are to be included
                if self.include_errors {
                    let cell_data = DataArray {
                        name: self.group_name_visit(mesh, *e_idx, *t_idx) + "_error",
                        elem: ElementType::Scalars {
                            num_comp: 1,
                            lookup_table: None,
                        },
                        data: IOBuffer::F64(errors),
                    };
                    attributes.cell.push(Attribute::DataArray(cell_data));
                }
            }
        }

        attributes
    }

    /// Repeat whatever set of values is in a vector
    fn repeat_values(values: Vec<f64>, repeat: u8) -> Vec<f64> {
        values
            .into_iter()
            .flat_map(|n| std::iter::repeat(n).take(repeat.into()))
            .collect()
    }

    /// Used in calculation of verticies
    fn update_offsets(offsets: &mut Vec<u64>, size: usize) {
        let offset = match offsets.is_empty() {
            true => size.try_into().unwrap(),
            false => (size + (*offsets.last().unwrap() as usize))
                .try_into()
                .unwrap(),
        };
        offsets.push(offset);
    }

    /// Fix the resolution issue in the background for 1-2 theta bins
    ///
    /// For performance the converter would have to be mutable, or the user
    /// would have to know to set the resolution for a couple of special cases.
    /// This is just easier for everyone.
    fn get_resolution(&self, n_bins: &usize) -> u8 {
        match n_bins {
            // only one theta bin, minimim verticies needed will be 3
            1 => self.resolution.max(3),
            // only two theta bins, minimim verticies needed will be 4
            2 => self.resolution.max(2),
            // anything else is fine
            _ => self.resolution,
        }
    }

    /// Initialise the rotation matrix from AXS if required
    fn init_rotation(axis: &[f64]) -> Option<Rotation<f64, 3>> {
        let axs_default = [0.0, 0.0, 1.0];

        if axs_default == *axis {
            None
        } else {
            let axs_default = Vector3::from(axs_default);
            let axs_user = Vector3::from([axis[0], axis[1], axis[2]]);
            Some(Rotation::face_towards(&axs_user, &axs_default))
        }
    }

    /// Get the correct ordering for matching voxels to cylinder vtk cells
    fn cylinder_cell_order(&self, mesh: &Mesh) -> Vec<usize> {
        let mut index: Vec<(usize, usize)> = (0..mesh.n_voxels_per_group())
            .map(|idx| {
                let (_, _, i, j, k) = mesh.voxel_index_to_etijk(idx);
                let key = k + (i * mesh.kints) + (j * mesh.iints * mesh.kints);
                (idx, key)
            })
            .collect();

        index.sort_by_key(|&(_, key)| key);
        index.into_iter().map(|(i, _)| i).collect()
    }
}
