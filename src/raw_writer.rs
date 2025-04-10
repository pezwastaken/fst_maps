use std::fs::File;

use std::io::{self, BufWriter};

use bincode::error::EncodeError;

use crate::fst_state::FstState;

#[derive(Debug)]
pub struct RawWriter<'a> {
    inner_writer: BufWriter<File>,
    to_file: &'a str,
    wrote_bytes: usize,
}

impl<'a> RawWriter<'a> {
    pub fn new(f: &'a str) -> Result<RawWriter<'a>, io::Error> {
        let out = BufWriter::new(File::create(f)?);

        Ok(Self {
            inner_writer: out,
            to_file: f,
            wrote_bytes: 0,
        })
    }

    //TODO?
    //this could be a generic state (maybe)
    pub fn write_state(&mut self, state: &FstState) -> Result<(), EncodeError> {
        let wrote = bincode::encode_into_std_write(
            &state,
            &mut self.inner_writer,
            bincode::config::standard(),
        );
        self.wrote_bytes += wrote?;
        Ok(())
    }
}
