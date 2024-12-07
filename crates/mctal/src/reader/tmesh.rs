use log::{debug, error, trace, warn};

use crate::error::{Error, Result};
use crate::{parsers::*, Geometry};
use crate::{Particle, Tmesh};

use super::Reader;

// ! TMESH block
impl Reader {
    pub(super) fn parse_tmesh(&mut self) -> Result<()> {
        debug!("---------------------");
        debug!(" Parsing TMESH block ");
        debug!("---------------------");

        // append a new empty tmesh to work with
        self.mctal.tmesh.push(Tmesh::default());

        // read mesh data into the new tmesh
        self.tmesh_header()?;
        self.tmesh_dimensions()?;
        self.tmesh_bins()?;
        self.tmesh_results()?;

        Ok(())
    }

    fn tmesh_header(&mut self) -> Result<()> {
        // read the tmesh header data
        // NOTE: always negative to mark as tmesh, unlike tally
        let tmesh_header = tmesh_header(&self.cached_line)?.1;
        debug!("Tmesh id    = {}", tmesh_header.id);
        debug!("Geometry    = {:?}", tmesh_header.geometry);

        // read the particles, always in a list unlike standard tallies
        // NOTE: not provided a number of particles to validate against
        let (_, values) = vector_of_u32(self.next_line()?)?;
        let particles = values
            .into_iter()
            .enumerate()
            .filter(|(_, v)| *v > 0)
            .map(|(i, _v)| Particle::from_id((i as u8) + 1))
            .collect();
        debug!("Particles   = {:?}", particles);

        // assign everything to the most recent tmesh
        let tmesh = self.last_tmesh_mut()?;
        tmesh.id = tmesh_header.id;
        tmesh.geometry = tmesh_header.geometry;
        tmesh.particles = particles;

        Ok(())
    }

    fn tmesh_dimensions(&mut self) -> Result<()> {
        // read the mesh bound totals
        let coords = tmesh_coordinates(self.next_line()?)?.1;
        debug!("CORA bins   = {}", coords.n_cora);
        debug!("CORB bins   = {}", coords.n_corb);
        debug!("CORC bins   = {}", coords.n_corc);

        // parse all three bound sets for long as relevant
        let n = coords.n_cora + coords.n_corb + coords.n_corc + 3;
        let mut bounds: Vec<f64> = Vec::with_capacity(n);
        while let Ok((_, values)) = vector_of_f64(self.next_line()?) {
            bounds.extend(values.into_iter());
        }

        // validate that the number of tmesh bounds found is expected
        if n != bounds.len() {
            return Err(Error::UnexpectedNumberOfTmeshBounds {
                expected: n,
                found: bounds.len(),
            });
        }

        // slice up the continuum of bounds into the three bounds
        // todo: figure out a cleaner way, indexing directly is always dumb
        let mut lower = 0;
        let mut upper = coords.n_cora + 1;
        let cora = bounds[lower..upper].to_vec();
        lower += upper;
        upper += coords.n_corb + 1;
        let corb = bounds[lower..upper].to_vec();
        let corc = bounds[upper..].to_vec();

        // validate the length of final bound lists
        if coords.n_cora + 1 != cora.len() {
            error!("Unexpected number of CORA values");
            return Err(Error::UnexpectedLength {
                expected: coords.n_cora,
                found: cora.len(),
            });
        }
        if coords.n_corb + 1 != corb.len() {
            error!("Unexpected number of CORB values");
            return Err(Error::UnexpectedLength {
                expected: coords.n_corb,
                found: corb.len(),
            });
        }
        if coords.n_corc + 1 != corc.len() {
            error!("Unexpected number of CORC values");
            return Err(Error::UnexpectedLength {
                expected: coords.n_corc,
                found: corc.len(),
            });
        }

        trace!("CORA bins   = {:?}", &cora);
        trace!("CORB bins   = {:?}", &corb);
        trace!("CORC bins   = {:?}", &corc);

        // assign everything to the most recent mesh
        let tmesh = self.last_tmesh_mut()?;
        tmesh.n_voxels = coords.n_voxels;
        tmesh.n_cora = coords.n_cora;
        tmesh.n_corb = coords.n_corb;
        tmesh.n_corc = coords.n_corc;
        tmesh.cora = cora;
        tmesh.corb = corb;
        tmesh.corc = corc;

        Ok(())
    }

    fn tmesh_bins(&mut self) -> Result<()> {
        let n_flagged_bins = basic_bin(&self.cached_line, 'd')?.1;
        debug!("Flagged [f] = {n_flagged_bins}");

        let n_user_bins = basic_bin(self.next_line()?, 'u')?.1;
        debug!("User    [u] = {n_user_bins}");

        let n_segment_bins = basic_bin(self.next_line()?, 's')?.1;
        debug!("Segment [s] = {n_segment_bins}");

        let n_multiplier_bins = basic_bin(self.next_line()?, 'm')?.1;
        debug!("Mult    [m] = {n_multiplier_bins}");

        let n_cosine_bins = basic_bin(self.next_line()?, 'c')?.1;
        debug!("Cosine  [c] = {n_cosine_bins}");

        let n_energy_bins = basic_bin(self.next_line()?, 'e')?.1;
        debug!("Energy  [e] = {n_energy_bins}");

        let n_time_bins = basic_bin(self.next_line()?, 't')?.1;
        debug!("Time    [t] = {n_time_bins}");

        // assign everything to the most recent mesh
        let tmesh = self.last_tmesh_mut()?;
        tmesh.n_flagged_bins = n_flagged_bins;
        tmesh.n_user_bins = n_user_bins;
        tmesh.n_segment_bins = n_segment_bins;
        tmesh.n_multiplier_bins = n_multiplier_bins;
        tmesh.n_cosine_bins = n_cosine_bins;
        tmesh.n_energy_bins = n_energy_bins;
        tmesh.n_time_bins = n_time_bins;

        Ok(())
    }

    fn tmesh_results(&mut self) -> Result<()> {
        // make sure this is the results section
        let vals = self.next_line()?;
        if !is_vals(vals) {
            return Err(Error::UnexpectedKeyword {
                expected: "vals".into(),
                found: vals.into(),
            });
        }

        // read voxel results for as long as relevant
        let n = self.last_tmesh()?.n_expected_results();
        let mut results = Vec::with_capacity(n);

        // this is the last thing in the block, we need to finish saving results
        // if it is EOF
        while let Ok(i) = self.next_line() {
            if let Ok((_, values)) = vector_of_tally_results(i) {
                results.extend(values.into_iter());
            } else {
                break;
            }
        }
        debug!("Results     = {}", results.len());

        // validate the length of parsed list
        if n != results.len() {
            error!("Unexpected number of voxels");
            return Err(Error::UnexpectedLength {
                expected: n,
                found: results.len(),
            });
        }
        trace!("Results      = {:?}", results);

        // assign everything to the most recent mesh
        let tmesh = self.last_tmesh_mut()?;
        tmesh.results = results;

        Ok(())
    }
}
