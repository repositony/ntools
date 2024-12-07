use crate::Particle;

/// Standard tally type data
///
/// Contains the results for any standard `F` tally.
///
/// Tally results are stored as a vector of [TallyResult], which contains a
/// value and corresponding relative error.
///
/// The tally fluctuation chart over all dump cycles is stored as a
/// [TallyFluctuation].  
///
/// ### Supported tallies
///
/// | Mnemonic | Tally Description                  | Fn units      | *Fn units |
/// | -------- | ---------------------------------- | ------------- | --------- |
/// | F1:P     | Integrated surface current         | particles     | MeV       |
/// | F2:P     | Surface flux                       | particles/cm2 | MeV/cm2   |
/// | F4:P     | Cell flux                          | particles/cm2 | MeV/cm2   |
/// | F5a:P    | Flux at a point or ring detector   | particles/cm2 | MeV/cm2   |
/// | FIP5:P   | Point detectors, pinhole           | particles/cm2 | MeV/cm2   |
/// | FIR5:P   | Point detectors, planar radiograph | particles/cm2 | MeV/cm2   |
/// | FIC5:P   | Point detectors, cyl radiograph    | particles/cm2 | MeV/cm2   |
/// | F6:P     | Cell energy deposition             | MeV/g         | jerks/g   |
/// | +F6      | Collision heating                  | MeV/g         | N/A       |
/// | F7:P     | Cell fission energy deposition     | MeV/g         | jerks/g   |
/// | F8:P     | Pulse height tally                 | pulses        | MeV       |
/// | +F8:P    | Charge deposition                  | charge        |  N/A      |
///
#[derive(Debug, Default)]
pub struct Tally {
    /// Tally number
    pub id: u32,
    /// Particle type
    pub particles: Vec<Particle>,
    /// Tally type (pinhole, ring, detector, etc...)
    pub kind: TallyKind,
    /// Tally modifier (none, *, +)
    pub modifier: Modifier,
    /// Tally comment
    pub comment: String,

    /// List of cell, surface, or detector numbers
    pub region_bins: BinData,
    /// Number of flagged bins (total vs. direct or flagged vs. unflagged)
    pub flagged_bins: BinData,
    /// Number of user bins
    pub user_bins: BinData,
    /// Number of multiplier bins
    pub multiplier_bins: BinData,
    /// List of bin bounds for a radiograph tally
    pub segment_bins: BinData,
    /// List of the cosine values
    pub cosine_bins: BinData,
    /// List of the energy values (MeV)
    pub energy_bins: BinData,
    /// List of the time values (shakes)
    pub time_bins: BinData,

    /// All tally values read directly
    pub results: Vec<TallyResult>,

    /// Tally fluctuation chart data
    pub tfc: Tfc,
}

impl Tally {
    /// Iterator over results by region (cell, surface, detector)
    pub fn iter(&self) -> TallyIterator {
        TallyIterator {
            tally: self,
            index: 0,
        }
    }

    /// Calculate expected number of results from MCTAl bin records
    pub fn n_expected_results(&self) -> usize {
        let values = [
            self.region_bins.number,
            self.flagged_bins.number,
            self.user_bins.number,
            self.segment_bins.number,
            self.multiplier_bins.number,
            self.cosine_bins.number,
            self.energy_bins.number,
            self.time_bins.number,
        ];
        // 0=unbounded but should be considered 1 bin
        values.iter().filter(|v| **v > 0).product()
    }

    /// Try to find results for a region (cell, surface, detector)
    pub fn find_result<T>(&self, region: T) -> Option<&[TallyResult]>
    where
        T: Into<f64> + Copy,
    {
        // will take the first match if there are a load of `0`s
        if let Some(idx) = self
            .region_bins
            .values
            .iter()
            .position(|&r| r == region.into())
        {
            return self.iter().nth(idx);
        }
        None
    }
}

// #[doc(hidden)]
pub struct TallyIterator<'a> {
    tally: &'a Tally,
    index: usize,
}

