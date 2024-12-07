/// Mctal file header information
///
/// This is basic metadata for run information and data blocks contained within
/// the file.
///
/// For example, take the following header:
///
/// ```text
/// mcnp6.mp   6     10/14/1066 22:31:17     2         1296681       281135206
///  Four uranium cans in air and aluminum
/// ntal     2
///     1    4
/// ```
///
/// This would parse to the [Header] as:
///
/// ```json
/// Header {
///     code: "mcnp6.mp",
///     version: "6",
///     date: "10/14/1066 22:31:17",
///     message: "Four uranium cans in air and aluminum",
///     dump: 2,
///     n_particles: 1296681,
///     n_perturbations: 0,
///     n_random: 281135206,
///     n_tallies: 2,
///     tally_numbers: [
///         1,
///         4,
///     ]
/// }
/// ```
#[derive(Debug, Default)]
pub struct Header {
    /// Name of the code, e.g. "MCNP6"
    pub code: String,
    /// Code version, e.g. "6.3"
    pub version: String,
    /// Date and time run, and host designator if available
    pub date: String,
    /// Dump number
    pub dump: u32,
    /// Message from the input deck
    pub message: String,
    /// Number of particle histories
    pub n_particles: u64,
    /// Number of pseudo-random numbers
    pub n_random: u64,
    /// Number of perturbations in the problem
    pub n_perturbations: u32,
    /// Number of tallies in the problem
    pub n_tallies: u32,
    /// List of tally identifiers
    pub tally_numbers: Vec<u32>,
}

impl Header {
    /// Check if a tally exists in the file results
    pub fn tally_exists(self, id: u32) -> bool {
        self.tally_numbers.contains(&id)
    }
}
