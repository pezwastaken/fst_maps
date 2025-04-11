use std::fs::File;

use std::io::{self, BufWriter};

use bincode::error::EncodeError;

use crate::fst_state::FstState;

#[derive(Debug)]
pub struct RawWriter<'a> {
    inner_writer: BufWriter<File>,
    to_file: &'a str,
    written_bytes: usize,
}

impl<'a> RawWriter<'a> {
    pub fn new(f: &'a str) -> Result<RawWriter<'a>, io::Error> {
        let out = BufWriter::new(File::create(f)?);

        Ok(Self {
            inner_writer: out,
            to_file: f,
            written_bytes: 0,
        })
    }

    pub fn get_written_bytes(&self) -> usize {
        self.written_bytes
    }

    //TODO?
    //this could be a generic state (maybe)
    pub fn write_state(&mut self, state: &FstState) -> Result<usize, EncodeError> {
        let wrote = bincode::encode_into_std_write(
            &state,
            &mut self.inner_writer,
            bincode::config::standard(),
        );
        if let Ok(b) = wrote {
            self.written_bytes += b;
            return Ok(b);
        }
        wrote
    }

    ///Writes an int with fixed length encoding
    pub fn write_fixed_int(&mut self, x: usize) -> Result<usize, EncodeError> {
        let config = bincode::config::standard().with_fixed_int_encoding();

        bincode::encode_into_std_write(x, &mut self.inner_writer, config)
    }
}