// ps: it could be done using .get() as well, however I wanted to use this approach to make it a bit more clear how Options are returned in this context
impl<'a> Iterator for TallyIterator<'a> {
    type Item = &'a [TallyResult];

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.tally.results.len() / self.tally.region_bins.number;
        if self.index < self.tally.region_bins.number {
            let result = self.tally.results.chunks(n).nth(self.index);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

/// Tally result containing a value and relative error
///
/// All tally results are output as `<value>` `<error>` pairs. Each pair is
/// stored as a [TallyResult] to avoid maintiaing multiple arrays and indexing
/// both to retrieve data that should be linked.
///
/// For example:
///
/// ```text
///  vals
///     1.00000E+00 0.1230  2.00000E+00 0.4560  3.00000E+00 0.7890 ...
/// ```
///
/// ```json
/// [
///     TallyResult {
///         value: 1.00000E+00,
///         error: 0.1230,
///     },
///     TallyResult {
///         value: 2.00000E+00,
///         error: 0.4560,
///     },
///     TallyResult {
///         value: 3.00000E+00,
///         error: 0.7890,
///     },
///     ...
/// ]
/// ```
#[derive(Debug, Default)]
pub struct TallyResult {
    /// Tally result value
    pub value: f64,
    /// Relative uncertainty
    pub error: f64,
}

// todo: implement operators for results (see voxel logic in mesh crate)
impl TallyResult {
    /// Absolute error on the result
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::TallyResult;
    /// let result = TallyResult {
    ///     value: 50.0,
    ///     error: 0.10,
    /// };
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(result.absolute_error(), 5.0);
    /// ```
    pub fn absolute_error(&self) -> f64 {
        self.value * self.error
    }

    /// Relative error on the result
    ///
    /// The MCNP tally outputs are provided and stored as the relative
    /// uncertainty anyway. However, having both
    /// [absolute_error()](TallyResult::absolute_error) and
    /// [relative_error()](TallyResult::relative_error) methods makes intent
    /// explicit.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::TallyResult;
    /// let result = TallyResult {
    ///     value: 50.0,
    ///     error: 0.10,
    /// };
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(result.relative_error(), 0.1);
    /// ```
    pub fn relative_error(&self) -> f64 {
        self.error
    }
}

/// Types of detector tally
///
/// The [Tally] can be any type of standard `F` tally, including detector
/// tallies.
///
/// A tally header contains `TALLY <id> <i> <j> <k>`, where `j` is the type of
/// tally.
///
/// This type is stored as a [TallyKind] with enumeration explicitly set to
/// match MCTAL identifiers.
///
/// | Value | Description of type    |
/// | ----- | ---------------------- |
/// | 0     | None                   |
/// | 1     | Point                  |
/// | 2     | Ring                   |
/// | 3     | Pinhole                |
/// | 4     | TransmittedRectangular |
/// | 5     | TransmittedCylindrical |
///
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TallyKind {
    #[default]
    /// None, generic tally type
    None = 0,
    /// Point detector tally
    Point = 1,
    /// Ring detector tally
    Ring = 2,
    /// Pinhole radiograph (FIP)
    Pinhole = 3,
    /// Transmitted image radiograph (rectangular grid, FIR),
    TransmittedRectangular = 4,
    /// Transmitted image radiograph (cylindrical grid, FIC)
    TransmittedCylindrical = 5,
}

/// Tally modifier
///
/// The [Tally] can be any type of standard `F` tally, and may have a modifier
/// on the input card such as `*F...` or `+F...`.
///
/// A tally header contains `TALLY <id> <i> <j> <k>`, where `k` is modifier
/// applied.
///
/// This type is stored as a [Modifier] with enumeration explicitly set to match
/// MCTAL identifiers.
///
/// | Value | Modifier |
/// | ----- | -------- |
/// | 0     | None     |
/// | 1     | Star     |
/// | 2     | Plus     |
///
/// This may change the interpretation of result units. See [Tally] for details.
///
#[derive(Debug, Default, PartialEq)]
pub enum Modifier {
    #[default]
    // No tally modifier, e.g. F4:n
    None = 0,
    // For the `*` tally modifier, e.g. *F4:n
    Star = 1,
    // For the `+` tally modifier, e.g. +F4:n
    Plus = 2,
}

/// Struct containing all relevant bin data
///
/// Bin information is stored under a series of character identifiers.
///
/// For example:
///
/// ```text
/// ct      11
///  -8.00000E-01 -6.00000E-01 -4.00000E-01 -2.00000E-01 -5.55112E-17  2.00000E-01
///   4.00000E-01  6.00000E-01  8.00000E-01  1.00000E+00
/// ```
///
/// will parse the cosine bins into
///
/// ```rust, ignore
/// BinData {
///     token: 'c',
///     number: 11,
///     kind: BinKind::Total,
///     flag: BinFlag::UpperBound,
///     unbound: false,
///     values: [-8.00000E-01, -6.00000E-01, -4.00000E-01...],
/// }
/// ```
///
/// A quick-reference for what the parser expects is shown in the table below.
///
/// | Token | Description     | Kind    | Flag    | Unbound | List of bin values        |
/// | ----- | --------------- | ------- | ------- | ------- | ------------------------  |
/// | `f`   | regions*        | &cross; | &check; | &cross; | &check; (unless detector) |
/// | `d`   | flagged bins    | &cross; | &check; | &cross; | &cross;                   |
/// | `u`   | user bins       | &check; | &check; | &cross; | &check; (if special)      |
/// | `s`   | segments        | &check; | &check; | &cross; | &check; (if segment)      |
/// | `m`   | multiplier bins | &check; | &check; | &cross; | &cross;                   |
/// | `c`   | cosine bins     | &check; | &check; | &check; | &check;                   |
/// | `e`   | energy bins     | &check; | &check; | &check; | &check;                   |
/// | `t`   | time bins       | &check; | &check; | &check; | &check;                   |
///
/// (*) Regions are cell, surface, or detector bins depending on tally type
///
#[derive(Debug, Default)]
pub struct BinData {
    pub token: char,
    pub number: usize,
    pub kind: BinKind,
    pub flag: BinFlag,
    pub unbound: bool,
    pub values: Vec<f64>,
}

/// Standard bin modifiers (i.e. `None`, `Total`, `Cumulative`)
///
/// Some tokens are followed by values under certain conditions, and may also
/// have multiple character tags to guide the interpretation of listed values.
///
/// For example, on the energy bins:
///
/// | Tag example | Description                                 |
/// | ----------- | ------------------------------------------- |
/// | `e`         | one unbounded energy bin (n=0 instead of 1) |
/// | `et`        | includes a "`t`otal" bin                    |
/// | `ec`        | energy bins are `c`umulative                |
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum BinKind {
    #[default]
    None,
    Total,
    Cumulative,
}

impl From<Option<char>> for BinKind {
    fn from(tag: Option<char>) -> Self {
        match tag {
            Some('t') => BinKind::Total,
            Some('c') => BinKind::Cumulative,
            _ => BinKind::None,
        }
    }
}

/// User flag for interpretation (i.e. `Upper bounds`, `Discrete points`)
///
/// While MCNP does not print this, flags are allowed in user generated mctal
/// files.
///
///  - When the flag is `0` or `none`, values are upper bin boundaries
///  - Otherwise, values are points where the tally values should be plotted
///
/// For example:
///
/// | Flag example | Description                                   |
/// | ------------ | --------------------------------------------- |
/// | `c 10`       | 10 cosine bins, no flag, assumed upper bounds |
/// | `c 10 0`     | 10 cosine bins, upper bounds                  |
/// | `c 10 1`     | 10 cosine bins, discrete points               |
///
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum BinFlag {
    #[default]
    UpperBound,
    Discrete,
}

impl From<Option<usize>> for BinFlag {
    fn from(flag: Option<usize>) -> Self {
        if let Some(f) = flag {
            if f > 0 {
                return BinFlag::Discrete;
            }
        }
        BinFlag::UpperBound
    }
}

/// Tally fluctuation chart
///
/// The entire tally fluctuation chart is recorded. The first line contains the
/// number of bins and metadata.
///
/// `TFC <n> <jtf>`
///
/// - `n` is the number of sets of tally fluctuation data
/// - `jtf` is a list of 8 numbers, indices of the tally fluctuation chart bins
///
/// These are recorded into the [TallyFluctuation] struct for convenience.
///
/// For example:
///
/// ```text
/// tfc    10       4       1       1      15       1       1       100       1
/// ```
///
/// ```rust, ignore
/// TallyFluctuation {
///     n_records: 10,
///     region_bins: 4,
///     flagged_bins: 1,
///     user_bins: 1,
///     segment_bins: 15,
///     multiplier_bins: 1,
///     cosine_bins: 1,
///     energy_bins: 100,
///     time_bins: 1,
///     results: [...]
/// }
/// ```
///
/// In the MCTAL file, records for the table follow on as many lines as needed
/// in the format: `<nps> <mean> <error> <fom>`.
///
/// These are stored as [TfcResult] records under [TallyFluctuation] `results`.
///
#[derive(Debug, Default)]
pub struct Tfc {
    /// Number of sets of tally fluctuation data
    pub n_records: u32,
    /// Number of flagged bins (total vs. direct or flagged vs. unflagged)
    pub n_flagged_bins: u32,
    /// Number of regions (i.e. cells or surfaces)
    pub n_region_bins: u32,
    /// Number of user bins
    pub n_user_bins: u32,
    /// Number of segments
    pub n_segment_bins: u32,
    /// Number of multiplier bins
    pub n_multiplier_bins: u32,
    /// Number of cosine bins
    pub n_cosine_bins: u32,
    /// Number of energy bins
    pub n_energy_bins: u32,
    /// Number of time bins
    pub n_time_bins: u32,

    /// Tally fluctuation data. i.e. nps, mean, error, figure of merit.
    pub results: Vec<TfcResult>,
}

/// Tally fluctuation chart results
///
/// In the MCTAL file, records for the table follow on as many lines as needed.
///
/// `<nps> <mean> <error> <fom>`
///
/// - `nps` are the number of particle histories run
/// - `mean` is the tally mean
/// - `error` is the relative error  
/// - `fom` is the tally Figure of Merit  
///
/// These are stored in a vector under [TallyFluctuation] `results`.
#[derive(Debug, Default)]
pub struct TfcResult {
    /// Number of particles
    pub nps: u64,
    /// Average value
    pub value: f64,
    /// Relative uncertainty
    pub error: f64,
    /// Figure-of-Merit value
    pub fom: f64,
}

impl TfcResult {
    /// Absolute error on the result
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::TfcResult;
    /// let result = TfcResult {
    ///     value: 50.0,
    ///     error: 0.10,
    ///     ..Default::default()
    /// };
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(result.absolute_error(), 5.0);
    /// ```
    pub fn absolute_error(&self) -> f64 {
        self.value * self.error
    }

    /// Relative error on the result
    ///
    /// The MCNP tally outputs are provided and stored as the relative
    /// uncertainty anyway. However, having both
    /// [absolute_error()](TfcResult::absolute_error) and
    /// [relative_error()](TfcResult::relative_error) methods makes intent
    /// explicit.
    ///
    /// For example:
    ///
    /// ```rust
    /// # use ntools_mctal::TfcResult;
    /// let result = TfcResult {
    ///     value: 50.0,
    ///     error: 0.10,
    ///     ..Default::default()
    /// };
    ///
    /// /// 10% relative error => 50.0 +/-5.0
    /// assert_eq!(result.relative_error(), 0.1);
    /// assert_eq!(result.absolute_error(), 5.0);
    /// ```
    pub fn relative_error(&self) -> f64 {
        self.error
    }
}
