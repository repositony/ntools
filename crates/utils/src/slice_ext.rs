use crate::error::{Error, Result};

/// Extends functionality for slices of float arrays
pub trait SliceExt<T> {
    /// Find the minimum value in float arrays
    ///
    /// Only provides the minimum value from a collection of valid numbers. Any
    /// NAN values, infinite values, or empty slices will return an error.
    ///
    /// ```rust
    /// # use ntools_utils::SliceExt;
    /// # use ntools_utils::Error;
    /// // Successful cases
    /// assert_eq!([1.1, 0.5, 2.2].try_min(), Ok(0.5));
    /// assert_eq!([1.1, f32::MIN, 2.2].try_min(), Ok(f32::MIN));
    ///
    /// // Error cases
    /// assert_eq!([1.1, f32::NAN, 2.2].try_min(), Err(Error::SliceContainsUndefinedValues));
    /// assert_eq!([1.1, f32::INFINITY, 2.2].try_min(), Err(Error::SliceContainsUndefinedValues));
    /// assert_eq!(Vec::<f32>::new().try_min(), Err(Error::SliceContainsNoValues));
    /// ```
    ///
    /// The float primitives (`f32`/`f64`) do not implement `Ord`` due to `NaN`
    /// being incomparable. Calling `min()` on a collection of floats is
    /// therefore not implemented in the standard library.
    ///
    /// This extension uses `total_cmp` to always produce an ordering in
    /// accordance to the totalOrder predicate as defined in the IEEE 754 (2008
    /// revision) floating point standard.
    fn try_min(&self) -> Result<T>;

    /// Find the maximum value in float arrays
    ///
    /// Only provides the maximum value from a collection of valid numbers. Any
    /// NAN values, infinite values, or empty slices will return an error.
    ///
    /// ```rust
    /// # use ntools_utils::SliceExt;
    /// # use ntools_utils::Error;
    /// // Successful cases
    /// assert_eq!([1.1, 0.5, 2.2].try_max(), Ok(2.2));
    /// assert_eq!([1.1, f32::MAX, 2.2].try_max(), Ok(f32::MAX));
    ///
    /// // Error cases
    /// assert_eq!([1.1, f32::NAN, 2.2].try_max(), Err(Error::SliceContainsUndefinedValues));
    /// assert_eq!([1.1, f32::INFINITY, 2.2].try_max(), Err(Error::SliceContainsUndefinedValues));
    /// assert_eq!(Vec::<f32>::new().try_max(), Err(Error::SliceContainsNoValues));
    /// ```
    ///
    /// The float primitives (`f32`/`f64`) do not implement `Ord`` due to `NaN`
    /// being incomparable. Calling `max()` on a collection of floats is
    /// therefore not implemented in the standard library.
    ///
    /// This extension uses `total_cmp` to always produce an ordering in
    /// accordance to the totalOrder predicate as defined in the IEEE 754 (2008
    /// revision) floating point standard.
    fn try_max(&self) -> Result<T>;

    /// Find index bin containing 'value', where bins are low < value <= high
    ///
    /// A value on a bin edge returns the bin below. Values equal to the lowest
    /// bound are considered part of the first bin.
    ///
    /// # Example
    /// ```text
    ///     MCNP   : emesh = 0.1 1.0 20.0
    ///     Meshtal: 0.00E+00 1.00E-01 1.00E+00 2.00E+01
    /// ```
    ///
    /// view of the requested energy groups
    /// ```text
    ///     0.0 <= bin 0 <= 0.1
    ///     0.1 < bin 1 <= 1.0
    ///     1.0 < bin 2 <= 20.0
    /// ```
    ///
    /// ```rust
    /// # use ntools_utils::SliceExt;
    /// let bounds = vec![0.0, 0.1, 1.0, 20.0];
    ///
    /// // Find values in the array
    /// assert_eq!(bounds.find_bin_inclusive(0.0 ), Ok(0));
    /// assert_eq!(bounds.find_bin_inclusive(0.5 ), Ok(1));
    /// assert_eq!(bounds.find_bin_inclusive(1.0 ), Ok(1));
    /// assert_eq!(bounds.find_bin_inclusive(20.0), Ok(2));
    ///
    /// // Values outside the bin bounds are an error case
    /// assert!(bounds.find_bin_inclusive(-1.0).is_err());
    /// assert!(bounds.find_bin_inclusive(21.0).is_err());
    /// ```
    fn find_bin_inclusive(&self, value: T) -> Result<usize>;

