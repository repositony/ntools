use ntools_utils::ValueExt;

/// Energy/Time groups are either `Total` or an upper bin edge
///
/// For energies, the meshtal outputs always define `emesh` bounds so there is
/// always at least one group. For a single energy bin this becomes a
/// [Group::Total] no matter the energy.
///
/// | Energy bounds | Groups                            |
/// | ------------- | --------------------------------- |
/// | None          | Total ("0.0 1e36" in output file) |
/// | 0.0 100       | Total                             |
/// | 0.0 20 100    | Value(20.0), Value(100.0), Total  |
///
/// For times, the meshtal outputs will exclude any `tmesh` bounds entirely if
/// none are given in the input file. In the case that there are no time bins
/// or just one, the group is set to [Group::Total] for consistency.
///
/// | Time bounds   | Groups                            |
/// | ------------- | --------------------------------- |
/// | None          | Total (Nothing in output file)    |
/// | 0.0 1e16      | Total                             |
/// | 0.0 1e16 1e36 | Value(1e16), Value(1e36), Total   |
///
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub enum Group {
    /// The 'Total' bin group
    Total,
    /// The upper edge of a bin
    Value(f64),
}

impl Group {
    #[inline]
    /// Check if the Group is the `Total` variant
    ///
    /// ```rust
    /// # use ntools_mesh::Group;
    /// let group = Group::Total;
    /// assert_eq!(group.is_total(), true);
    ///
    /// let group = Group::Value(2.0);
    /// assert_eq!(group.is_total(), false);
    /// ```
    pub const fn is_total(&self) -> bool {
        matches!(*self, Self::Total)
    }

    #[inline]
    /// Check if the Group is the `Value` variant
    ///
    /// ```rust
    /// # use ntools_mesh::Group;
    /// let group = Group::Total;
    /// assert_eq!(group.is_value(), false);
    ///
    /// let group = Group::Value(2.0);
    /// assert_eq!(group.is_value(), true);
    /// ```
    pub const fn is_value(&self) -> bool {
        !self.is_total()
    }
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{}", value.sci(5, 2)),
            Self::Total => write!(f, "Total"),
        }
    }
}
