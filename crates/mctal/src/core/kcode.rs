/// KCODE tally data
///
/// The KCODE card specifies the MCNP criticality source that is used for
/// determining keff.
///
/// If a MCTAL file is written during a KCODE problem, the number of recorded
/// cycles, settle cycles, and variables provided are noted.
///
/// All KCODE quantities for each cycle are then listed on as many lines as
/// necessary. These results are stored as a vector of [KcodeResult] for
/// convenience.
#[derive(Debug, Default)]
pub struct Kcode {
    /// Number of cycles recorded
    pub recorded_cycles: u32,
    /// Number of cyles taken to stabilise result
    pub settle_cycles: u32,
    /// Number of user defined variables
    pub variables_provided: u32,
    /// List of Kcode results
    pub results: Vec<KcodeResult>,
}

/// KCODE quantities for a cycle
#[derive(Debug)]
pub struct KcodeResult {
    /// keff collision (col)
    pub collision: f64,
    /// keff absorption (abs)
    pub absorption: f64,
    /// keff track length (trk)
    pub track_length: f64,

    /// Average keff collision
    pub av_collision: f64,
    /// Average keff collision, standard deviation
    pub av_collision_sigma: f64,
    /// Average keff absorption
    pub av_absorption: f64,
    /// Average keff absorption, standard deviation
    pub av_absorption_sigma: f64,
    /// Average keff track length
    pub av_track_length: f64,
    /// Average keff track length, standard deviation
    pub av_track_length_sigma: f64,

    /// Average col/abs/trk keff
    pub av_col_abs_trk: f64,
    /// Average col/abs/trk keff, standard deviation
    pub av_col_abs_trk_sigma: f64,

    /// Average col/abs/trk keff by cycles skipped
    pub av_col_abs_trk_by_cycle: f64,
    /// Average col/abs/trk keff by cycles skipped, standard deviation
    pub av_col_abs_trk_by_cycle_sigma: f64,

    /// Prompt removal lifetime (collision)
    pub lifetime_collision: f64,
    /// Prompt removal lifetime (absorption)
    pub lifetime_absorption: f64,

    /// Prompt removal lifetime (col/abs/trk-len)
    pub av_lifetime: f64,
    /// Prompt removal lifetime (col/abs/trk-len), standard deviation
    pub av_lifetime_sigma: f64,

    /// Number of histories used in the cycle
    pub n_histories: f64,

    /// Figure of merit (when `mct` on `PRDMP` is 1)
    pub fom: f64,
}

impl KcodeResult {
    /// Return Kcode parameters as a vector
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.collision,
            self.absorption,
            self.track_length,
            self.lifetime_collision,
            self.lifetime_absorption,
            self.av_collision,
            self.av_collision_sigma,
            self.av_absorption,
            self.av_absorption_sigma,
            self.av_track_length,
            self.av_track_length_sigma,
            self.av_col_abs_trk,
            self.av_col_abs_trk_sigma,
            self.av_col_abs_trk_by_cycle,
            self.av_col_abs_trk_by_cycle_sigma,
            self.av_lifetime,
            self.av_lifetime_sigma,
            self.n_histories,
            self.fom,
        ]
    }
}

impl<T> TryFrom<&[T]> for KcodeResult
where
    T: Into<f64> + Copy,
{
    type Error = crate::error::Error;

    /// Initialise a [KcodeResult] from a list of numbers
    ///
    /// This will fail on any list not 18-19 values long.
    fn try_from(values: &[T]) -> Result<Self, Self::Error> {
        {
            // make sure the array provided is the right length
            if !(values.len() == 18 || values.len() == 19) {
                return Err(Self::Error::UnexpectedNumberOfKcodeValues {
                    expected: "18-19".to_string(),
                    found: values.len(),
                });
            }

            // verbose but no need for anything fancy here
            Ok(KcodeResult {
                collision: values[0].into(),
                absorption: values[1].into(),
                track_length: values[2].into(),
                lifetime_collision: values[3].into(),
                lifetime_absorption: values[4].into(),
                av_collision: values[5].into(),
                av_collision_sigma: values[6].into(),
                av_absorption: values[7].into(),
                av_absorption_sigma: values[8].into(),
                av_track_length: values[9].into(),
                av_track_length_sigma: values[10].into(),
                av_col_abs_trk: values[11].into(),
                av_col_abs_trk_sigma: values[12].into(),
                av_col_abs_trk_by_cycle: values[13].into(),
                av_col_abs_trk_by_cycle_sigma: values[14].into(),
                av_lifetime: values[15].into(),
                av_lifetime_sigma: values[16].into(),
                n_histories: values[17].into(),

                // col/abs/trk-len keff. figure of merit only printed if the mct option on the PRDMP card is equal to 1
                fom: if values.len() > 18 {
                    values[18].into()
                } else {
                    0.0
                },
            })
        }
    }
}