    /// Find index bin containing 'value', where bins are low <= value < high
    ///
    /// A value on a bin edge returns the bin above. Values equal to the highest
    /// bound are considered part of the last bin.
    ///
    /// This mirrors the actual MCNP binning behaviour. `EMESH` card entries are
    /// upper edges, so in general values on a boundary will be recorded in the
    /// bin above. A special case is made for energies exactly on the last upper
    /// edge, since this is actually included in the tallied results.
    ///
    ///  # Example
    /// ```text
    ///     MCNP card : EMESH = 0.1 1.0 20.0
    ///     Mesh.emesh: 0.00E+00 1.00E-01 1.00E+00 2.00E+01
    /// ```
    ///
    /// MCNP view of these bins:
    /// ```text
    ///     0.0 <= bin 0 < 0.1
    ///     0.1 <= bin 1 < 1.0
    ///     1.0 <= bin 2 <= 20.0
    /// ```
    ///
    /// ```rust
    /// # use ntools_utils::SliceExt;
    /// let bounds = vec![0.0, 0.1, 1.0, 20.0];
    ///
    /// // Find values in the array
    /// assert_eq!(bounds.find_bin_exclusive(0.0 ), Ok(0));
    /// assert_eq!(bounds.find_bin_exclusive(0.5 ), Ok(1));
    /// assert_eq!(bounds.find_bin_exclusive(1.0 ), Ok(2));
    /// assert_eq!(bounds.find_bin_inclusive(20.0), Ok(2));
    ///
    /// // Values outside the bin bounds are an error case
    /// assert!(bounds.find_bin_exclusive(-1.0).is_err());
    /// assert!(bounds.find_bin_exclusive(21.0).is_err());
    /// ```
    fn find_bin_exclusive(&self, value: T) -> Result<usize>;

    /// Find the index or indicies of bins a value falls on
    ///
    /// This handles the case where you may need to average across multiple bins
    /// when on/near a bin edge.
    ///
    /// Notes:
    ///     - Bin edges must be in ascending order
    ///     - Tolerance is a factor, not absolute or percentage
    ///
    /// ```rust
    /// # use ntools_utils::SliceExt;
    /// let bounds = vec![0.0, 10.0, 20.0];
    ///
    /// // Find one index when not near a bin edge
    /// assert_eq!(bounds.find_bin_average(5.0, 0.1), Ok(vec![0]));
    ///
    /// // Find both indices when within 10% of the bin edge
    /// assert_eq!(bounds.find_bin_average( 9.5, 0.1), Ok(vec![0,1]));
    /// assert_eq!(bounds.find_bin_average(10.2, 0.1), Ok(vec![0,1]));
    ///
    /// // Values outside the bin bounds are an error case
    /// assert!(bounds.find_bin_average(-1.0, 0.1).is_err());
    /// assert!(bounds.find_bin_average(21.0, 0.1).is_err());
    /// ```
    fn find_bin_average(&self, value: T, tol: T) -> Result<Vec<usize>>;
}

impl SliceExt<f64> for [f64] {
    fn try_min(&self) -> Result<f64> {
        if self.iter().any(|v| !v.is_finite()) {
            return Err(Error::SliceContainsUndefinedValues);
        };

        if let Some(v) = self.iter().min_by(|a, b| a.total_cmp(b)).copied() {
            Ok(v)
        } else {
            Err(Error::SliceContainsNoValues)
        }
    }

    fn try_max(&self) -> Result<f64> {
        if self.iter().any(|v| !v.is_finite()) {
            return Err(Error::SliceContainsUndefinedValues);
        };

        if let Some(v) = self.iter().max_by(|a, b| a.total_cmp(b)).copied() {
            Ok(v)
        } else {
            Err(Error::SliceContainsNoValues)
        }
    }

    fn find_bin_inclusive(&self, value: f64) -> Result<usize> {
        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value,
                lower_bound: *lower_bound,
                upper_bound: *upper_bound,
            });
        }

        // special case for being on the lowest edge
        if &value == lower_bound {
            return Ok(0);
        }

        // try to find the bin index, range INCLUSIVE of upper edge
        for (idx, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            if low < &value && &value <= high {
                return Ok(idx);
            }
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }

    fn find_bin_exclusive(&self, value: f64) -> Result<usize> {
        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value,
                lower_bound: *lower_bound,
                upper_bound: *upper_bound,
            });
        }

        // special case for being on the upper edge
        if &value == upper_bound {
            return Ok(n - 2);
        }

        // try to find the bin index, range INCLUSIVE of upper edge
        for (idx, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            if low <= &value && &value < high {
                return Ok(idx);
            }
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }

    fn find_bin_average(&self, value: f64, tol: f64) -> Result<Vec<usize>> {
        // check the tolerance is reasonable
        if tol > 1.0 {
            return Err(Error::UnreasonableBoundaryTolerance {
                tolerance: tol,
                minimum: 0.0,
                maximum: 1.0,
            });
        }

        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value even relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value,
                lower_bound: *lower_bound,
                upper_bound: *upper_bound,
            });
        }

        // at this point we know it has at least 2 values due to the initial check
        let l_tol = (upper[0] - lower_bound).abs() * tol;
        let r_tol = (upper_bound - lower.last().unwrap()).abs() * tol;

        // can succeed early in the special case for being on the lowest edge
        if value < lower_bound + l_tol {
            return Ok(vec![0]);
        }

        // can succeed early in the special case for being on the upper edge
        if value > upper_bound - r_tol {
            return Ok(vec![n - 2]);
        }

        // try to find the bin index, or bin indicies if within the bound tolerance
        let mut index = Vec::with_capacity(2);

        for (i, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            // oustide the bin bounds, move on
            if value > *high || value < *low {
                continue;
            }

            // calculate the absolute tolerence for the bin value
            let tolerance = (tol * (high - low)).abs();

            // within some tolerance of the lower bin edge
            if value <= low + tolerance {
                if i != 0 {
                    index.push(i - 1);
                }
                index.push(i);
                return Ok(index);
            }
            // within some tolerance of the upper bin edge
            else if value >= high - tolerance {
                index.push(i);
                if i != n - 2 {
                    index.push(i + 1);
                }
                return Ok(index);
            }

            // the final case here is low+tol <= x <= high-tol, and is true by
            // elimiation of the other cases
            index.push(i);
            return Ok(index);
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }
}

