use bincode::{Decode, error::DecodeError};
use memmap2::{Mmap, MmapOptions};
use std::fs::File;

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

    fn try_decode_state<T>(&self, offset: usize) -> Result<T, DecodeError>
    where
        T: Decode<()>,
    {
        // assert!(self.prev_addr > offset);
        if let Some(bytes) = self.mmap.as_ref() {
            assert!(!bytes.is_empty());

            let upper_bound = offset + self.max_state_len;

            let decoded = bincode::decode_from_slice::<T, _>(
                &bytes[offset..upper_bound],
                bincode::config::standard(),
            );

            //simply return the object while discarding the number of read bytes
            decoded.map(|(obj, _)| obj)
        } else {
            panic!("try_decode_state called on None mmap");
        }
    }

    fn try_decode_int(&self, offset: usize, is_fixed_int: bool) -> Result<usize, DecodeError> {
        if let Some(bytes) = self.mmap.as_ref() {
            //let config = bincode::config::standard().with_fixed_int_encoding();
            //let upper_bound = offset + len;

            let decoded;
            if !is_fixed_int {
                let config = bincode::config::standard();
                decoded =
                    bincode::decode_from_slice::<usize, _>(&bytes[offset..offset + 8], config);
            } else {
                let config = bincode::config::standard().with_fixed_int_encoding();
                decoded =
                    bincode::decode_from_slice::<usize, _>(&bytes[offset..offset + 8], config);
            }

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

    ///Returns the first state and its address
    pub fn read_first_state<T>(&mut self) -> Option<(T, usize)>
    where
        T: Decode<()>,
    {
        const FIXED_U64_LEN: usize = 8;

        let file_len = self.file_len()?;
        assert_ne!(file_len, 0);

        let offset = file_len as usize - FIXED_U64_LEN;

        let Ok(max_state_len) = self.try_decode_int(offset, true) else {
            return None;
        };

        self.max_state_len = max_state_len;

        let offset = offset - FIXED_U64_LEN;

        let Ok(state_len) = self.try_decode_int(offset, true) else {
            return None;
        };

        assert!(offset > offset - state_len);
        let offset = offset - state_len;

        let res = self.read_state_at_addr::<T>(offset);
        if offset == 0 {
            println!("offset is 0 in read state at addr");
        }

        res.map(|v| (v, offset))
    }

    ///decodes an fst state given its address by first deserializing its length (fixed 8 bytes
    ///starting from addr)
    pub fn read_state_at_addr<T>(&mut self, addr: usize) -> Option<T>
    where
        T: Decode<()>,
    {
        const FIXED_U64_LEN: u64 = 8;

        let Ok(state) = self.try_decode_state::<T>(addr) else {
            eprintln!(
                "couldn't decode state starting from offset {} from file {:?}",
                addr,
                self.file.metadata()
            );
            return None;
        };

        return Some(state);
    }
}
