use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
struct Topo(u8);

#[derive(Debug, Clone, Encode, Decode)]
struct BigTopo(Vec<(u8, usize)>);

#[derive(Debug, Clone, Encode, Decode)]
struct FinalState(u8);
#[derive(Debug, Clone, Encode, Decode)]
struct BasicState(u8);
#[derive(Debug, Clone, Encode, Decode)]
struct BasicStateWithNext(u8, u64);
#[derive(Debug, Clone, Encode, Decode)]
struct MultiTransition(Vec<(u8, usize)>);

#[derive(Debug, Encode, Decode)]
enum State {
    Final(FinalState),
    Basic(BasicState),
    BasicWithNext(BasicStateWithNext),
    Multi(MultiTransition),
}

fn needs_byte(c: u8) -> bool {
    let mask = 0b0100_0000u8;
    c & mask != 0

    // if c & mask != 0 {
    //     return true;
    // }
    // false
}

fn encode_byte(c: u8) -> [u8; 2] {
    const SHIFT_ONE_BYTE: u8 = 8;
    // const : u8 = 8;

    println!("{:08b}", c);
    //first 6 bits from the left
    let mut leftmost_bits = c >> 2;
    //set first 2 bits to 11 (byte continues)
    leftmost_bits |= 0b1100_0000u8;
    println!("leftmost bits: {:08b}", leftmost_bits);

    //last two bits from the left (first two from the right)
    let rightmost_bits = (c << 4) & 0b0011_0000u8;
    println!("rightmost bits: {:08b}", rightmost_bits);
    // let res: u16 = 0u16 | (leftmost_bits as u16) << SHIFT_ONE_BYTE | rightmost_bits as u16;

    // println!("res: {:016b}", res);
    [leftmost_bits, rightmost_bits]
    // res
}

fn continues(b: u8) -> bool {
    b & 0b0100_0000u8 != 0
}

///decode a u8 that's been encoded in a slice of bytes.
///Checking the 2nd bit in every byte tells us whether we need to read another byte or not.
fn decode_byte(bytes: &[u8]) -> u8 {
    let first_byte = bytes[0];
    if continues(first_byte) {
        let first = first_byte << 2;

        //a char cannot be more than two bytes long (when encoded)
        assert!(bytes[1] & 0b0100_0000 == 0);

        let second = bytes[1] >> 4;

        return first | second;
    } else {
        return first_byte & 0b0011_1111u8;
    }
}

fn continues_as_int(b: u8) -> u8 {
    (b >> 6) & 0b0000_0001
}

fn prefix_no_var_len(c: u8) -> u8 {
    c | 0b1000_0000
}

//TODO: use an iterator for this!
//It removes the need for allocations!!!!!!!!!!!
// fn encode()

fn main() {
    // let c: u8 = 255u8;
    // println!("original: {:08b}", c);
    //
    // let x: u64 = 256;
    // let res = x.to_be_bytes();
    //
    // dbg!(res.len());
    // println!("byte 1: {:08b}", res[0]);
    // println!("byte 2: {:08b}", res[1]);

    let x: u64 = u64::max_value();
    dbg!(x);

    let mut encoder = Encoder::new();
    let bytes = encoder.encode_u64(x);
    dbg!(&bytes);

    dbg!(decode_u64(&bytes.0));

    // if needs_byte(c) {
    //     println!("{} needs another byte!", c);
    //
    //     let tmp = encode_byte(c);
    //
    //     let tmp = decode_byte(&tmp);
    //     println!("decoded: {:08b}", tmp);
    //     println!("char: {}", tmp);
    // } else {
    //     let encoded = prefix_no_var_len(c);
    //     println!("encoded: {:08b}", encoded);
    //     let bytes = [encoded; 1];
    //     let res = decode_byte(&bytes);
    //     println!("decoded: {:08b}", res);
    //     println!("char (original): {}", c as char);
    //     println!("char (decoded): {}", res as char);
    // }
    //
    //
}

