use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

pub trait Encodable {
    fn encode(&self, writer: &mut BufWriter<File>) -> io::Result<usize>;
}

#[derive(Debug)]
pub struct NoTransitions;
#[derive(Debug)]
pub struct Simple(u8, bool);
#[derive(Debug)]
pub struct SimpleWithNext(u8, usize, bool);
#[derive(Debug)]
pub struct Complex(Vec<(u8, usize)>, bool);

#[derive(Debug)]
pub enum CompressedFsaState {
    NoTransitionsState(NoTransitions, usize),
    SimpleState(Simple, usize),
    SimpleWithNextState(SimpleWithNext, usize),
    ComplexState(Complex, usize),
}

impl CompressedFsaState {
    pub fn set_addr(&mut self, addr: usize) {
        match self {
            CompressedFsaState::NoTransitionsState(_, x) => *x = addr,
            CompressedFsaState::SimpleState(_, x) => *x = addr,
            CompressedFsaState::SimpleWithNextState(_, x) => *x = addr,
            CompressedFsaState::ComplexState(_, x) => *x = addr,
        }
    }
}

impl Encodable for CompressedFsaState {
    fn encode(&self, writer: &mut BufWriter<File>) -> io::Result<usize> {
        match self {
            CompressedFsaState::NoTransitionsState(no_transitions, _) => {
                no_transitions.encode(writer)
            }
            CompressedFsaState::SimpleState(simple, _) => simple.encode(writer),
            CompressedFsaState::SimpleWithNextState(simple_with_next, _) => {
                simple_with_next.encode(writer)
            }
            CompressedFsaState::ComplexState(complex, _) => complex.encode(writer),
        }
    }
}

impl NoTransitions {
    pub fn encode(&self, buffer: &mut BufWriter<File>) -> io::Result<usize> {
        buffer.write(&[0x50u8])
    }

    pub fn decode(bytes: &[u8]) -> Self {
        assert_eq!(bytes[0], 0x50);
        NoTransitions
    }
}

struct Writer {
    buffer: [u8; 100],
    count: usize,
}

impl Writer {
    fn new() -> Self {
        Self {
            buffer: [0; 100],
            count: 0,
        }
    }

    fn write_byte(&mut self, b: u8) {
        self.buffer[self.count] = b;
        self.count += 1;
    }
}

impl Simple {
    pub fn new(b: u8, is_final: bool) -> Self {
        Self { 0: b, 1: is_final }
    }

    ///Writes the encoded symbol to buffer in reverse.
    ///Returns the number of written bytes (either 1 or 2).
    fn encode(&self, buffer: &mut BufWriter<File>) -> io::Result<usize> {
        let c = self.0;

        match c {
            //fits in one encoded byte
            c if c < 16 => {
                //set first 4 bits to [000f] (byte continues)
                let f = (self.1 as u8) << 4;
                let res = c | f;

                buffer.write(&[res])
                // buffer.write_byte(res);
                // 1
            }

            _ => {
                const SYMBOL_CONTINUES: u8 = 0x20;

                //first four bits (MS)
                let leftmost_bits = c >> 4;

                //set first 4 bits to [000f] (byte continues)
                let f = (self.1 as u8) << 4;
                let res_1 = leftmost_bits | f | SYMBOL_CONTINUES;

                //move the four lower bits to the upper side
                let rightmost_bits = c << 4;

                buffer.write(&[rightmost_bits, res_1])
            }
        }
    }

    fn decode(bytes: &[u8]) -> Self {
        let first_byte = bytes[bytes.len() - 1];
        match first_byte {
            c if first_byte < 32 => {
                let is_final = (c & 0x10) == 0x10;

                let tmp = c & 0x0F;
                Simple(tmp, is_final)
            }

            c => {
                let second_byte = bytes[bytes.len() - 2];

                let is_final = (c & 0x10) == 0x10;

                let first_4 = (c & 0x0F) << 4;
                let remaining = second_byte >> 4;

                let tmp = first_4 | remaining;

                Simple(tmp, is_final)
            }
        }
    }

    #[inline(always)]
    pub fn is_equal(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }

    #[inline(always)]
    pub fn get_symbol(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn is_final(&self) -> bool {
        self.1
    }
}

impl SimpleWithNext {
    pub fn new(b: u8, next: usize, is_final: bool) -> Self {
        Self {
            0: b,
            1: next,
            2: is_final,
        }
    }

