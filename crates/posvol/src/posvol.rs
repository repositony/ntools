// ntools modules
use ntools_format::f;

// external crates
use serde::{Deserialize, Serialize};

/// Representation of data in a UKAEA CuV posvol file
///
/// The byte layout is very simple. The 6 dimension values in the first block
/// are stored as [Dimensions].
///
/// ```text
/// <block byte length>
///     <resolution i> <resolution j> <resolution k>
///     <iints+1> <jints+1> <kints+1>
/// <block byte length>
/// ```
///
/// The second block contains all cell data in a continuous array, and is stored
/// as a vector of cell values (`Vec<i32>`).
///
/// ```text
/// <block byte length>
///     <voxel 0, subvoxel 0> <voxel 0, subvoxel 1>  <voxel 0, subvoxel 2> ...
///     <voxel 1, subvoxel 0> <voxel 1, subvoxel 1>  <voxel 1, subvoxel 2> ...
///     ... and so on
/// <block byte length>
/// ```
#[derive(Debug, Serialize, Default)]
pub struct Posvol {
    /// The dimensions given in the first block of data
    pub dimensions: Dimensions,
    /// List of dominant cells for every subvoxel
    pub cells: Vec<i32>,
}

impl Posvol {
    /// Vector of subvoxel cell groups
    ///
    /// Extremely common to iterate over the voxels in chunks of subvoxel cells.
    pub fn subvoxels(&self) -> Vec<&[i32]> {
        self.cells
            .chunks_exact(self.dimensions.number_of_subvoxels())
            .collect()
    }

    /// Number of voxels expected in the file
    pub fn number_of_voxels(&self) -> usize {
        self.dimensions.number_of_voxels()
    }

    /// Number of samples per voxel expected in the file
    pub fn number_of_subvoxels(&self) -> usize {
        self.dimensions.number_of_subvoxels()
    }

    /// Total number of cells expected in the file
    pub fn number_of_cells(&self) -> usize {
        self.dimensions.number_of_cells()
    }
}

impl std::fmt::Display for Posvol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = "Posvol {\n".to_string();
        s += &f!(
            "    voxels: {} ({}x{}x{})\n",
            self.dimensions.number_of_voxels(),
            self.dimensions.n_x - 1,
            self.dimensions.n_y - 1,
            self.dimensions.n_z - 1
        );
        s += &f!(
            "    subvoxels: {} ({}x{}x{})\n",
            self.dimensions.number_of_subvoxels(),
            self.dimensions.res_x,
            self.dimensions.res_y,
            self.dimensions.res_z
        );
        s += &f!(
            "    cells: {} ({}x{})\n}}",
            self.dimensions.number_of_cells(),
            self.dimensions.number_of_voxels(),
            self.dimensions.number_of_subvoxels(),
        );

        write!(f, "{}", s)
    }
}

/// Dimension values in the first [Posvol] data block
///
/// Stores the six dimension values in the first block of binary data.
///
/// Fields correspond to the sample resolution in x, y, and z dimensions, and
/// the number of mesh bounds in each mesh coordinate axis.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Dimensions {
    /// Sample resolution in x
    pub res_x: i32,
    /// Sample resolution in y
    pub res_y: i32,
    /// Sample resolution in z
    pub res_z: i32,

    /// Number of mesh bounds in x (iints+1)
    pub n_x: i32,
    /// Number of mesh bounds in y (iints+1)
    pub n_y: i32,
    /// Number of mesh bounds in z (iints+1)
    pub n_z: i32,
}

impl Dimensions {
    /// Number of voxels expected in the file
    ///
    /// Taken from the mesh bounds found in the dimensions of the file header.
    pub fn number_of_voxels(&self) -> usize {
        ((self.n_x - 1) * (self.n_y - 1) * (self.n_z - 1)) as usize
    }

    /// Number of samples per voxel expected in the file
    ///
    /// For example: If the sample resolution is 3x4x5, then there should be
    /// 60 regions (sub-voxels) inside every voxel.
    pub fn number_of_subvoxels(&self) -> usize {
        (self.res_x * self.res_y * self.res_z) as usize
    }

    /// Total number of cells expected in the file
    ///
    /// The product of the number of voxels and the number of samples per voxel.
    ///
    /// For example: If the sample resolution is 3x4x5 over a mesh with 100
    /// voxels, then there will be 100*60 total cells. i.e. one per sub-voxel
    /// region.
    pub fn number_of_cells(&self) -> usize {
        self.number_of_voxels() * self.number_of_subvoxels()
    }

    /// Expected size of full cells array based on the header dimensions
    pub fn cell_array_byte_length(&self) -> usize {
        self.number_of_cells() * std::mem::size_of::<i32>()
    }
}

impl std::fmt::Display for Dimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
