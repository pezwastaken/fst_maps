use bincode::{Decode, error::DecodeError};
use memmap2::{Mmap, MmapOptions};
use std::fs::File;

use crate::automaton::AutomatonState;

pub struct AutomatonDecoder {
    mmap: Option<Mmap>,
    file: File,
}

impl AutomatonDecoder {
    pub fn new(f: &str) -> std::io::Result<Self> {
        let f = File::open(f)?;

        let map = unsafe { MmapOptions::new().map(&f).unwrap() };

        Ok(Self {
            mmap: Some(map),
            file: f,
        })
    }

    fn try_decode_state<T>(&self, offset: usize, len: usize) -> Result<T, DecodeError>
    where
        T: AutomatonState + Decode<()>,
    {
        if let Some(bytes) = self.mmap.as_ref() {
            let upper_bound = offset + len;

            let decoded = bincode::decode_from_slice::<T, _>(
                &bytes[offset..upper_bound],
                bincode::config::standard(),
            );

            //simply return the object while discarding the number of read bytes
            decoded.map(|(obj, _)| obj)
        } else {
            panic!("try_decode called on None mmap");
        }
    }

    fn try_decode_fixed_int(&self, offset: usize, len: usize) -> Result<usize, DecodeError> {
        if let Some(bytes) = self.mmap.as_ref() {
            //let config = bincode::config::standard().with_fixed_int_encoding();
            let upper_bound = offset + len;
            let config = bincode::config::standard().with_fixed_int_encoding();

            let decoded =
                bincode::decode_from_slice::<usize, _>(&bytes[offset..upper_bound], config);

            //simply return the integer while discarding the number of read bytes
            decoded.map(|(obj, _)| obj)
        } else {
            panic!("try_decode called on None mmap");
        }
    }

    //fn mmap_buf(&mut self, offset: u64, len: usize) {
    //    //TODO handle errors when mapping the file
    //    self.mmap = Some(unsafe {
    //        MmapOptions::new()
    //            .offset(offset)
    //            .len(len)
    //            .map(&self.file)
    //            .unwrap()
    //    });
    //    //dbg!(&self.mmap);
    //}

    fn file_len(&self) -> Option<u64> {
        match self.file.metadata() {
            Ok(meta) => Some(meta.len()),
            Err(_) => None,
        }
    }

    pub fn read_first_state<T>(&self) -> Option<T>
    where
        T: AutomatonState + Decode<()>,
    {
        const FIXED_U64_LEN: usize = 8;

        let file_len = self.file_len()?;
        let offset = file_len as usize - FIXED_U64_LEN;

        self.read_state_at_addr::<T>(offset)
    }

    ///decodes an fst state given its address by first deserializing its length (fixed 8 bytes
    ///starting from addr)
    pub fn read_state_at_addr<T>(&self, addr: usize) -> Option<T>
    where
        T: AutomatonState + Decode<()>,
    {
        const FIXED_U64_LEN: u64 = 8;

        //self.mmap_buf(addr, FIXED_U64_LEN as usize);

        //len (in bytes) of the initial state
        //conversion is safe
        let Ok(state_len) = self.try_decode_fixed_int(addr, FIXED_U64_LEN as usize) else {
            eprintln!(
                "couldn't decode fixed int starting from offset {} with len {} from file {:?}",
                addr,
                FIXED_U64_LEN,
                self.file.metadata()
            );
            return None;
        };

        //safe
        let offset = addr - state_len;
        //remap file using the new offset and first state len as the buffer length
        //self.mmap_buf(offset, state_len);

        let Ok(state) = self.try_decode_state::<T>(offset, state_len) else {
            eprintln!(
                "couldn't decode state starting from offset {} with len {} from file {:?}",
                offset,
                state_len,
                self.file.metadata()
            );
            return None;
        };

        return Some(state);
    }
}

#[allow(unused)]
#[cfg(test)]
mod fst_decoder_u_tests {
    use crate::automaton::AutomatonState;

    use super::AutomatonDecoder;

    //#[test]
    //fn file_len() {
    //    let decoder = FstDecoder::new("test.fst").expect("couln't open file");
    //
    //    assert_eq!(160, decoder.file_len().unwrap());
    //}
    //
    //#[test]
    //fn read_first_state() {
    //    let mut decoder = FstDecoder::new("test.fst").expect("couln't open file");
    //
    //    assert_eq!(160, decoder.file_len().unwrap());
    //
    //    let first_state = decoder.read_first_state();
    //    assert!(first_state.is_some());
    //
    //    //dbg!(first_state.unwrap());
    //}
    //
    //#[test]
    //fn walk_transition_from_q0() {
    //    let mut decoder = FstDecoder::new("test.fst").expect("couln't open file");
    //
    //    assert_eq!(160, decoder.file_len().unwrap());
    //
    //    let first_state = decoder.read_first_state().unwrap();
    //
    //    //dbg!(first_state.unwrap());
    //
    //    let (next_state_addr, output) = first_state.delta_with_output('c' as u8).unwrap();
    //    assert_eq!(output.get_value_as_deref().unwrap(), "ciao".as_bytes());
    //
    //    let next_state = decoder.read_state_at_addr(next_state_addr as u64);
    //    assert!(next_state.is_some());
    //
    //    let (next_state, output) = first_state.delta_with_output('c' as u8).unwrap();
    //    assert_eq!(output.get_value_as_deref().unwrap(), "ciao".as_bytes());
    //}
}