    ///Returns the number of bytes needed.
    fn encode(&self, buffer: &mut BufWriter<File>) -> io::Result<usize> {
        let c = self.0;
        let to_state = self.1;
        let is_final = self.2;

        const BASE_FIRST_BYTE: u8 = 0x40u8;

        match c {
            //fits in one encoded byte
            c if c < 16 => {
                //set first 4 bits to [010f]
                let f = (is_final as u8) << 4;
                let res = BASE_FIRST_BYTE | c | f;

                let written = encode_u64_reverse(buffer, to_state as u64)?;

                buffer.write(&[res])?;

                Ok(written + 1)
            }

            _ => {
                const SYMBOL_CONTINUES: u8 = 0x20;

                //first four bits (MS)
                let leftmost_bits = c >> 4;

                //set first 4 bits to [000f] (byte continues)
                let f = (is_final as u8) << 4;
                let res_1 = BASE_FIRST_BYTE | leftmost_bits | f | SYMBOL_CONTINUES;
                println!("encoded first byte (simple WITH extra byte): {:08b}", res_1);

                //move the four lower bits to the upper side
                let rightmost_bits = c << 4;

                let written = encode_u64_reverse(buffer, to_state as u64)?;

                buffer.write(&[rightmost_bits, res_1])?;

                // buffer.write_byte(rightmost_bits);
                // buffer.write_byte(res_1);
                //
                Ok(written + 2)
            }
        }
    }

    fn decode(bytes: &[u8]) -> Self {
        let first_byte = bytes[bytes.len() - 1];

        let is_final = (first_byte & 0x10) == 0x10;

        match first_byte {
            c if first_byte <= 0x5F => {
                //char fits in one byte (first_byte) and then there's the varint u64

                let symbol = decode_symbol_1_byte(c);

                let (to_state, _) = decode_u64(&bytes[..bytes.len() - 1]);

                Self(symbol, to_state as usize, is_final)
            }

            _ => {
                let symbol = decode_symbol_2_bytes(first_byte, bytes[bytes.len() - 2]);

                let (to_state, _) = decode_u64(&bytes[..bytes.len() - 2]);

                Self(symbol, to_state as usize, is_final)
            }
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }

    pub fn get_symbol(&self) -> u8 {
        self.0
    }

    pub fn get_dest(&self) -> usize {
        self.1
    }
}

impl Complex {
    pub fn new(transitions: Vec<(u8, usize)>, is_final: bool) -> Self {
        Self {
            0: transitions,
            1: is_final,
        }
    }

    ///Returns the encoded symbol plus the number of bytes needed.
    ///Either 1 or 2.
    fn encode(&self, buffer: &mut BufWriter<File>) -> io::Result<usize> {
        let transitions = &self.0;

        assert!(transitions.len() > 1);

        //encode last transition that will have a certain flag set (2nd bit from the left = 1)
        let mut start_from = self.encode_transition_last(buffer)?;

        for i in (1..transitions.len() - 1).rev() {
            start_from += self.encode_transition_not_last(i, buffer)?;
        }

        //know where to start to encode the next transitions
        start_from += self.encode_transition_not_last(0, buffer)?;

        Ok(start_from)
    }

    ///Encode any transition (that is not the last one) of a complex state
    ///and returns the number of written bytes
    fn encode_transition_not_last(
        &self,
        t_index: usize,
        buffer: &mut BufWriter<File>,
    ) -> io::Result<usize> {
        const BASE_FIRST_BYTE: u8 = 0x80u8;
        self.encode_transition(t_index, BASE_FIRST_BYTE, buffer)
    }

    ///Encode the first transition of a complex state and returns the number of written bytes
    fn encode_transition_last(&self, buffer: &mut BufWriter<File>) -> io::Result<usize> {
        const BASE_FIRST_BYTE: u8 = 0xC0u8;
        let t_index = self.0.len() - 1;
        self.encode_transition(t_index, BASE_FIRST_BYTE, buffer)
    }

    pub fn decode(bytes: &[u8]) -> Self {
        let buffer_last = bytes.len() - 1;
        let first_byte = bytes[buffer_last];

        assert_eq!(first_byte & 0x40, 0);

        let is_final = (first_byte & 0x10) == 0x10;

        let mut transitions: Vec<(u8, usize)> = Vec::with_capacity(2);

        let mut i: usize = buffer_last;

        loop {
            dbg!(i);
            let hdr = bytes[i];
            // dbg!(i);

            let symbol = match hdr {
                //symbol fits in one byte
                c if (hdr & 0x20u8) == 0 => {
                    let tmp_symbol = decode_symbol_1_byte(c);
                    i -= 1;
                    dbg!(i);
                    tmp_symbol
                }

                //symbol continues in the next byte
                _ => {
                    let tmp_symbol = decode_symbol_2_bytes(bytes[i], bytes[i - 1]);
                    i -= 2;
                    dbg!(i);
                    tmp_symbol
                }
            };

            let (to_state, read) = decode_u64(&bytes[..=i]);
            dbg!(read);
            dbg!(i);

            dbg!(symbol as char);
            dbg!(to_state);

            transitions.push((symbol, to_state as usize));

            match hdr & 0x40u8 {
                0 => (),
                _ => break,
            }

            i -= read;
        }

        Self(transitions, is_final)
    }

