mod header;
mod kcode;
mod tally;
mod tmesh;

use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

use log::{trace, warn};

use crate::core::Tally;
use crate::core::Tmesh;
use crate::error::{Error, Result};
use crate::parsers::{data_block, Block};
use crate::Mctal;

/// Internal reader for the MCTAL file
pub(crate) struct Reader {
    mctal: Mctal,
    lines: Lines<BufReader<File>>,
    cached_line: String,
}

// ! Internal API
impl Reader {
    // Advances to the next line, saving it to the cache and returning a ref
    pub(crate) fn next_line(&mut self) -> Result<&str> {
        self.cached_line = self.lines.next().ok_or(Error::EndOfFile)??;
        Ok(self.cached_line.as_str())
    }

    /// Create a new reader for the path provided
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            mctal: Mctal::new(),
            lines: reader.lines(),
            cached_line: String::new(),
        })
    }

    pub(crate) fn last_tally(&self) -> Result<&Tally> {
        Ok(self.mctal.tallies.last().ok_or(Error::NoTallyInitialised)?)
    }

    pub(crate) fn last_tally_mut(&mut self) -> Result<&mut Tally> {
        Ok(self
            .mctal
            .tallies
            .last_mut()
            .ok_or(Error::NoTallyInitialised)?)
    }

    pub(crate) fn last_tmesh(&self) -> Result<&Tmesh> {
        Ok(self.mctal.tmesh.last().ok_or(Error::NoTallyInitialised)?)
    }

    pub(crate) fn last_tmesh_mut(&mut self) -> Result<&mut Tmesh> {
        Ok(self
            .mctal
            .tmesh
            .last_mut()
            .ok_or(Error::NoTallyInitialised)?)
    }

    /// Parse the mcnp mctal file
    pub(crate) fn read(&mut self) -> Result<Mctal> {
        self.next_line()?;

        // check for data blocks, process as appropriate
        while let Ok((_, block)) = data_block(&self.cached_line) {
            // These not necessarily in a fixed order
            match block {
                Block::Header => self.parse_header()?,
                Block::Tally => self.parse_tally()?,
                Block::Tmesh => self.parse_tmesh()?,
                Block::Kcode => self.parse_kcode()?,
                Block::Blank => {
                    // keep going if only blank, or break if EOF
                    if self.next_line().is_err() {
                        break;
                    }
                }
            }
        }

        Ok(std::mem::take(&mut self.mctal))
    }
}
