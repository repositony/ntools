// internal modules
use crate::error::{Error, Result};
use crate::Group;
use ntools_utils::{f, ValueExt};

/// Variants for the type of [Point] coordinates
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PointKind {
    /// Point (i, j, k) interpreted as indicies
    Index = 0,
    #[default]
    /// Point (i, j, k) interpreted as cartesian (x, y, z)
    Rectangular = 1,
    /// Point (i, j, k) interpreted as cylindrical (r, z, t)
    Cylindrical = 2,
}

impl std::fmt::Display for PointKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Self::Index => "Index",
            Self::Cylindrical => "Cylindrical",
            Self::Rectangular => "Rectangular",
        };
        write!(f, "{}", s)
    }
}

/// Method variants for dealing with a [Point] on boundaries
///
/// Leaving the treatment of a [Point] on a voxel boundary to the user's
/// imagination is a bad idea. Providing a variant of this type forces the
/// behaviour to be explicit.
///
/// The variants may be:
/// - **Lower** - ON the boundary returns the lower voxel result
/// - **Upper** - ON the boundary returns the higher voxel result
/// - **Average** - NEAR the boundary returns average of both voxels
///
/// For example, if the x-bounds of two voxels were [1.0, 2.0, 3.0]
/// and you specified x=2.0:
///     > lower            => (1.0 <  x <= 2.0), choose voxel 0
///     > upper            => (2.0 <= x <  3.0), choose voxel 1
///     > average, tol=0.1 => (1.9 <  x <  2.1), average both
///
/// Boundary cases are special and will be included for each extreme. For
/// example, `Upper` will return the last voxel for exactly 3.0 in this case.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryTreatment {
    /// Values exactly on a boundary return the lower voxel
    Lower,
    /// Values exactly on a boundary return the higher voxel
    Upper,
    /// Values within a tolerance of a boundary return an average of both voxels
    Average(f64),
}

impl Default for BoundaryTreatment {
    fn default() -> Self {
        Self::Average(0.001)
    }
}

impl std::fmt::Display for BoundaryTreatment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match *self {
            Self::Average(tol) => f!("Average (tol={tol})"),
            Self::Lower => "Lower".to_string(),
            Self::Upper => "Upper".to_string(),
        };
        write!(f, "{}", s)
    }
}

/// Generic representation of a point in the mesh geometry
///
/// A [Point] represents a location somewhere in the mesh data. It must specify
/// the time and energy groups, the (i,j,k) coordinates, and how these values
/// should be interpreted.
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// Energy [Group](crate::group::Group)
    pub e: Group,
    /// Time [Group](crate::group::Group)
    pub t: Group,
    /// i coordinate
    pub i: f64,
    /// j coordinate
    pub j: f64,
    /// k coordinate
    pub k: f64,
    /// Coordiante system
    pub kind: PointKind,
}

impl Default for Point {
    fn default() -> Self {
        Self {
            e: Group::Total,
            t: Group::Total,
            i: 0.0,
            j: 0.0,
            k: 0.0,
            kind: PointKind::default(),
        }
    }
}

impl Point {
    /// Create a new [Point] with the default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a [Point] from (x,y,z) cartesian coordinates
    ///
    /// Anything that can be turned into an `f64` value will work. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// assert_eq!( Point::from_xyz(1, 2.0, 3),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 2.0,
    ///                 k: 3.0,
    ///                 kind: PointKind::Rectangular,
    ///                 ..Default::default()})
    /// ```
    pub fn from_xyz<T, U, V>(x: T, y: U, z: V) -> Self
    where
        T: Into<f64> + Copy,
        U: Into<f64> + Copy,
        V: Into<f64> + Copy,
    {
        Self {
            i: x.into(),
            j: y.into(),
            k: z.into(),
            ..Default::default()
        }
    }

    /// Create a [Point] from (r,z,t) cylindrical coordinates
    ///
    /// Anything that can be turned into an `f64` value will work. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// assert_eq!( Point::from_rzt(1, 10, 0.5),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 10.0,
    ///                 k: 0.5,
    ///                 kind: PointKind::Cylindrical,
    ///                 ..Default::default()})
    /// ```
    pub fn from_rzt<T, U, V>(r: T, z: U, t: V) -> Self
    where
        T: Into<f64> + Copy,
        U: Into<f64> + Copy,
        V: Into<f64> + Copy,
    {
        Self {
            i: r.into(),
            j: z.into(),
            k: t.into(),
            kind: PointKind::Cylindrical,
            ..Default::default()
        }
    }

    /// Create a [Point] from (i,j,k) indexing
    ///
    /// Note that any non-exact values will be cast to `usize` at the time of
    /// use. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// assert_eq!( Point::from_ijk(1, 2, 3),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 2.0,
    ///                 k: 3.0,
    ///                 kind: PointKind::Index,
    ///                 ..Default::default()})
    /// ```
    pub fn from_ijk<T, U, V>(i: T, j: U, k: V) -> Self
    where
        T: Into<f64> + Copy,
        U: Into<f64> + Copy,
        V: Into<f64> + Copy,
    {
        Self {
            i: i.into(),
            j: j.into(),
            k: k.into(),
            kind: PointKind::Index,
            ..Default::default()
        }
    }