    fn encode_transition(
        &self,
        t_index: usize,
        base: u8,
        buffer: &mut BufWriter<File>,
    ) -> io::Result<usize> {
        let is_final = (self.1 as u8) << 4;

        //encode the first transition
        let (c, to_state) = self.0[t_index];

        match c {
            c if c < 16 => {
                let first_byte = base | c | is_final;

                println!(
                    "encoded char (Complex (first) no extra byte): {:08b}",
                    first_byte
                );
                let written = encode_u64_reverse(buffer, to_state as u64)?;
                buffer.write(&[first_byte])?;

                Ok(written + 1)
            }
            _ => {
                const SYMBOL_CONTINUES: u8 = 0x20;

                //first four bits (MS)
                let leftmost_bits = c >> 4;

                //set first 4 bits to [000f] (byte continues)
                let res_1 = base | SYMBOL_CONTINUES | is_final | leftmost_bits;
                println!(
                    "encoded first byte (complex WITH extra byte): {:08b}",
                    res_1
                );

                let written = encode_u64_reverse(buffer, to_state as u64)?;

                //move the four lower bits to the upper side
                let rightmost_bits = c << 4;

                buffer.write(&[rightmost_bits, res_1])?;

                Ok(written + 2)
            }
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        let self_transitions = &self.0;
        let other_state_transitions = &other.0;

        //actually check transitions
        //compare char of transition 1 with char of transition 2
        //same thing with destination states
        let all_transitions_equal = self_transitions
            .iter()
            .zip(other_state_transitions)
            .all(|((char1, state1), (char2, state2))| (*char1 == *char2) && (*state1 == *state2));

        return all_transitions_equal;
    }

    #[inline(always)]
    pub fn get_transitions(&self) -> &[(u8, usize)] {
        &self.0
    }
}

#[inline]
fn decode_symbol_2_bytes(first_byte: u8, second_byte: u8) -> u8 {
    let first_4_bits = (first_byte & 0x0F) << 4;
    let remaining = second_byte >> 4;

    first_4_bits | remaining
}

#[inline]
fn decode_symbol_1_byte(byte: u8) -> u8 {
    byte & 0x0F
}

#[inline]
pub fn encoded_len(byte: u8) -> usize {
    if byte < 16 {
        return 1;
    }
    2
}

///Encodes a u64 using varint and returns the number of written bytes
fn encode_u64_reverse(buffer: &mut BufWriter<File>, v: u64) -> io::Result<usize> {
    if v < 0x80 {
        return buffer.write(&[v as u8]);
        // return 1;
    }
    inner_encode_u64_reverse(buffer, v)
}

fn inner_encode_u64_reverse(buffer: &mut BufWriter<File>, v: u64) -> io::Result<usize> {
    match v {
        v if v < 1 << 14 => {
            let x = (v as u16) | 0x8000;
            buffer.write(&[x as u8, (x >> 8) as u8])
            // 2
        }
        v if v < 1 << 21 => {
            let x = ((v as u32) | 0xC00000) << 8;
            buffer.write(&[x as u8, (x >> 8) as u8, (x >> 16) as u8])
            // 3
        }
        v if v < 1 << 28 => {
            let x = (v as u32) | 0xE0000000;
            buffer.write(&[x as u8, (x >> 8) as u8, (x >> 16) as u8, (x >> 24) as u8])
            // 4
        }
        v if v < 1 << 35 => {
            let x = (v | 0xF000000000) << 24;
            buffer.write(&[
                x as u8,
                (x >> 8) as u8,
                (x >> 16) as u8,
                (x >> 24) as u8,
                (x >> 32) as u8,
            ])
            // 5
        }
        v if v < 1 << 42 => {
            let x = (v | 0xF80000000000) << 16;
            buffer.write(&[
                x as u8,
                (x >> 8) as u8,
                (x >> 16) as u8,
                (x >> 24) as u8,
                (x >> 32) as u8,
                (x >> 40) as u8,
            ])
            // 6
        }
        v if v < 1 << 49 => {
            let x = (v | 0xFC000000000000) << 8;
            buffer.write(&[
                x as u8,
                (x >> 8) as u8,
                (x >> 16) as u8,
                (x >> 24) as u8,
                (x >> 32) as u8,
                (x >> 40) as u8,
                (x >> 48) as u8,
            ])
            // 7
        }
        v if v < 1 << 56 => {
            let x = v | 0xFE00000000000000;
            buffer.write(&[
                x as u8,
                (x >> 8) as u8,
                (x >> 16) as u8,
                (x >> 24) as u8,
                (x >> 32) as u8,
                (x >> 40) as u8,
                (x >> 48) as u8,
                (x >> 56) as u8,
            ])
            // 8
        }
        _ => {
            buffer.write(&[
                v as u8,
                (v >> 8) as u8,
                (v >> 16) as u8,
                (v >> 24) as u8,
                (v >> 32) as u8,
                (v >> 40) as u8,
                (v >> 48) as u8,
                (v >> 56) as u8,
                0xFFu8,
            ])
            // 9
        }
    }
}

///Decode the first available u64 starting from byte 0.
///Returns the decoded u64 and the number of read bytes.
///Giving it more bytes does simply nothing.
fn decode_u64(bytes: &[u8]) -> (u64, usize) {
    let buffer_last = bytes.len() - 1;
    let hdr = bytes[buffer_last];

    match hdr {
        hdr if (hdr <= 0x7F) => (hdr as u64, 1),
        hdr if (hdr <= 0xBF) => (
            ((hdr as u64 & 0x3F) << 8) | bytes[buffer_last - 1] as u64,
            2,
        ),
        hdr if (hdr <= 0xDF) => (
            ((hdr as u64 & 0x1F) << 16)
                | (bytes[buffer_last - 1] as u64) << 8
                | bytes[buffer_last - 2] as u64,
            3,
        ),
        hdr if (hdr <= 0xEF) => (
            ((hdr as u64 & 0x0F) << 24)
                | (bytes[buffer_last - 1] as u64) << 16
                | (bytes[buffer_last - 2] as u64) << 8
                | bytes[buffer_last - 3] as u64,
            4,
        ),
        hdr if (hdr <= 0xF7) => (
            ((hdr as u64 & 0x07) << 32)
                | (bytes[buffer_last - 1] as u64) << 24
                | (bytes[buffer_last - 2] as u64) << 16
                | (bytes[buffer_last - 3] as u64) << 8
                | bytes[buffer_last - 4] as u64,
            5,
        ),
        hdr if (hdr <= 0xFB) => (
            ((hdr as u64 & 0x03) << 40)
                | (bytes[buffer_last - 1] as u64) << 32
                | (bytes[buffer_last - 2] as u64) << 24
                | (bytes[buffer_last - 3] as u64) << 16
                | (bytes[buffer_last - 4] as u64) << 8
                | bytes[buffer_last - 5] as u64,
            6,
        ),
        hdr if (hdr <= 0xFD) => (
            ((hdr as u64 & 0x01) << 48)
                | (bytes[buffer_last - 1] as u64) << 40
                | (bytes[buffer_last - 2] as u64) << 32
                | (bytes[buffer_last - 3] as u64) << 24
                | (bytes[buffer_last - 4] as u64) << 16
                | (bytes[buffer_last - 5] as u64) << 8
                | bytes[buffer_last - 6] as u64,
            7,
        ),
        hdr if (hdr <= 0xFE) => (
            (bytes[buffer_last - 1] as u64) << 48
                | (bytes[buffer_last - 2] as u64) << 40
                | (bytes[buffer_last - 3] as u64) << 32
                | (bytes[buffer_last - 4] as u64) << 24
                | (bytes[buffer_last - 5] as u64) << 16
                | (bytes[buffer_last - 6] as u64) << 8
                | bytes[buffer_last - 7] as u64,
            7,
        ),
        _ => (
            (bytes[buffer_last - 1] as u64) << 56
                | (bytes[buffer_last - 2] as u64) << 48
                | (bytes[buffer_last - 3] as u64) << 40
                | (bytes[buffer_last - 4] as u64) << 32
                | (bytes[buffer_last - 5] as u64) << 24
                | (bytes[buffer_last - 6] as u64) << 16
                | (bytes[buffer_last - 7] as u64) << 8
                | bytes[buffer_last - 8] as u64,
            8,
        ),
    }
}
