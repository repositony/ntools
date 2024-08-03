// standard library
use std::ops::RangeInclusive;

// ntools modules
use ntools_utils::f;

// internal modules
use crate::vtk::builder::WeightsToVtkBuilder;
use crate::vtk::Vertex;
use crate::WeightWindow;

// extrenal crates
use nalgebra::{Rotation, Vector3};
use vtkio::model::{
    Attribute, Attributes, ByteOrder, CellType, Cells, Coordinates, DataArray, DataSet,
    ElementType, Extent, IOBuffer, RangeExtent, RectilinearGridPiece, UnstructuredGridPiece,
    Version, VertexNumbers, Vtk,
};
use vtkio::xml::Compressor;

/// Convert weight window sets to vtk formats for plotting
///
/// All of the of logic for converting weight window sets into the right VTK
/// types and formats is implemented here. This includes calculating verticies
/// for cylindrical cases as an unstructured mesh.
///
/// The fields remain public for direct use, but for convenience and style
/// preference a builder pattern is also implemented and recommended.
///
/// # Formatting
///
/// Included are a couple of more advanced options for VTK preferences.
///
/// Most useful is the byte ordering, which is important for binary file
/// compatability with plotting software. ParaView does not care, but something
/// like Visit only likes big endian. This is the default for convenience but is
/// completely up to the user.
///
/// ```rust
/// # use ntools_weights::vtk::WeightsToVtk;
/// # use vtkio::model::ByteOrder;
/// let converter = WeightsToVtk::builder()
///     .byte_order(ByteOrder::LittleEndian)
///     .build();
/// ```
///
/// Perhaps less useful is the compression method for XML file formats, but it
/// is included for completeness anyway.
///
/// ```rust
/// # use ntools_weights::vtk::WeightsToVtk;
/// # use vtkio::xml::Compressor;
/// let converter = WeightsToVtk::builder()
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
/// # use ntools_weights::vtk::WeightsToVtk;
/// let converter = WeightsToVtk::builder()
///     .resolution(3)
///     .build();
/// ```
///
/// Setting the `resolution` to 3 will subbdivide the theta bins into 3, thereby
/// tripling the number of edges plotted from 8 to 24 for a more rounded look.
///
/// Note that this can increase memory usage and file size significantly but is
/// a nice feature for generating more accurate cylinders.  
#[derive(Debug, PartialEq)]
pub struct WeightsToVtk {
    /// Byte ordering as big or little endian
    pub byte_order: ByteOrder,
    /// compression method for xml file formats
    pub compressor: Compressor,
    /// Cylindrical mesh resolution
    pub resolution: u8,
}

// Public API
impl WeightsToVtk {
    /// Start with the default configuration
    pub fn new() -> WeightsToVtk {
        Default::default()
    }

    /// Get an instance of the [WeightsToVtkBuilder]
    pub fn builder() -> WeightsToVtkBuilder {
        WeightsToVtkBuilder::default()
    }

    /// Convert a [WeightWindow] to Vtk object
    ///
    /// Once the configuration is set through either the builder or changing the
    /// fields directly, convert any [WeightWindow] into a Vtk ready for writing
    /// or futher processing.
    pub fn convert(&self, weight_window: &WeightWindow) -> Vtk {
        match weight_window.nwg {
            1 => self.rectangular_vtk(weight_window),
            2 => self.cylindrical_vtk(weight_window),
            _ => panic!("Unknown geometry"),
        }
    }
}

impl Default for WeightsToVtk {
    fn default() -> Self {
        WeightsToVtkBuilder::default().build()
    }
}

/// Implementations for proecessing Rectangular mesh types
impl WeightsToVtk {
    /// Convert WeightWindow data to vtkio types for writing
    fn rectangular_vtk(&self, weight_window: &WeightWindow) -> Vtk {
        Vtk {
            version: Version::Auto,
            title: f!("{:?} weight window sets", weight_window.particle),
            byte_order: self.byte_order,
            file_path: None,
            data: DataSet::inline(RectilinearGridPiece {
                extent: Self::extent(weight_window),
                coords: Self::coordinates(weight_window),
                data: self.collect_attributes(weight_window),
            }),
        }
    }

