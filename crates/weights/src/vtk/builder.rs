// internal modules
use crate::vtk::convert::WeightsToVtk;
use crate::vtk::ByteOrder;

// extrenal crates
use vtkio::xml::Compressor;

/// Builder implementation for WeightsToVtk configuration
///
/// The fields of [WeightsToVtk] are left public for direct use but the module
/// also implements a builder.
///
/// For those not familiar, the builder allows for chained setter calls for a
/// functional approach that could be considered more readable. Any number of
/// parameters can be set this way (including none).
///
/// To get the final [WeightsToVtk] from the builder, call
/// [build()](WeightsToVtkBuilder::build).  
///
/// ```rust, no_run
/// # use ntools_weights::vtk::{WeightsToVtk, weights_to_vtk};
/// # use ntools_weights::vtk::write_vtk;
/// # use ntools_weights::vtk::VtkFormat;
/// # use vtkio::{xml::Compressor, model::ByteOrder};
/// # use ntools_weights::WeightWindow;
/// // Make a new builder, change some values
/// let converter = WeightsToVtk::builder()
///     .resolution(3)
///     .compressor(Compressor::LZMA)
///     .byte_order(ByteOrder::LittleEndian)
///     .build();
///
/// // Convert the weight windows using the parameters set
/// let vtk = weights_to_vtk(&WeightWindow::default());
///
/// // Wite the VTK to a file in one of several formats
/// write_vtk(vtk, "output.vtk", VtkFormat::Xml).unwrap();
/// ```
///
/// This helps separate the configuration from the actual conversion logic, and
/// is often a style preference for many users.
pub struct WeightsToVtkBuilder {
    /// Byte ordering as big or little endian
    byte_order: ByteOrder,
    /// compression method for xml file formats
    compressor: Compressor,
    /// Cylindrical mesh resolution
    resolution: u8,
}

impl WeightsToVtkBuilder {
    /// Create a new instance of the builder with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the [WeightsToVtk] type
    pub fn build(self) -> WeightsToVtk {
        WeightsToVtk {
            byte_order: self.byte_order,
            compressor: self.compressor,
            resolution: self.resolution,
        }
    }

    /// Cylindrical mesh resolution
    ///
    /// Warning: Every vertex is defined explicitly, so large values will
    /// significantly increase memory usage and file size.
    ///
    /// Integer value for increasing angular resolution of cylindrical meshes.
    /// Cylinders are approximated to straight edge segments so it can be useful
    /// to round this off by splitting voxels into multiple smaller segments.
    ///
    /// e.g. 4 theta bins gives 4 edges and therefore looks square. Using
    /// `--resolution 3` generates 12 edges instead and looks more rounded in
    /// plots.
    pub fn resolution(mut self, resolution: u8) -> Self {
        self.resolution = resolution;
        self
    }

    /// Set the byte ordering
    ///
    /// Note that Visit being Visit only reads big endian, even though most
    /// systems are little endian. The byte order has one variant of the
    /// ByteOrder, defaulting to big endian for convenience.
    pub fn byte_order(mut self, order: ByteOrder) -> Self {
        self.byte_order = order;
        self
    }

    /// Set the compression method for xml file formats
    ///
    /// Generally just use LZMA but other options are available:
    /// - lzma (default)
    /// - lz4
    /// - zlib
    /// - none
    pub fn compressor(mut self, xml_compressor: Compressor) -> Self {
        self.compressor = xml_compressor;
        self
    }
}

impl Default for WeightsToVtkBuilder {
    fn default() -> Self {
        Self {
            byte_order: ByteOrder::BigEndian,
            compressor: Compressor::LZMA,
            resolution: 1,
        }
    }
}
