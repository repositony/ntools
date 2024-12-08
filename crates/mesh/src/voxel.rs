//! Module for voxel-related data and implementations

// ntools modules
use crate::group::Group;
use ntools_utils::ValueExt;

// standard library
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

/// Representation of a single voxel in the mesh
///
/// The global `index` of the voxel is included to maintain consistency between
/// output [Format](crate::format::Format) variants. Parsing line-by-line rather
/// than trying to load an entire file into memory leaves the voxels in an
/// inconsistent order for several formats.
///
/// ## Memory usage
///
/// The minimum information required would be just the result and uncertainty
/// (16 Bytes), and the maximum would include the cooridinates and energy/time
/// groups (80 Bytes).
///
/// The current implementation is a compromise between the two at 24 Bytes, and
/// all other values may be derived from the [Mesh](crate::mesh::Mesh) given the
/// voxel index.
///
/// Several operators are implemented for convenience where it makes sense,
/// including Addition (`+`, `+=`), Subtraction (`-`, `-=`),
/// Multiplication (`*`, `*=`), and Division (`/`, `/=`).
///
/// In all cases, the LHS index is taken, and the RHS may be either another
/// [Voxel] or anything that can be converted into an `f64` primitive.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Voxel {
    /// Global voxel index
    pub index: usize,
    /// Tallied voxel result
    pub result: f64,
    /// Relative error on result
    pub error: f64,
}

impl Voxel {
    /// Returns the absolute error for the voxel
    ///
    /// Example:
    ///
    ///```rust
    /// # use ntools_mesh::Voxel;
    /// let voxel = Voxel {
    ///     result: 50.0,
    ///     error: 0.10,
    ///     ..Default::default()
    /// };
    ///
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(voxel.absolute_error(), 5.0);
    /// ```
    pub fn absolute_error(&self) -> f64 {
        (self.result * self.error).abs()
    }

    /// Returns the relative error for the voxel
    ///
    /// The MCNP mesh results are output and stored as the relative
    /// uncertainty anyway. However, having both
    /// [absolute_error()](Voxel::absolute_error) and
    /// [relative_error()](Voxel::relative_error) methods makes intent
    /// explicit.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mesh::Voxel;
    /// let voxel = Voxel {
    ///     result: 50.0,
    ///     error: 0.10,
    ///     ..Default::default()
    /// };
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(voxel.relative_error(), 0.1);
    /// ```
    pub fn relative_error(&self) -> f64 {
        self.error.abs()
    }

    /// Raise the voxel to some power
    ///
    ///```rust
    /// # use ntools_mesh::Voxel;
    /// let voxel = Voxel {
    ///     result: 10.0,
    ///     error: 0.10,
    ///     ..Default::default()
    /// };
    ///
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(voxel.powf(2.0).result, 100.0);
    /// assert_eq!(voxel.powf(2.0).error, 2.0);
    /// ```
    pub fn powf(self, value: impl Into<f64>) -> Voxel {
        let v = value.into();
        let error = self.absolute_error() * v;
        Self {
            index: self.index,
            result: self.result.powf(v),
            error,
        }
    }
}

impl std::fmt::Display for Voxel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:<5.0}{:>13}{:>13}",
            self.index,
            self.result.sci(5, 2),
            self.error.sci(5, 2)
        )
    }
}

impl Add<Self> for Voxel {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let result = self.result + other.result;
        let absolute_error =
            (self.absolute_error().powi(2) + other.absolute_error().powi(2)).sqrt();

        // turn into relative error if appropriate, otherwise follow MCNP
        // and cap to 1.0 as meaningless
        let relative_error = if absolute_error > result {
            1.0
        } else {
            absolute_error / result
        };

        Self {
            index: self.index,
            result,
            error: relative_error.abs(),
        }
    }
}

impl AddAssign<Self> for Voxel {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T> Add<T> for Voxel
where
    T: Into<f64>,
{
    type Output = Self;
    fn add(self, other: T) -> Self {
        let result = self.result + other.into();
        let relative_error = if self.error > result {
            1.0
        } else {
            self.error / result
        };

        Self {
            index: self.index,
            result,
            error: relative_error.abs(),
        }
    }
}

impl<T> AddAssign<T> for Voxel
where
    T: Into<f64>,
{
    fn add_assign(&mut self, other: T) {
        *self = *self + other.into();
    }
}

impl Sub<Self> for Voxel {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let result = self.result - other.result;
        let absolute_error =
            (self.absolute_error().powi(2) + other.absolute_error().powi(2)).sqrt();

        // turn into relative error if appropriate, otherwise follow MCNP
        // and cap to 1.0 as meaningless
        let relative_error = if absolute_error > result {
            1.0
        } else {
            absolute_error / result
        };

        Self {
            index: self.index,
            result,
            error: relative_error.abs(),
        }
    }
}

impl SubAssign<Self> for Voxel {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<T> Sub<T> for Voxel
where
    T: Into<f64>,
{
    type Output = Self;
    fn sub(self, other: T) -> Self {
        let result = self.result - other.into();
        let relative_error = if self.error > result {
            1.0
        } else {
            self.error / result
        };

        Self {
            index: self.index,
            result,
            error: relative_error.abs(),
        }
    }
}

impl<T> SubAssign<T> for Voxel
where
    T: Into<f64>,
{
    fn sub_assign(&mut self, other: T) {
        *self = *self - other.into();
    }
}

impl Mul<Self> for Voxel {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            index: self.index,
            result: self.result * other.result,
            error: (self.error.powi(2) + other.error.powi(2)).sqrt(),
        }
    }
}

impl MulAssign<Self> for Voxel {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<T> Mul<T> for Voxel
where
    T: Into<f64>,
{
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self {
            index: self.index,
            result: self.result * other.into(),
            error: self.error,
        }
    }
}

impl<T> MulAssign<T> for Voxel
where
    T: Into<f64>,
{
    fn mul_assign(&mut self, other: T) {
        *self = *self * other.into();
    }
}

impl Div<Self> for Voxel {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        // for now retun something that looks invalid by MCNP standards when
        // dividing by zero
        let (result, error) = if other.result == 0.0 {
            (0.0, 1.0)
        } else {
            (
                self.result / other.result,
                (self.error.powi(2) + other.error.powi(2)).sqrt(),
            )
        };

        Self {
            index: self.index,
            result,
            error,
        }
    }
}

impl DivAssign<Self> for Voxel {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl<T> Div<T> for Voxel
where
    T: Into<f64>,
{
    type Output = Self;
    fn div(self, other: T) -> Self {
        Self {
            index: self.index,
            result: self.result / other.into(),
            error: self.error,
        }
    }
}

impl<T> DivAssign<T> for Voxel
where
    T: Into<f64>,
{
    fn div_assign(&mut self, other: T) {
        *self = *self / other.into();
    }
}

/// Convenience structure for collecting voxel coordiante information
///
/// The coordinate data of each [Voxel] are a complete set. Every coordiante is
/// derivable from the global voxel index given knowledge of the
/// [Mesh](crate::mesh::Mesh) fields. It is therefore primarily used in
/// [Mesh](crate::mesh::Mesh) methods.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VoxelCoordinate {
    /// Energy group (MeV)
    pub energy: Group,
    /// Time group (shakes)
    pub time: Group,
    /// i coordinate at the voxel centre
    pub i: f64,
    /// j coordinate at the voxel centre
    pub j: f64,
    /// k coordinate at the voxel centre
    pub k: f64,
}

impl Default for VoxelCoordinate {
    fn default() -> Self {
        Self {
            energy: Group::Total,
            time: Group::Total,
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }
}