    /// Create a [Point] from an array of `[x,y,z]` cartesian coordinates
    ///
    /// Anything that can be turned into an `f64` value will work. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// let xyz = vec![1.0, 2.0, 3.0];
    /// assert_eq!( Point::from_xyz_vec(&xyz).unwrap(),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 2.0,
    ///                 k: 3.0,
    ///                 kind: PointKind::Rectangular,
    ///                 ..Default::default()})
    /// ```
    pub fn from_xyz_vec<T>(values: &[T]) -> Result<Self>
    where
        T: Into<f64> + Copy,
    {
        match values.len() {
            3 => Ok(Point {
                i: values[0].into(),
                j: values[1].into(),
                k: values[2].into(),
                ..Default::default()
            }),
            _ => Err(Error::UnexpectedLength {
                expected: 3,
                found: values.len(),
            }),
        }
    }

    /// Create a [Point] from an array of `[r,z,t]` cylindrical coordinates
    ///
    /// Anything that can be turned into an `f64` value will work. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// let rzt = vec![1.0, 2.0, 3.0];
    /// assert_eq!( Point::from_rzt_vec(&rzt).unwrap(),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 2.0,
    ///                 k: 3.0,
    ///                 kind: PointKind::Cylindrical,
    ///                 ..Default::default()})
    /// ```
    pub fn from_rzt_vec<T>(values: &[T]) -> Result<Self>
    where
        T: Into<f64> + Copy,
    {
        match values.len() {
            3 => Ok(Point {
                i: values[0].into(),
                j: values[1].into(),
                k: values[2].into(),
                kind: PointKind::Cylindrical,
                ..Default::default()
            }),
            _ => Err(Error::UnexpectedLength {
                expected: 3,
                found: values.len(),
            }),
        }
    }

    /// Create a [Point] from an array of `[i,j,k]` indices
    ///
    /// Note that any non-exact values will be cast to `usize` at the time of
    /// use. For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// let ijk = vec![1, 2, 3];
    /// assert_eq!( Point::from_ijk_vec(&ijk).unwrap(),
    ///             Point{
    ///                 i: 1.0,
    ///                 j: 2.0,
    ///                 k: 3.0,
    ///                 kind: PointKind::Index,
    ///                 ..Default::default()})
    /// ```
    pub fn from_ijk_vec<T>(values: &[T]) -> Result<Self>
    where
        T: Into<f64> + Copy,
    {
        match values.len() {
            3 => Ok(Point {
                i: values[0].into(),
                j: values[1].into(),
                k: values[2].into(),
                kind: PointKind::Index,
                ..Default::default()
            }),
            _ => Err(Error::UnexpectedLength {
                expected: 3,
                found: values.len(),
            }),
        }
    }

    /// Turn point into an array
    ///
    /// For example:
    /// ```rust
    /// # use ntools_mesh::{Point, PointKind};
    /// let point = Point{  i: 1.0,
    ///                     j: 2.0,
    ///                     k: 3.0,
    ///                     ..Default::default()};
    /// assert_eq!( point.as_array(), [1.0, 2.0, 3.0] )
    /// ```
    pub fn as_array(&self) -> [f64; 3] {
        [self.i, self.j, self.k]
    }

    // /// Rotate a point about the origin
    // pub fn rotate(&mut self, rotation: &Rotation<f64, 3>) {
    //     let a = rotation.transform_vector(&Vector3::from(self.as_array()));
    //     self.i = a[0];
    //     self.j = a[1];
    //     self.k = a[2];
    // }

    // /// Pass along simple translation by (x,y,z) cartesian coordinates
    // pub fn translate(&mut self, translation: &[f64; 3]) {
    //     self.i += translation[0];
    //     self.j += translation[1];
    //     self.k += translation[2];
    // }

    // /// Inverts a translation of (x,y,z) cartesian coordinates
    // pub fn translate_inv(&mut self, translation: &[f64; 3]) {
    //     self.i -= translation[0];
    //     self.j -= translation[1];
    //     self.k -= translation[2];
    // }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut s = match self.e {
            Group::Value(e) => f!("({:>13}, ", e.sci(5, 2)),
            Group::Total => f!("({:>13}, ", "Total"),
        };

        s += &match self.t {
            Group::Value(t) => f!("{:>13}, ", t.sci(5, 2)),
            Group::Total => f!("{:>13}, ", "Total"),
        };

        s += &f!(
            "{:>13},{:>13},{:>13})",
            self.i.sci(5, 2),
            self.j.sci(5, 2),
            self.k.sci(5, 2)
        );

        write!(f, "{}", s)
    }
}
