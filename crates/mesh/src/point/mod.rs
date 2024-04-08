//! Utilities for extracting points from meshes
//!
//! The main entry points of the module are [find_voxel()] and [find_voxels()]
//! for returning the [Voxel]s found at each location.
//!
//! It is not assumed that the [Point] provided is in the format appropriate for
//! the mesh geometry, and will be quitely converted to the correct geometry
//! type in the background. i.e. [Geometry::Rectangular] for XYZ meshes and
//! [Geometry::Cylindrical] for RZT.
//!
//! ## Defining points directly
//!
//! This can of course be done thgough the struct explicitly, but for
//! convenience there are several methods for initialising points without
//! having to define common defaults.
//!
//! ```rust
//! # use meshtal::point::Point;
//! # use meshtal::mesh::{Geometry, Group};
//! // Explcict definition
//! Point {
//!     e: Group::Total,
//!     t: Group::Total,
//!     i: 1.0,
//!     j: 2.0,
//!     k: 3.0,
//!     coordinate_type: Geometry::Rectangular
//! };
//!
//! // Convenience methods
//! Point::from_xyz(1.0, 2.0, 3.0);
//! Point::from_rzt(1.0, 2.0, 0.5);
//! Point::from_xyz_vec(&vec![1.0, 2.0, 3.0]);
//! Point::from_rzt_vec(&vec![1.0, 2.0, 0.5]);
//! ```
//!
//! Another useful method set converted between geometry types
//!
//! ```rust
//! # use meshtal::point::Point;
//! # use meshtal::mesh::{Geometry, Group};
//! // Default rectangular point
//! let mut point = Point::new();
//!
//! // Convert to cylindrical equivalent
//! point = point.as_rzt();
//!
//! // Convert back to original cartesian
//! point = point.as_xyz();
//! ```
//!
//! ## Points from an input file
//!  
//! Using [read_points_file()] reads an input file that will be interpreted with
//! the following rules for each line:
//!
//! | Example line               | Interpretation           |
//! | -------------------------- | ------------------------ |
//! | Starts with `#`            | comment                  |
//! | `rzt`, `cyl`, `xyz`, `rec` | geometry keyword         |
//! | 1.0 2.0 3.0                | i, j, k                  |
//! | 1e2  1.0 2.0 3.0           | energy, i, j, k          |
//! | 1e2 total 1.0 2.0 3.0      | energy, time, i, j, k    |
//!
//! Anything else is ignored. For examples see `data/points.txt`.
//!
//! Example
//! ```rust
//! # use meshtal::point::{read_points_file, Point};
//! // Read any valid user points from a text file
//! let points: Vec<Point> = read_points_file("./data/points.txt").unwrap();
//! ```
//!
//! Any location specified in the input file will also be converted to the
//! correct mesh geometry type automatically for convenience.
