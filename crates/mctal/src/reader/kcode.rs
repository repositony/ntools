// internal
use super::Reader;
use crate::error::{Error, Result};
use crate::parsers::*;
use crate::KcodeResult;

// extrenal
use log::{debug, error, trace};

impl Reader {
    pub(super) fn parse_kcode(&mut self) -> Result<()> {
        debug!("---------------------");
        debug!(" Parsing KCODE block ");
        debug!("---------------------");

        // read the kcode header data
        let mut kcode = kcode_header(&self.cached_line)?.1;

        // parse lines for the results
        while let Ok(result) = self.kcode_result() {
            kcode.results.push(result);
        }

        debug!("Results : {}", kcode.results.len());
        trace!("Results : {:#?}", kcode.results);

        // validate the length of kcode results list
        let expected = kcode.recorded_cycles as usize;
        if expected != kcode.results.len() {
            error!("Unexpected number of kcode records");
            Err(Error::UnexpectedLength {
                expected,
                found: kcode.results.len(),
            })
        } else {
            self.mctal.kcode = Some(kcode);
            debug!("Kcode read successful");
            Ok(())
        }
    }

    fn kcode_result(&mut self) -> Result<KcodeResult> {
        // we know there are a maximum of 19 values
        let mut values = Vec::with_capacity(19);

        // todo so basically if this breaks then eof was unexpected, that makes sense
        // 5x values per line, 18-19 values => always take the next 4 lines
        for _i in 0..4 {
            values.extend(vector_of_f64(self.next_line()?)?.1);
        }

        // turn the list into a struct
        KcodeResult::try_from(values.as_ref())
    }
}
