use log::debug;

use crate::error::Result;
use crate::parsers::*;
use crate::Header;

use super::Reader;

// ! Header block
impl Reader {
    pub(super) fn parse_header(&mut self) -> Result<()> {
        debug!("----------------------");
        debug!(" Parsing Header block ");
        debug!("----------------------");

        // read the first line of the mctal file
        let first = first_line(&self.cached_line)?.1;
        debug!("Code name   = {:?}", first.code_name);
        debug!("Version     = {:?}", first.version);
        debug!("Date        = {:?}", first.problem_id);
        debug!("Dump        = {}", first.dump);
        debug!("NPS         = {}", first.nps);

        // read the message, which must be one line
        let message = self.next_line()?.trim().to_string();
        debug!("Message     = {message:?}");

        // find the number of tallies and potential perturbations
        let (ntal, npert) = ntal_npert(&self.next_line()?)?.1;
        debug!("n pert      = {npert}");
        debug!("n tallies   = {ntal}");

        // either collect the tally numbers or skip to the next line if none
        let mut tally_numbers: Vec<u32> = Vec::with_capacity(ntal as usize);
        if ntal > 0 {
            while let Ok((_i, values)) = vector_of_u32(self.next_line()?) {
                tally_numbers.extend(values.into_iter());
            }
        } else {
            self.next_line()?;
        };
        debug!("Tally ids   = {:?}", tally_numbers);

        // Collate info into a Header struct for convenience
        self.mctal.header = Header {
            code: first.code_name,
            version: first.version,
            date: first.problem_id,
            dump: first.dump,
            n_particles: first.nps,
            n_random: first.random_numbers,
            message,
            n_tallies: ntal,
            n_perturbations: npert,
            tally_numbers,
        };
        debug!("Header read successful");

        Ok(())
    }
}
