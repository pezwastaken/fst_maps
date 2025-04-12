use bincode::{Decode, error::DecodeError};
use memmap2::{Mmap, MmapOptions};
use std::{fmt::Debug, fs::File};

use crate::fst_state::FstState;

pub struct FstDecoder {
    mmap: Option<Mmap>,
    file: File,
}

impl FstDecoder {
    pub fn new(f: &str) -> std::io::Result<Self> {
        Ok(Self {
            mmap: None,
            file: File::open(f)?,
        })
    }

    fn try_decode_state(&self) -> Result<FstState, DecodeError>
//where
    //    T: Decode<()> + Debug,
    {
        if let Some(bytes) = self.mmap.as_ref() {
            let decoded =
                bincode::decode_from_slice::<FstState, _>(bytes, bincode::config::standard());
            //simply return the object while discarding the number of read bytes
            decoded.map(|(obj, _)| obj)
            //
        } else {
            panic!("try_decode called on None mmap");
        }
    }

    fn try_decode_fixed_int(&self) -> Result<usize, DecodeError> {
        if let Some(bytes) = self.mmap.as_ref() {
            let config = bincode::config::standard().with_fixed_int_encoding();

            let decoded = bincode::decode_from_slice::<usize, _>(bytes, config);

            //simply return the integer while discarding the number of read bytes
            decoded.map(|(obj, _)| obj)
            //
        } else {
            panic!("try_decode called on None mmap");
        }
    }

    fn mmap_buf(&mut self, offset: u64, len: usize) {
        //let path: PathBuf = "test.txt".into();

        //let file = OpenOptions::new().read(true);
        //.write(true)
        //.create(true)
        //.open(&path)?;
        //file.set_len(13)?;

        //let file = File::open(self.file).expect("couldn't open fst file");

        //TODO handle errors when mapping the file
        self.mmap = Some(unsafe {
            MmapOptions::new()
                .offset(offset)
                .len(len)
                .map(&self.file)
                .unwrap()
        });
        dbg!(&self.mmap);
    }

    fn file_len(&self) -> Option<u64> {
        match self.file.metadata() {
            Ok(meta) => Some(meta.len()),
            Err(_) => None,
        }
    }

    fn read_first_state(&mut self) -> Option<FstState> {
        const FIXED_U64_LEN: u64 = 8;

        let file_len = self.file_len()?;
        let offset = file_len - FIXED_U64_LEN;

        self.mmap_buf(offset, FIXED_U64_LEN as usize);

        //len (in bytes) of the initial state
        //conversion is safe
        let Ok(first_state_len) = self.try_decode_fixed_int() else {
            eprintln!(
                "couldn't decode fixed int starting from offset {} with len {} from file {:?}",
                offset,
                FIXED_U64_LEN,
                self.file.metadata()
            );
            return None;
        };

        //safe
        let offset = offset - first_state_len as u64;
        //remap file using the new offset and first state len as the buffer length
        self.mmap_buf(offset, first_state_len);

        let Ok(first_state) = self.try_decode_state() else {
            eprintln!(
                "couldn't decode state starting from offset {} with len {} from file {:?}",
                offset,
                first_state_len,
                self.file.metadata()
            );
            return None;
        };

        return Some(first_state);
    }
}

#[allow(unused)]
#[cfg(test)]
mod fst_decoder_u_tests {
    use super::FstDecoder;

    #[test]
    fn file_len() {
        let decoder = FstDecoder::new("test.fst").expect("couln't open file");

        assert_eq!(162, decoder.file_len().unwrap());
    }

    #[test]
    fn read_first_state() {
        let mut decoder = FstDecoder::new("test.fst").expect("couln't open file");

        assert_eq!(162, decoder.file_len().unwrap());

        let first_state = decoder.read_first_state();
        assert!(first_state.is_some());

        dbg!(first_state.unwrap());
    }
}
