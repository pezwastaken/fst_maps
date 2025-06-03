#[derive(Debug)]
pub struct NoTransitions;
#[derive(Debug)]
pub struct Simple(u8, bool);
#[derive(Debug)]
pub struct SimpleWithNext(u8, usize, bool);
#[derive(Debug)]
pub struct Complex(Vec<(u8, usize)>, bool);

impl NoTransitions {
    pub fn encode() -> u8 {
        0x50u8
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
    ///Writes the encoded symbol to buffer in reverse.
    ///Returns the number of written bytes (either 1 or 2).
    fn encode(&self, buffer: &mut Writer) -> usize {
        let c = self.0;

        match c {
            //fits in one encoded byte
            c if c < 16 => {
                //set first 4 bits to [000f] (byte continues)
                let f = (self.1 as u8) << 4;
                let res = c | f;

                buffer.write_byte(res);
                1
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

                buffer.write_byte(rightmost_bits);
                buffer.write_byte(res_1);
                2
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
}

impl SimpleWithNext {
    ///Returns the number of bytes needed.
    fn encode(&self, buffer: &mut Writer) -> usize {
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

                let written = encode_u64_reverse(buffer, to_state as u64);

                buffer.write_byte(res);

                written + 1
            }

            _ => {
                const SYMBOL_CONTINUES: u8 = 0x20;

                //first four bits (MS)
                let leftmost_bits = c >> 4;

                //set first 4 bits to [000f] (byte continues)
                let f = (is_final as u8) << 4;
                let res_1 = BASE_FIRST_BYTE | leftmost_bits | f | SYMBOL_CONTINUES;

                //move the four lower bits to the upper side
                let rightmost_bits = c << 4;

                let written = encode_u64_reverse(buffer, to_state as u64);

                buffer.write_byte(rightmost_bits);
                buffer.write_byte(res_1);

                written + 2
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
}

impl Complex {
    ///Returns the encoded symbol plus the number of bytes needed.
    ///Either 1 or 2.
    fn encode(&self, buffer: &mut Writer) -> usize {
        let transitions = &self.0;

        assert!(transitions.len() > 1);

        //encode last transition that will have a certain flag set (2nd bit from the left = 1)
        let mut start_from = self.encode_transition_last(buffer);

        for i in (1..transitions.len() - 1).rev() {
            start_from += self.encode_transition_not_last(i, buffer);
        }

        //know where to start to encode the next transitions
        self.encode_transition_not_last(0, buffer);

        buffer.count
    }

    ///Encode any transition (that is not the last one) of a complex state
    ///and returns the number of written bytes
    fn encode_transition_not_last(&self, t_index: usize, buffer: &mut Writer) -> usize {
        const BASE_FIRST_BYTE: u8 = 0x80u8;
        self.encode_transition(t_index, BASE_FIRST_BYTE, buffer)
    }

    ///Encode the first transition of a complex state and returns the number of written bytes
    fn encode_transition_last(&self, buffer: &mut Writer) -> usize {
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

    fn encode_transition(&self, t_index: usize, base: u8, buffer: &mut Writer) -> usize {
        let is_final = (self.1 as u8) << 4;

        //encode the first transition
        let (c, to_state) = self.0[t_index];

        match c {
            c if c < 16 => {
                let first_byte = base | c | is_final;

                let written = encode_u64_reverse(buffer, to_state as u64);
                buffer.write_byte(first_byte);

                written + 1
            }
            _ => {
                const SYMBOL_CONTINUES: u8 = 0x20;

                //first four bits (MS)
                let leftmost_bits = c >> 4;

                //set first 4 bits to [000f] (byte continues)
                let res_1 = base | SYMBOL_CONTINUES | is_final | leftmost_bits;

                let written = encode_u64_reverse(buffer, to_state as u64);

                //move the four lower bits to the upper side
                let rightmost_bits = c << 4;

                buffer.write_byte(rightmost_bits);
                buffer.write_byte(res_1);

                written + 2
            }
        }
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

#[derive(Debug)]
pub enum CompressedState {
    SimpleState(Simple),
    SimpleWithNextState(SimpleWithNext),
    ComplexState(Complex),
}

impl CompressedState {
    pub fn new(c: u8, f: bool) -> Self {
        Self::SimpleState(Simple(c, f))
    }

    pub fn new_simple_with_next(c: u8, to_state: usize, f: bool) -> Self {
        Self::SimpleWithNextState(SimpleWithNext(c, to_state, f))
    }

    pub fn new_complex(f: bool) -> Self {
        Self::ComplexState(Complex(Vec::with_capacity(2), f))
    }

    fn encode_state(state: &CompressedState, bytes: &mut [u8]) {
        //
    }
}

fn main() {
    let mut buffer = Writer::new();

    // dbg!(encode_u64_reverse(&mut buffer, 1000));
    //
    // let mut buffer = Writer::new();

    ////Simple
    // let simple_trans = Simple(b'a', false);
    //
    // simple_trans.encode(&mut buffer);
    //
    // let decoded = Simple::decode(&buffer.buffer[0..2]);
    // dbg!(decoded.0 as u8);
    // dbg!(decoded.1);

    ////SimpleWithNext
    // let state = SimpleWithNext(255, 10000, false);
    // let written = state.encode(&mut buffer);
    //
    // let decoded_state = SimpleWithNext::decode(&buffer.buffer[..written]);
    // dbg!(decoded_state);

    //Complex
    let state = Complex(
        vec![
            (10, 255),
            (b'a', 1000),
            (b'b', 2000),
            (11, usize::max_value()),
        ],
        false,
    );
    let wrote = state.encode(&mut buffer);
    dbg!(wrote);
    dbg!(&buffer.buffer[..20]);

    let decoded_state = Complex::decode(&buffer.buffer[..wrote]);
    dbg!(decoded_state);

    // let x: u64 = u64::max_value();
    // dbg!(x);
    //
    // let mut encoder = Encoder::new();
    // let bytes = encoder.encode_u64(x);
    // dbg!(&bytes);
    //
    // dbg!(decode_u64(&bytes.0));
}

///Encodes a u64 using varint and returns the number of written bytes
fn encode_u64_reverse(buffer: &mut Writer, v: u64) -> usize {
    if v < 0x80 {
        buffer.write_byte(v as u8);
        return 1;
    }
    inner_encode_u64_reverse(buffer, v)
}

fn inner_encode_u64_reverse(buffer: &mut Writer, v: u64) -> usize {
    match v {
        v if v < 1 << 14 => {
            let x = (v as u16) | 0x8000;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            2
        }
        v if v < 1 << 21 => {
            let x = ((v as u32) | 0xC00000) << 8;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            3
        }
        v if v < 1 << 28 => {
            let x = (v as u32) | 0xE0000000;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            buffer.write_byte((x >> 24) as u8);
            4
        }
        v if v < 1 << 35 => {
            let x = (v | 0xF000000000) << 24;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            buffer.write_byte((x >> 24) as u8);
            buffer.write_byte((x >> 32) as u8);
            5
        }
        v if v < 1 << 42 => {
            let x = (v | 0xF80000000000) << 16;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            buffer.write_byte((x >> 24) as u8);
            buffer.write_byte((x >> 32) as u8);
            buffer.write_byte((x >> 40) as u8);
            6
        }
        v if v < 1 << 49 => {
            let x = (v | 0xFC000000000000) << 8;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            buffer.write_byte((x >> 24) as u8);
            buffer.write_byte((x >> 32) as u8);
            buffer.write_byte((x >> 40) as u8);
            buffer.write_byte((x >> 48) as u8);
            7
        }
        v if v < 1 << 56 => {
            let x = v | 0xFE00000000000000;
            buffer.write_byte(x as u8);
            buffer.write_byte((x >> 8) as u8);
            buffer.write_byte((x >> 16) as u8);
            buffer.write_byte((x >> 24) as u8);
            buffer.write_byte((x >> 32) as u8);
            buffer.write_byte((x >> 40) as u8);
            buffer.write_byte((x >> 48) as u8);
            buffer.write_byte((x >> 56) as u8);
            8
        }
        _ => {
            buffer.write_byte(v as u8);
            buffer.write_byte((v >> 8) as u8);
            buffer.write_byte((v >> 16) as u8);
            buffer.write_byte((v >> 24) as u8);
            buffer.write_byte((v >> 32) as u8);
            buffer.write_byte((v >> 40) as u8);
            buffer.write_byte((v >> 48) as u8);
            buffer.write_byte((v >> 56) as u8);
            buffer.write_byte(0xFFu8);
            9
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

// fn encode_u64(v: u64) -> (Vec<u8>, usize) {
//     //fits in one byte
//     if v < 0x80 {
//         return (vec![v as u8], 1);
//     }
//     inner_encode_u64(v)
// }
//
// fn inner_encode_u64(v: u64) -> (Vec<u8>, usize) {
//     match v {
//         v if v < 1 << 14 => {
//             let x = (v as u16) | 0x8000;
//             (vec![(x >> 8) as u8, x as u8], 2)
//         }
//         v if v < 1 << 21 => {
//             let x = ((v as u32) | 0xC00000) << 8;
//             (vec![(x >> 16) as u8, (x >> 8) as u8, x as u8], 3)
//         }
//         v if v < 1 << 28 => {
//             let x = (v as u32) | 0xE0000000;
//             (
//                 vec![(x >> 24) as u8, (x >> 16) as u8, (x >> 8) as u8, x as u8],
//                 4,
//             )
//         }
//         v if v < 1 << 35 => {
//             let x = (v | 0xF000000000) << 24;
//             (
//                 vec![
//                     (x >> 32) as u8,
//                     (x >> 24) as u8,
//                     (x >> 16) as u8,
//                     (x >> 8) as u8,
//                     x as u8,
//                 ],
//                 5,
//             )
//         }
//         v if v < 1 << 42 => {
//             let x = (v | 0xF80000000000) << 16;
//             (
//                 vec![
//                     (x >> 40) as u8,
//                     (x >> 32) as u8,
//                     (x >> 24) as u8,
//                     (x >> 16) as u8,
//                     (x >> 8) as u8,
//                     x as u8,
//                 ],
//                 6,
//             )
//         }
//         v if v < 1 << 49 => {
//             let x = (v | 0xFC000000000000) << 8;
//             (
//                 vec![
//                     (x >> 48) as u8,
//                     (x >> 40) as u8,
//                     (x >> 32) as u8,
//                     (x >> 24) as u8,
//                     (x >> 16) as u8,
//                     (x >> 8) as u8,
//                     x as u8,
//                 ],
//                 7,
//             )
//         }
//         v if v < 1 << 56 => {
//             let x = v | 0xFE00000000000000;
//             (
//                 vec![
//                     (x >> 56) as u8,
//                     (x >> 48) as u8,
//                     (x >> 40) as u8,
//                     (x >> 32) as u8,
//                     (x >> 24) as u8,
//                     (x >> 16) as u8,
//                     (x >> 8) as u8,
//                     x as u8,
//                 ],
//                 8,
//             )
//         }
//         _ => (
//             vec![
//                 0xFFu8,
//                 (v >> 56) as u8,
//                 (v >> 48) as u8,
//                 (v >> 40) as u8,
//                 (v >> 32) as u8,
//                 (v >> 24) as u8,
//                 (v >> 16) as u8,
//                 (v >> 8) as u8,
//                 v as u8,
//             ],
//             9,
//         ),
//     }
// }
