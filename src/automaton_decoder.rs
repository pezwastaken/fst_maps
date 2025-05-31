use memmap2::{Mmap, MmapOptions};
use std::fmt::Debug;
use std::fs::File;

use crate::{
    compressed_states::{
        Complex, CompressedFsaState, CompressedStateTag, Decodable, NoTransitions, Simple,
        SimpleWithNext, decode_u64, detect_compressed_type,
    },
    raw_fsa_state::InnerFsaState,
};

pub struct AutomatonDecoder {
    mmap: Option<Mmap>,
    file: File,
    max_state_len: usize,
}

impl AutomatonDecoder {
    pub fn new(f: &str) -> std::io::Result<Self> {
        let f = File::open(f)?;

        let map = unsafe { MmapOptions::new().map(&f).unwrap() };

        Ok(Self {
            mmap: Some(map),
            file: f,
            max_state_len: 0,
        })
    }

    ///Returns the decoded state and the number of read bytes
    fn try_decode_state<T>(&self, addr: usize) -> Option<(T, usize)>
    where
        T: Decodable + Debug,
    {
        // assert!(self.prev_addr > offset);
        if let Some(bytes) = self.mmap.as_ref() {
            assert!(!bytes.is_empty());

            let state_slice = &bytes[..=addr];

            let (decoded_state, read_bytes) = T::from_slice(state_slice, addr);

            return Some((decoded_state, read_bytes));

            // &bytes[lower_bound..=addr],
        } else {
            panic!("try_decode_state called on None mmap");
        }
    }

    fn try_decode_int(&self, at: usize) -> Option<(u64, usize)> {
        if let Some(bytes) = self.mmap.as_ref() {
            const MAX_U64_LEN: usize = 9;

            let decoded = decode_u64(&bytes[..=at]);
            return Some(decoded);
        } else {
            todo!("should return None!");
        }
    }

    fn file_len(&self) -> Option<u64> {
        match self.file.metadata() {
            Ok(meta) => Some(meta.len()),
            Err(_) => None,
        }
    }

    pub fn read_first_state(&mut self) -> Option<InnerFsaState> {
        let file_len = self.file_len()?;
        assert_ne!(file_len, 0);
        // dbg!(file_len);

        let at = file_len as usize - 1;

        let Some((max_state_len, read_bytes)) = self.try_decode_int(at) else {
            return None;
        };

        // dbg!(max_state_len);
        // dbg!(read_bytes);

        self.max_state_len = max_state_len as usize;

        //here's where the first state begins
        let at = at - read_bytes;
        // dbg!(at);

        let res = self.read_state_at_addr(at);
        if at == 0 {
            println!("offset is 0 in read state at addr");
        }

        res
    }

    ///decodes an fst state given its address by first deserializing its length (fixed 8 bytes
    ///starting from addr)
    pub fn read_state_at_addr(&mut self, addr: usize) -> Option<InnerFsaState> {
        let Some(state) = self.try_decode_state::<CompressedFsaState>(addr) else {
            eprintln!(
                "couldn't decode state starting from offset {} from file {:?}",
                addr,
                self.file.metadata()
            );
            return None;
        };

        let res = InnerFsaState::from(state.0);
        // dbg!(&res);
        Some(res)
    }
}
