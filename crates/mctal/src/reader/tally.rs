use log::{debug, error, trace, warn};

use crate::core::{BinData, BinFlag, BinKind, Particle, Tally, TallyKind, TallyResult};
use crate::error::{Error, Result};
use crate::parsers::*;

use super::Reader;

// ! Tally block
impl Reader {
    pub(super) fn parse_tally(&mut self) -> Result<()> {
        debug!("---------------------");
        debug!(" Parsing Tally block ");
        debug!("---------------------");

        // append a new empty tally to work with
        self.mctal.tallies.push(Tally::default());

        // read tally data into the new tally
        self.tally_header()?;
        self.tally_bins()?;
        self.tally_results()?;
        self.tally_tfc()?;

        Ok(())
    }

    fn tally_header(&mut self) -> Result<()> {
        // read the tally header data
        let tally_header = tally_header(&self.cached_line)?.1;
        debug!("Tally id    = {:?}", tally_header.id);
        debug!("Type        = {:?}", tally_header.kind);
        debug!("Modifier    = {:?}", tally_header.modifier);

        // read the particles from either predefined values or a list
        let particles = if tally_header.particle_flag.is_negative() {
            // parse bin value lines for as long as relevant
            let (_, values) = vector_of_u32(&self.next_line()?)?;
            values
                .into_iter()
                .enumerate()
                .filter(|(_, v)| *v > 0)
                .map(|(i, _v)| Particle::from_id((i as u8) + 1))
                .collect()
        } else {
            // convert the predefined values to particles
            match tally_header.particle_flag {
                1 => vec![Particle::Neutron],
                2 => vec![Particle::Photon],
                3 => vec![Particle::Neutron, Particle::Photon],
                4 => vec![Particle::Electron],
                5 => vec![Particle::Neutron, Particle::Electron],
                6 => vec![Particle::Photon, Particle::Photon],
                _ => vec![Particle::Unknown],
            }
        };
        debug!("Particles   = {:?}", particles);

        // validate that the number of particles found is expected
        let expected = tally_header.particle_flag.unsigned_abs() as usize;
        if expected != particles.len() {
            return Err(Error::UnexpectedLength {
                expected,
                found: particles.len(),
            });
        }

        // read comments, which can be over multiple lines or nothing
        let mut comment = String::new();
        while !self.next_line()?.starts_with("f") {
            comment.push_str(self.cached_line.trim());
            comment.push_str(" ");
        }
        debug!("Comment     = \"{}\"", comment);

        // assign everything to the most recent tally
        let tally = self.last_tally_mut()?;
        tally.id = tally_header.id;
        tally.kind = tally_header.kind;
        tally.modifier = tally_header.modifier;
        tally.particles = particles;
        tally.comment = comment;

        Ok(())
    }

    fn tally_bins(&mut self) -> Result<()> {
        // grab all the bin data in turn
        let regions = self.regions()?;
        let flagged = self.flagged()?;
        let user = self.user()?;
        let segment = self.segment()?;
        let multiplier = self.multiplier()?;
        let cosine = self.cosine()?;
        let energy = self.energy()?;
        let time = self.time()?;

        // assign everything to the most recent tally
        let tally = self.last_tally_mut()?;
        tally.region_bins = regions;
        tally.flagged_bins = flagged;
        tally.user_bins = user;
        tally.segment_bins = segment;
        tally.multiplier_bins = multiplier;
        tally.cosine_bins = cosine;
        tally.energy_bins = energy;
        tally.time_bins = time;

        Ok(())
    }

    fn regions(&mut self) -> Result<BinData> {
        // read user bin data
        let mut bins = bin_data(&self.cached_line, 'f')?.1;

        // parse bin value lines for as long as relevant
        // NOTE: detector tallies do not print a list
        let tally_kind = self.last_tally()?.kind;
        if tally_kind != TallyKind::None {
            self.next_line()?;
        } else {
            while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
                bins.values.extend(values.into_iter());
            }
        }

        // validate the length of parsed list
        let expected = match tally_kind {
            TallyKind::None => bins.number,
            _ => 0,
        };
        let found = bins.values.len();

