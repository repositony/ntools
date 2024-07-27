// standard library
use std::ops::RangeInclusive;

// ntools modules
use ntools_support::f;

// internal modules
use crate::vtk::builder::WeightsToVtkBuilder;
use crate::WeightWindow;

// extrenal crates
use vtkio::model::{
    Attribute, Attributes, ByteOrder, Coordinates, DataArray, DataSet, ElementType, Extent,
    IOBuffer, RangeExtent, RectilinearGridPiece, Version, Vtk,
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
            2 => todo!(),
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