    /// Defines number of mesh voxels in each extent for the rectilinear grid
    fn extent(ww: &WeightWindow) -> Extent {
        let range_ext: RangeExtent = [
            RangeInclusive::new(0, ww.nfx as i32),
            RangeInclusive::new(0, ww.nfy as i32),
            RangeInclusive::new(0, ww.nfz as i32),
        ];
        Extent::Ranges(range_ext)
    }

    /// Defines coordiantes for rectilinear grid from mesh bounds
    fn coordinates(ww: &WeightWindow) -> Coordinates {
        Coordinates {
            x: IOBuffer::F64(
                std::iter::once(ww.x0)
                    .chain(ww.qps_x.iter().map(|x| x[1]))
                    .collect(),
            ),
            y: IOBuffer::F64(
                std::iter::once(ww.y0)
                    .chain(ww.qps_y.iter().map(|y| y[1]))
                    .collect(),
            ),
            z: IOBuffer::F64(
                std::iter::once(ww.z0)
                    .chain(ww.qps_z.iter().map(|z| z[1]))
                    .collect(),
            ),
        }
    }

    /// Collect rectilinear cell results into appropriate order/format
    fn collect_attributes(&self, ww: &WeightWindow) -> Attributes {
        // make sure the data can be chunked
        if (ww.nfx * ww.nfy * ww.nfz) == 0 {
            panic!("At least one fine mesh dimension has length 0");
        }

        // already ordered correctly so can just iterate over each set
        let weight_sets = ww.weights.chunks(ww.nfx * ww.nfy * ww.nfz);

        let mut attributes: Attributes = Attributes::new();
        for (i, group) in weight_sets.enumerate() {
            let cell_data = DataArray {
                // todo: do something more clever here later
                name: f!("group_{i}"),
                elem: ElementType::Scalars {
                    num_comp: 1,
                    lookup_table: None,
                },
                data: IOBuffer::F64(group.to_vec()),
            };
            attributes.cell.push(Attribute::DataArray(cell_data));
        }

        attributes
    }
}

