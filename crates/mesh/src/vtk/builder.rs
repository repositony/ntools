// internal modules
use crate::vtk::MeshToVtk;

// extrenal crates
use log::warn;
use vtkio::model::ByteOrder;
use vtkio::xml::Compressor;

/// Builder implementation for MeshToVtk configuration
///
/// The fields of [MeshToVtk] are left public for direct use but the module also
/// implements a builder.
///
/// For those not familiar, the builder allows for chained setter calls for a
/// functional approach that could be considered more readable. Any number of
/// parameters can be set this way (including none).
///
/// To get the final [MeshToVtk] from the builder, call
/// [build()](MeshToVtkBuilder::build).  
///
/// ```rust, no_run
/// # use ntools_mesh::vtk::{write_vtk, MeshToVtk, MeshToVtkBuilder, VtkFormat};
/// # use ntools_mesh::{Mesh, Group};
/// # use vtkio::{xml::Compressor, model::ByteOrder};
/// # let mesh = Mesh::default();
/// // Make a new builder, change some values
/// let converter = MeshToVtk::builder()
///     .include_errors(true)
///     .energy_groups(vec![0])     // first energy group
///     .time_groups(vec![1, 2])    // second and third time groups
///     .compressor(Compressor::LZMA)
///     .byte_order(ByteOrder::LittleEndian)
///     .resolution(3)
///     .build();
///
/// // Convert the mesh using the parameters set
/// let vtk = converter.convert(&mesh);
///
/// // Write to "output.vtk" using the old ASCII text format
/// write_vtk(vtk, "./output.vtk", VtkFormat::LegacyAscii).unwrap();
/// ```
///
/// This helps separate the configuration from the actual conversion logic, and
/// is often a style preference for many users.
pub struct MeshToVtkBuilder {
    /// Target energy group(s)
    energy_groups: Vec<usize>,
    /// Target energy group(s)
    time_groups: Vec<usize>,
    /// Include errors mesh in output files
    include_errors: bool,
    /// Byte ordering as big or little endian
    byte_order: ByteOrder,
    /// compression method for xml file formats
    compressor: Compressor,
    /// Cylindrical mesh resolution
    resolution: u8,
}

impl MeshToVtkBuilder {
    /// Create a new instance of the builder with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Build the [MeshToVtk] type
    pub fn build(self) -> MeshToVtk {
        MeshToVtk {
            byte_order: self.byte_order,
            compressor: self.compressor,
            resolution: self.resolution,
            energy_groups: self.energy_groups,
            time_groups: self.time_groups,
            include_errors: self.include_errors,
        }
    }

    /// Target energy group(s)
    ///
    /// By default all energy groups are included in the vtk. Specific energy
    /// groups can be provided to reduce file sizes.
    pub fn energy_groups(mut self, groups: Vec<usize>) -> Self {
        self.energy_groups = groups;
        self
    }

    /// Target time group(s)
    ///
    /// By default all time groups are included in the vtk. Specific time
    /// groups can be provided to reduce file sizes.
    pub fn time_groups(mut self, groups: Vec<usize>) -> Self {
        self.time_groups = groups;
        self
    }

    /// Include errors mesh in output files
    ///
    /// Error meshes omitted by default to save space. If enabled, every mesh
    /// will have a corresponding relative uncertainty dataset. Of course, this
    /// ~doubles file size, which is fine in most cases.
    pub fn include_errors(mut self, include: bool) -> Self {
        self.include_errors = include;
        self
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
        if resolution > 1 {
            warn!(
                "Warning: Increasing cylindrical mesh resolution may increase memory usage significantly"
            );
        }
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

impl Default for MeshToVtkBuilder {
    fn default() -> Self {
        Self {
            byte_order: ByteOrder::BigEndian,
            compressor: Compressor::LZMA,
            resolution: 1,
            energy_groups: Vec::new(),
            time_groups: Vec::new(),
            include_errors: false,
        }
    }
}