struct Encoder {
    encoded_bytes: [u8; 9],
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            encoded_bytes: [0; 9],
        }
    }

    pub fn encode_u64(&mut self, v: u64) -> (&[u8], usize) {
        if v < 0x80 {
            self.encoded_bytes[0] = v as u8;
            return (&self.encoded_bytes[..1], 1);
        }
        self.inner_encode_u64(v)
    }

    fn inner_encode_u64(&mut self, v: u64) -> (&[u8], usize) {
        match v {
            v if v < 1 << 14 => {
                let x = (v as u16) | 0x8000;
                self.encoded_bytes[0] = (x >> 8) as u8;
                self.encoded_bytes[1] = x as u8;
                (&self.encoded_bytes[..2], 2)
            }
            v if v < 1 << 21 => {
                let x = ((v as u32) | 0xC00000) << 8;
                self.encoded_bytes[0] = (x >> 16) as u8;
                self.encoded_bytes[1] = (x >> 8) as u8;
                self.encoded_bytes[2] = x as u8;
                (&self.encoded_bytes[..3], 3)
            }
            v if v < 1 << 28 => {
                let x = (v as u32) | 0xE0000000;
                self.encoded_bytes[0] = (x >> 24) as u8;
                self.encoded_bytes[1] = (x >> 16) as u8;
                self.encoded_bytes[2] = (x >> 8) as u8;
                self.encoded_bytes[3] = x as u8;
                (&self.encoded_bytes[..4], 4)
            }
            v if v < 1 << 35 => {
                let x = (v | 0xF000000000) << 24;
                self.encoded_bytes[0] = (x >> 32) as u8;
                self.encoded_bytes[1] = (x >> 24) as u8;
                self.encoded_bytes[2] = (x >> 16) as u8;
                self.encoded_bytes[3] = (x >> 8) as u8;
                self.encoded_bytes[4] = x as u8;
                (&self.encoded_bytes[..5], 5)
            }
            v if v < 1 << 42 => {
                let x = (v | 0xF80000000000) << 16;
                self.encoded_bytes[0] = (x >> 40) as u8;
                self.encoded_bytes[1] = (x >> 32) as u8;
                self.encoded_bytes[2] = (x >> 24) as u8;
                self.encoded_bytes[3] = (x >> 16) as u8;
                self.encoded_bytes[4] = (x >> 8) as u8;
                self.encoded_bytes[5] = x as u8;
                (&self.encoded_bytes[..6], 6)
            }
            v if v < 1 << 49 => {
                let x = (v | 0xFC000000000000) << 8;
                self.encoded_bytes[0] = (x >> 48) as u8;
                self.encoded_bytes[1] = (x >> 40) as u8;
                self.encoded_bytes[2] = (x >> 32) as u8;
                self.encoded_bytes[3] = (x >> 24) as u8;
                self.encoded_bytes[4] = (x >> 16) as u8;
                self.encoded_bytes[5] = (x >> 8) as u8;
                self.encoded_bytes[6] = x as u8;
                (&self.encoded_bytes[..7], 7)
            }
            v if v < 1 << 56 => {
                let x = v | 0xFE00000000000000;
                self.encoded_bytes[0] = (x >> 56) as u8;
                self.encoded_bytes[1] = (x >> 48) as u8;
                self.encoded_bytes[2] = (x >> 40) as u8;
                self.encoded_bytes[3] = (x >> 32) as u8;
                self.encoded_bytes[4] = (x >> 24) as u8;
                self.encoded_bytes[5] = (x >> 16) as u8;
                self.encoded_bytes[6] = (x >> 8) as u8;
                self.encoded_bytes[7] = x as u8;
                (&self.encoded_bytes[..8], 8)
            }
            _ => {
                self.encoded_bytes[0] = 0xFFu8;
                self.encoded_bytes[1] = (v >> 56) as u8;
                self.encoded_bytes[2] = (v >> 48) as u8;
                self.encoded_bytes[3] = (v >> 40) as u8;
                self.encoded_bytes[4] = (v >> 32) as u8;
                self.encoded_bytes[5] = (v >> 24) as u8;
                self.encoded_bytes[6] = (v >> 16) as u8;
                self.encoded_bytes[7] = (v >> 8) as u8;
                self.encoded_bytes[8] = v as u8;
                (&self.encoded_bytes[..9], 9)
            }
        }
    }
}

///Decode the first available u64 starting from byte 0.
///Giving it more bytes does simply nothing.
fn decode_u64(bytes: &[u8]) -> u64 {
    let hdr = bytes[0];

    match hdr {
        hdr if (hdr <= 0x7F) => hdr as u64,
        hdr if (hdr <= 0xBF) => ((hdr as u64 & 0x3F) << 8) | bytes[1] as u64,
        hdr if (hdr <= 0xDF) => {
            ((hdr as u64 & 0x1F) << 16) | (bytes[1] as u64) << 8 | bytes[2] as u64
        }
        hdr if (hdr <= 0xEF) => {
            ((hdr as u64 & 0x0F) << 24)
                | (bytes[1] as u64) << 16
                | (bytes[2] as u64) << 8
                | bytes[3] as u64
        }
        hdr if (hdr <= 0xF7) => {
            ((hdr as u64 & 0x07) << 32)
                | (bytes[1] as u64) << 24
                | (bytes[2] as u64) << 16
                | (bytes[3] as u64) << 8
                | bytes[4] as u64
        }
        hdr if (hdr <= 0xFB) => {
            ((hdr as u64 & 0x03) << 40)
                | (bytes[1] as u64) << 32
                | (bytes[2] as u64) << 24
                | (bytes[3] as u64) << 16
                | (bytes[4] as u64) << 8
                | bytes[5] as u64
        }
        hdr if (hdr <= 0xFD) => {
            ((hdr as u64 & 0x01) << 48)
                | (bytes[1] as u64) << 40
                | (bytes[2] as u64) << 32
                | (bytes[3] as u64) << 24
                | (bytes[4] as u64) << 16
                | (bytes[5] as u64) << 8
                | bytes[6] as u64
        }
        hdr if (hdr <= 0xFE) => {
            (bytes[1] as u64) << 48
                | (bytes[2] as u64) << 40
                | (bytes[3] as u64) << 32
                | (bytes[4] as u64) << 24
                | (bytes[5] as u64) << 16
                | (bytes[6] as u64) << 8
                | bytes[7] as u64
        }
        _ => {
            (bytes[1] as u64) << 56
                | (bytes[2] as u64) << 48
                | (bytes[3] as u64) << 40
                | (bytes[4] as u64) << 32
                | (bytes[5] as u64) << 24
                | (bytes[6] as u64) << 16
                | (bytes[7] as u64) << 8
                | bytes[8] as u64
        }
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