/// Implementations for proecessing Cylindrical mesh types
impl WeightsToVtk {
    /// Convert mesh voxel data to vtkio types for writing
    fn cylindrical_vtk(&self, ww: &WeightWindow) -> Vtk {
        // generate cell verticies from mesh bounds
        let (points, offset, cell_types) = self.cell_verticies(ww);
        let connect = (0..*offset.last().unwrap()).collect::<Vec<u64>>();

        Vtk {
            version: Version::Auto,
            title: f!("Particle {} weights", ww.particle),
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
                data: self.collect_cyl_attributes(ww),
            }),
        }
    }

    /// Cylinders need to be built explicitly from vertex points
    fn cell_verticies(&self, ww: &WeightWindow) -> (Vec<f64>, Vec<u64>, Vec<CellType>) {
        let mut points: Vec<f64> = Vec::new();
        let mut offsets: Vec<u64> = Vec::new();
        let mut cell_types: Vec<CellType> = Vec::new();
        let rotation_axs = Self::init_rotation(&[ww.x1, ww.y1, ww.z1]);
        let rotation_vec = ww.y2.atan2(ww.x2);

        // first inner segments always CellType::Wedge
        for layer in 0..ww.ncy {
            self.wedge_segments(
                ww,
                layer,
                &mut points,
                &mut offsets,
                &mut cell_types,
                &rotation_axs,
                rotation_vec,
            );
        }

        // any additional ring segments use CellType::Voxel
        if ww.ncx > 1 {
            // start from 1, the first ring is already made from CellType::Wedge
            for ring in 1..ww.ncx {
                for layer in 0..ww.ncy {
                    self.voxel_segments(
                        ww,
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
        ww: &WeightWindow,
        layer: usize,
        points: &mut Vec<f64>,
        offsets: &mut Vec<u64>,
        cell_types: &mut Vec<CellType>,
        rotation_axs: &Option<Rotation<f64, 3>>,
        rotation_vec: f64,
    ) {
        let mut step = 2.0 * std::f64::consts::PI / (ww.ncz as f64);
        step /= self.get_resolution(&ww.ncz) as f64;

        // move this shit out of here
        let r = ww.qps_x[0][1]; // outer radius, the inner is always 0

        // wedge type has 6 verticies
        // only need to find three and then repeat for the lower layer
        for i in 0..(ww.ncz * self.get_resolution(&ww.ncz) as usize) {
            let t0 = step * (i as f64) + rotation_vec;
            let t1 = step * (i as f64 + 1.0) + rotation_vec;

            let x0 = r * t0.cos();
            let y0 = r * t0.sin();

            let x1 = r * t1.cos();
            let y1 = r * t1.sin();

            for idx in layer..=(layer + 1) {
                let z = if idx == 0 { 0.0 } else { ww.qps_y[idx - 1][1] };

                points.extend(
                    Vertex { x: 0.0, y: 0.0, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x0, y: y0, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x1, y: y1, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
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
        ww: &WeightWindow,
        ring: usize,
        layer: usize,
        points: &mut Vec<f64>,
        offsets: &mut Vec<u64>,
        cell_types: &mut Vec<CellType>,
        rotation_axs: &Option<Rotation<f64, 3>>,
        rotation_vec: f64,
    ) {
        let mut step = 2.0 * std::f64::consts::PI / (ww.ncz as f64);
        step /= self.get_resolution(&ww.ncz) as f64;

        let r0 = ww.qps_x[ring - 1][1]; // inner radius
        let r1 = ww.qps_x[ring][1]; // outer radius

        // voxel type has 8 verticies
        // only need to find 4 and then repeat at lower layer
        for i in 0..(ww.ncz * self.get_resolution(&ww.ncz) as usize) {
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

            for idx in layer..=(layer + 1) {
                let z = if idx == 0 { 0.0 } else { ww.qps_y[idx - 1][1] };

                points.extend(
                    Vertex { x: x00, y: y00, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x01, y: y01, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x10, y: y10, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
                points.extend(
                    Vertex { x: x11, y: y11, z }
                        .rotate(rotation_axs)
                        .translate(&[ww.x0, ww.y0, ww.z0])
                        .as_array(),
                );
            }

            Self::update_offsets(offsets, 8);
            cell_types.push(CellType::Voxel);
        }
    }

    /// Bring all of the cell data together
    fn collect_cyl_attributes(&self, ww: &WeightWindow) -> Attributes {
        // make sure the data can be chunked
        if (ww.nfx * ww.nfy * ww.nfz) == 0 {
            panic!("At least one fine mesh dimension has length 0");
        }

        // The vtk voxels have been built to the i-j-k indexing order
        let cell_order = Self::get_order(ww);

        // iterate over each set
        let weight_sets = ww.weights.chunks(ww.nfx * ww.nfy * ww.nfz);

        let mut attributes: Attributes = Attributes::new();
        for (i, set) in weight_sets.enumerate() {
            // reorder back into voxel i-j-k indexing rom cell k-j-i indexing
            let mut results = Self::sort_set(set, &cell_order);

            if self.resolution > 1 {
                results = Self::repeat_values(results, self.get_resolution(&ww.ncz));
            }

            let cell_data = DataArray {
                // todo: do something more clever here later
                name: f!("group_{i}"),
                elem: ElementType::Scalars {
                    num_comp: 1,
                    lookup_table: None,
                },
                data: IOBuffer::F64(results),
            };
            attributes.cell.push(Attribute::DataArray(cell_data));
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

    /// Get the correct ordering required for cell index back to voxel index
    fn get_order(ww: &WeightWindow) -> Vec<usize> {
        (0..ww.nfx * ww.nfy * ww.nfz)
            .map(|cell_idx| ww.cell_index_to_voxel_index(cell_idx))
            .collect()
    }

    // todo there is surely a better way to do this
    fn sort_set(values: &[f64], keys: &[usize]) -> Vec<f64> {
        let mut new_vec = values.iter().zip(keys.iter()).collect::<Vec<_>>();
        new_vec.sort_by_key(|&(_, key)| key);
        new_vec.into_iter().map(|(value, _)| *value).collect()
    }
}