        if found != expected {
            error!("Unexpected number of region bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("Regions [f] = {:?}", bins.number);
        trace!("Regions [f] = {:?}", bins.values);

        Ok(bins)
    }

    fn flagged(&mut self) -> Result<BinData> {
        // read flagged bin data, supposedly no list will follow
        let bins = bin_data(&self.cached_line, 'd')?.1;
        debug!("Flagged [d] = {:?}", bins.number);
        Ok(bins)
    }

    fn user(&mut self) -> Result<BinData> {
        // read user bin data
        let mut bins = bin_data(&self.next_line()?, 'u')?.1;

        // parse bin value lines for as long as relevant
        // NOTE: contrary to the manuals, this can print bin values
        while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
            bins.values.extend(values.into_iter());
        }

        // validate the length of parsed list
        let expected = match bins.kind {
            BinKind::Total => bins.number - 1,
            _ => bins.number,
        };
        let found = bins.values.len();

        if found == 0 && expected != found {
            let id = self.last_tally()?.id;
            warn!("No user [u] bins listed for Tally {id} (expected {expected})");
        } else if expected != found {
            error!("Unexpected number of region bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("User    [u] = {:?}", bins.number);
        trace!("User    [u] = {:?}", bins.values);
        Ok(bins)
    }

    fn segment(&mut self) -> Result<BinData> {
        // read segment bin data
        let mut bins = bin_data(&self.cached_line, 's')?.1;

        // parse bin value lines for as long as relevant
        while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
            bins.values.extend(values.into_iter());
        }

        // validate the length of parsed list
        let found = bins.values.len();
        let mut expected = bins.number;
        if bins.kind == BinKind::Total {
            expected -= 1
        };
        if found != 0 && self.last_tally()?.kind != TallyKind::None {
            expected += 1
        }

        if found == 0 && expected != found {
            let id = self.last_tally()?.id;
            warn!("No segment [s] bins listed for Tally {id} (expected {expected})");
        } else if expected != found {
            error!("Unexpected number of segment bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("Segment [s] = {:?}", bins.number);
        trace!("Segment [s] = {:?}", bins.values);
        Ok(bins)
    }

    fn multiplier(&mut self) -> Result<BinData> {
        // read multiplier bin data, supposedly no list will follow
        let bins = bin_data(&self.cached_line, 'm')?.1;
        debug!("Mult    [m] = {:?}", bins.number);
        trace!("Mult    [m] = {:?}", bins.values);
        Ok(bins)
    }

    fn cosine(&mut self) -> Result<BinData> {
        // read cosine bin data
        let mut bins = bin_data(&self.next_line()?, 'c')?.1;

        // parse bin value lines for as long as relevant
        while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
            bins.values.extend(values.into_iter());
        }

        // validate the length of parsed list
        let found = bins.values.len();
        let mut expected = bins.number;
        if bins.kind == BinKind::Total {
            expected -= 1
        };
        if found != 0 && self.last_tally()?.kind != TallyKind::None {
            expected += 1
        }
        if found != expected {
            error!("Unexpected number of cosine bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("Cosine  [c] = {:?} ({:?})", bins.number, bins.kind);
        trace!("Cosine  [c] = {:?}", bins.values);
        Ok(bins)
    }

    fn energy(&mut self) -> Result<BinData> {
        // read energy bin data
        let mut bins = bin_data(&self.cached_line, 'e')?.1;

        // parse bin value lines for as long as relevant
        while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
            bins.values.extend(values.into_iter());
        }

        // validate the length of parsed list
        let expected = match bins.kind {
            BinKind::Total => bins.number - 1,
            _ => bins.number,
        };
        let found = bins.values.len();

        if expected != found {
            error!("Unexpected number of energy bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("Energy  [e] = {:?} ({:?})", bins.number, bins.kind);
        trace!("Energy  [e] = {:?}", bins.values);
        Ok(bins)
    }

    fn time(&mut self) -> Result<BinData> {
        // read time bin data
        let mut bins = bin_data(&self.cached_line, 't')?.1;

        // parse bin value lines for as long as relevant
        while let Ok((_i, values)) = vector_of_f64(self.next_line()?) {
            bins.values.extend(values.into_iter());
        }

        // validate the length of parsed list
        let expected = match bins.kind {
            BinKind::Total => bins.number - 1,
            _ => bins.number,
        };
        let found = bins.values.len();
        if expected != found {
            error!("Unexpected number of time bins");
            return Err(Error::UnexpectedLength { expected, found });
        }

        debug!("Time    [t] = {:?} ({:?})", bins.number, bins.kind);
        trace!("Time    [t] = {:?}", bins.values);
        Ok(bins)
    }

    fn tally_results(&mut self) -> Result<()> {
        // make sure this is the results section
        if !is_vals(&self.cached_line) {
            return Err(Error::UnexpectedKeyword {
                expected: "vals".into(),
                found: self.cached_line.clone(),
            });
        }

        // get next lines for as long as relevant
        let n = self.last_tally()?.n_expected_results();
        let mut results = Vec::with_capacity(n);
        while let Ok((_i, values)) = vector_of_tally_results(self.next_line()?) {
            results.extend(values.into_iter());
        }
        debug!("N results   = {}", results.len());
        trace!("Results     = {:?}", results);

        // validate the length of parsed list
        if n != results.len() {
            error!("Unexpected number of results");
            return Err(Error::UnexpectedLength {
                expected: n,
                found: results.len(),
            });
        }

        // assign everything to the most recent tally
        let tally = self.last_tally_mut()?;
        tally.results = results;

        Ok(())
    }

    fn tally_tfc(&mut self) -> Result<()> {
        // get the header info
        let mut tfc = tfc(&self.cached_line)?.1;

        // this is the last thing in the block, we need to finish saving results
        // if it is EOF
        while let Ok(i) = self.next_line() {
            if let Ok((_, value)) = tfc_result(i) {
                tfc.results.push(value);
            } else {
                break;
            }
        }

        debug!("N Tfc       = {}", tfc.results.len());
        trace!("Tfc results = {:?}", tfc.results);

        // validate the length of parsed list
        let expected = tfc.n_records as usize;
        let found = tfc.results.len();
        if expected != found {
            error!("Unexpected number of Tally Fluctuation results");
            return Err(Error::UnexpectedLength { expected, found });
        }

        // assign everything to the most recent tally
        let tally = self.last_tally_mut()?;
        tally.tfc = tfc;

        Ok(())
    }
}