impl SliceExt<f32> for [f32] {
    fn try_min(&self) -> Result<f32> {
        if self.iter().any(|v| !v.is_finite()) {
            return Err(Error::SliceContainsUndefinedValues);
        };

        if let Some(v) = self.iter().min_by(|a, b| a.total_cmp(b)).copied() {
            Ok(v)
        } else {
            Err(Error::SliceContainsNoValues)
        }
    }

    fn try_max(&self) -> Result<f32> {
        if self.iter().any(|v| !v.is_finite()) {
            return Err(Error::SliceContainsUndefinedValues);
        };

        if let Some(v) = self.iter().max_by(|a, b| a.total_cmp(b)).copied() {
            Ok(v)
        } else {
            Err(Error::SliceContainsNoValues)
        }
    }

    fn find_bin_inclusive(&self, value: f32) -> Result<usize> {
        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value: value.into(),
                lower_bound: (*lower_bound).into(),
                upper_bound: (*upper_bound).into(),
            });
        }

        // special case for being on the lowest edge
        if &value == lower_bound {
            return Ok(0);
        }

        // try to find the bin index, range INCLUSIVE of upper edge
        for (idx, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            if low < &value && &value <= high {
                return Ok(idx);
            }
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }

    fn find_bin_exclusive(&self, value: f32) -> Result<usize> {
        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value: value.into(),
                lower_bound: (*lower_bound).into(),
                upper_bound: (*upper_bound).into(),
            });
        }

        // special case for being on the upper edge
        if &value == upper_bound {
            return Ok(n - 2);
        }

        // try to find the bin index, range INCLUSIVE of upper edge
        for (idx, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            if low <= &value && &value < high {
                return Ok(idx);
            }
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }

    fn find_bin_average(&self, value: f32, tol: f32) -> Result<Vec<usize>> {
        // check the tolerance is reasonable
        if tol > 1.0 {
            return Err(Error::UnreasonableBoundaryTolerance {
                tolerance: tol.into(),
                minimum: 0.0,
                maximum: 1.0,
            });
        }

        // make sure there are bin edges to check against
        let n = self.len();
        if n < 2 {
            return Err(Error::BelowMinimumSliceLength {
                length: n,
                minimum_required: 2,
            });
        }

        // should be fine to unwrap as not empty
        let (lower_bound, upper) = self.split_first().unwrap();
        let (upper_bound, lower) = self.split_last().unwrap();

        // is the value even relevant?
        if &value < lower_bound || &value > upper_bound {
            return Err(Error::ValueOutsideOfBounds {
                value: value.into(),
                lower_bound: (*lower_bound).into(),
                upper_bound: (*upper_bound).into(),
            });
        }

        // at this point we know it has at least 2 values due to the initial check
        let l_tol = (upper[0] - lower_bound).abs() * tol;
        let r_tol = (upper_bound - lower.last().unwrap()).abs() * tol;

        // can succeed early in the special case for being on the lowest edge
        if value < lower_bound + l_tol {
            return Ok(vec![0]);
        }

        // can succeed early in the special case for being on the upper edge
        if value > upper_bound - r_tol {
            return Ok(vec![n - 2]);
        }

        // try to find the bin index, or bin indicies if within the bound tolerance
        let mut index = Vec::with_capacity(2);

        for (i, (low, high)) in lower.iter().zip(upper.iter()).enumerate() {
            // oustide the bin bounds, move on
            if value > *high || value < *low {
                continue;
            }

            // calculate the absolute tolerence for the bin value
            let tolerance = (tol * (high - low)).abs();

            // within some tolerance of the lower bin edge
            if value <= low + tolerance {
                if i != 0 {
                    index.push(i - 1);
                }
                index.push(i);
                return Ok(index);
            }
            // within some tolerance of the upper bin edge
            else if value >= high - tolerance {
                index.push(i);
                if i != n - 2 {
                    index.push(i + 1);
                }
                return Ok(index);
            }

            // the final case here is low+tol <= x <= high-tol, and is true by
            // elimiation of the other cases
            index.push(i);
            return Ok(index);
        }

        // this should be unreachable
        Err(Error::UncapturedErrorCondition)
    }
}
