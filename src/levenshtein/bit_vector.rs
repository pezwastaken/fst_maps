const VECTOR_MAX_SIZE: usize = 3;

///This struct is used to represent an arbitrary string as a collection of
/// bit vectors
///
///Each character c in word w will be used to create a bit vector describing
///the indices at which the character c can be found (within w).
pub struct FullBitVectors {
    inner_bit_vectors: Vec<Vec<bool>>,

    //this contains an entry for each letter `c`(at index i) in word `w` and tells
    //the position where the bit vector for that letter resides (in inner_bit_vectors).
    //It's simply to avoid duplication of bit vectors.
    bit_vector_refs: [Option<usize>; 255],
    sink: Vec<bool>,
}

impl FullBitVectors {
    ///Returns a sequence of bitvectors where its elements are used as references
    ///to generate the bitvectors that are going to be fed to the automaton.
    pub fn new(word: &[u8]) -> Self {
        let mut cache: [Option<usize>; 255] = [None; 255];

        let mut inner_bitvecs: Vec<Vec<bool>> = Vec::with_capacity(word.len());

        for c in word.iter() {
            //miss, the character still doesn't have a bit vector
            match cache[*c as usize] {
                None => {
                    //index at which the new bitvec will be found
                    let at_index = inner_bitvecs.len();

                    let bitvec = Self::new_bitvec(*c, word);
                    inner_bitvecs.push(bitvec);

                    //add to cache
                    cache[*c as usize] = Some(at_index);
                }
                //bit vector is already in cache => no need to re-compute it
                Some(_) => {}
            }
        }

        //push the bitvec for the terminator character
        let at_index = inner_bitvecs.len();
        let bitvec = Self::new_term_bitvec(word.len());
        inner_bitvecs.push(bitvec);
        cache[0] = Some(at_index);

        Self {
            inner_bit_vectors: inner_bitvecs,
            bit_vector_refs: cache,
            sink: vec![false; word.len()],
        }
    }

    ///Get bitvec for char `c`.
    pub fn get_bitvec(&self, c: u8) -> &[bool] {
        let at = self.bit_vector_refs[c as usize];
        at.map(|v| self.inner_bit_vectors[v].as_ref())
            .unwrap_or(&self.sink)
    }

    #[inline]
    fn new_bitvec(c: u8, word: &[u8]) -> Vec<bool> {
        let mut bitvec = vec![false; word.len() + 2];
        for (i, x) in word.iter().enumerate() {
            if c == *x {
                bitvec[i] = true;
            }
        }
        bitvec
    }

    #[inline]
    fn new_term_bitvec(word_len: usize) -> Vec<bool> {
        let mut bitvec = vec![false; word_len + 2];
        bitvec[word_len] = true;
        bitvec[word_len + 1] = true;
        bitvec
    }
}

///This struct represents a bit vector used by transitions of the nula.
///NOTE: this is not used by data type `FullBitVectors` since this is an optimized
///version that doesn't capture all occurrences of a character `c` in a word `w`.
#[derive(Debug)]
pub struct BitVector {
    positions: [bool; VECTOR_MAX_SIZE],
}

impl BitVector {
    //TODO
    //new from (char, string)

    #[inline]
    pub fn new() -> Self {
        Self {
            positions: [false; VECTOR_MAX_SIZE],
        }
    }

    pub const fn from_bool_arr(arr: [bool; VECTOR_MAX_SIZE]) -> Self {
        Self { positions: arr }
    }

    ///It is assumed that ref_string only contains the strictly necessary characters.
    ///No more no less.
    ///
    ///Example
    ///c = 'e'
    ///at_index = 2 'hue'
    ///
    ///ref_bit_vector (for 'e') = 0 1 0  'hey' results in this bitvec
    ///note that this ref_bit_vector is not optimized to a window of fixed size.
    ///It represents the full string as a sequence of booleans
    /// indicating whether the character at index i is equal to the one used
    /// as a "reference".
    ///
    ///
    ///from_str shall return:
    /// 1 0 0       since 'e' in ref_string is one char to the left of `at_index`
    pub fn from_str(at_index: usize, ref_bit_vector: &[bool]) -> Self {
        let mut bit_vector = Self::new();

        let ones = ref_bit_vector
            .iter()
            .enumerate()
            .filter(|(_, x)| **x == true);

        for (i, _) in ones {
            bit_vector.set_one_with_offset(i as isize - at_index as isize);
        }
        bit_vector
    }

    //TODO should make sure that offset is within the valid bounds
    #[inline]
    fn set_one_with_offset(&mut self, offset: isize) {
        if offset >= -1 && offset <= 1 {
            let i: usize = (offset + 1) as usize;
            assert!(i <= 2);
            self.positions[i] = true;
        }
        //offset out of bounds, no op
    }

    ///Used to generate the bitvector for the terminator character
    #[inline]
    pub fn terminator_bitvec(len_current_str: usize, len_ref_str: usize) -> Self {
        let offset = len_ref_str as isize - len_current_str as isize;

        let mut bitvec = [false; VECTOR_MAX_SIZE];

        let offset: isize = if offset < -1 { -1 } else { offset };

        if offset >= -1 && offset <= 1 {
            let offset: usize = (offset + 1) as usize;
            for i in offset..VECTOR_MAX_SIZE {
                bitvec[i] = true;
            }
        }
        Self { positions: bitvec }
    }

    #[inline]
    pub fn one_at_pos(&self, x: usize) -> bool {
        self.positions[x]
    }

    #[inline]
    pub fn zero_at_pos(&self, x: usize) -> bool {
        !self.positions[x]
    }

    pub fn set_one(&mut self, at_index: usize) {
        self.positions[at_index] = true;
    }

    pub fn get_positions(&self) -> &[bool] {
        &self.positions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitVectorMask([i8; VECTOR_MAX_SIZE]);

impl BitVectorMask {
    pub fn new(mask: [i8; VECTOR_MAX_SIZE]) -> Self {
        Self { 0: mask }
    }

    #[inline]
    pub fn mask_equal(&self, bit_vector: &BitVector) -> bool {
        let mut res = true;
        for i in 0..VECTOR_MAX_SIZE {
            res = res
                && ((self.0[i] == -1)
                    || (self.0[i] == 0 && bit_vector.positions[i] == false)
                    || (self.0[i] == 1 && bit_vector.positions[i]));
        }
        res
    }
}

impl From<&BitVector> for BitVectorMask {
    fn from(value: &BitVector) -> Self {
        let mut arr = [0; VECTOR_MAX_SIZE];
        for i in 0..VECTOR_MAX_SIZE {
            if value.one_at_pos(i) {
                arr[i] = 1;
            }
        }
        Self { 0: arr }
    }
}

#[test]
fn test_mask_1() {
    let mut bit_vector = BitVector::new();
    bit_vector.set_one(2);

    let mask = BitVectorMask([-1; 3]);
    assert!(mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_2() {
    let mut bit_vector = BitVector::new();
    bit_vector.set_one(1);

    let mask = BitVectorMask([-1, 1, -1]);
    assert!(mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_3() {
    let mut bit_vector = BitVector::new();
    bit_vector.set_one(1);

    let mask = BitVectorMask([-1, 0, -1]);
    assert!(!mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_4() {
    let mut bit_vector = BitVector::new();
    bit_vector.set_one(0);
    let mask = BitVectorMask([1, 0, 0]);
    assert!(mask.mask_equal(&bit_vector));
}

#[test]
fn set_with_offset() {
    let mut bit_vector = BitVector::new();
    bit_vector.set_one_with_offset(-1);
    assert_eq!(bit_vector.positions, [true, false, false]);
}

#[test]
fn from_str_1() {
    let ref_str = [false, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(2, &ref_str);
    assert_eq!(bit_vector.positions, [true, false, false]);
}

#[test]
fn from_str_2() {
    let ref_str = [false, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(0, &ref_str);
    assert_eq!(bit_vector.positions, [false, false, true]);
}

#[test]
fn from_str_3() {
    let ref_str = [true, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(1, &ref_str);
    assert_eq!(bit_vector.positions, [true, true, false]);
}

#[test]
fn from_str_4() {
    let ref_str = [true, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(1, &ref_str);
    assert_eq!(bit_vector.positions, [true, true, false]);
}

#[test]
fn from_str_5() {
    let word = "pizza";
    let bitvecs = FullBitVectors::new(word.as_bytes());

    let bitvec_a = bitvecs.get_bitvec(b'a');
    assert_eq!(bitvec_a, [false, false, false, false, true, false, false]);

    //let's assume word `all`
    let x = BitVector::from_str(0, bitvec_a);
    assert_eq!(x.positions, [false, false, false]);

    let bitvec_z = bitvecs.get_bitvec(b'z');
    assert_eq!(bitvec_z, [false, false, true, true, false, false, false]);

    let x = BitVector::from_str(2, bitvec_z);
    assert_eq!(x.positions, [false, true, true]);

    let bitvec_i = bitvecs.get_bitvec(b'i');
    assert_eq!(bitvec_i, [false, true, false, false, false, false, false]);

    let x = BitVector::from_str(8, bitvec_i);
    assert_eq!(x.positions, [false, false, false]);

    let x = BitVector::from_str(3, bitvec_i);
    assert_eq!(x.positions, [false, false, false]);

    let x = BitVector::from_str(2, bitvec_i);
    assert_eq!(x.positions, [true, false, false]);
}

#[test]
fn from_str_6() {
    let ref_word = "pizza";
    let bitvecs = FullBitVectors::new(ref_word.as_bytes());

    let current_word = b"paz";

    for (i, c) in current_word.iter().enumerate() {
        let ref_bitvec = bitvecs.get_bitvec(*c);

        let _ = BitVector::from_str(i, ref_bitvec);
        //dbg!(res);
    }

    let bitvec_a = bitvecs.get_bitvec(b'a');
    assert_eq!(bitvec_a, [false, false, false, false, true, false, false]);

    //let's assume word `all`
    let x = BitVector::from_str(0, bitvec_a);
    assert_eq!(x.positions, [false, false, false]);

    let bitvec_z = bitvecs.get_bitvec(b'z');
    assert_eq!(bitvec_z, [false, false, true, true, false, false, false]);

    let x = BitVector::from_str(2, bitvec_z);
    assert_eq!(x.positions, [false, true, true]);

    let bitvec_i = bitvecs.get_bitvec(b'i');
    assert_eq!(bitvec_i, [false, true, false, false, false, false, false]);

    let x = BitVector::from_str(8, bitvec_i);
    assert_eq!(x.positions, [false, false, false]);

    let x = BitVector::from_str(3, bitvec_i);
    assert_eq!(x.positions, [false, false, false]);

    let x = BitVector::from_str(2, bitvec_i);
    assert_eq!(x.positions, [true, false, false]);
}

#[test]
fn from_str_term() {
    let word = "pizza";
    let bitvecs = FullBitVectors::new(word.as_bytes());

    let bitvec_0 = bitvecs.get_bitvec(0);
    assert_eq!(bitvec_0, [false, false, false, false, false, true, true]);

    //let's assume word `all`
    let x = BitVector::from_str(4, bitvec_0);
    assert_eq!(x.positions, [false, false, true]);
}

#[test]
fn fullbitvec_1() {
    let word = "ciao";
    let bitvecs = FullBitVectors::new(word.as_bytes());

    assert_eq!(
        bitvecs.get_bitvec(b'c'),
        [true, false, false, false, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'i'),
        [false, true, false, false, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'a'),
        [false, false, true, false, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'o'),
        [false, false, false, true, false, false]
    );

    assert_eq!(bitvecs.inner_bit_vectors.len(), 5);
}

#[test]
fn fullbitvec_2() {
    let word = "kiwi";
    let bitvecs = FullBitVectors::new(word.as_bytes());

    assert_eq!(bitvecs.inner_bit_vectors.len(), 4);

    assert_eq!(
        bitvecs.get_bitvec(b'k'),
        [true, false, false, false, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'i'),
        [false, true, false, true, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'w'),
        [false, false, true, false, false, false]
    );
    assert_eq!(
        bitvecs.get_bitvec(b'i'),
        [false, true, false, true, false, false]
    );
}

#[test]
fn test_shorter_ref_1() {
    let ref_str = "pan";
    let bitvecs = FullBitVectors::new(ref_str.as_bytes());

    // let word = "pangolino";
    let bitvec_1 = BitVector::from_str(0, bitvecs.get_bitvec(b'p'));
    dbg!(bitvec_1);

    let bitvec_1 = BitVector::from_str(7, bitvecs.get_bitvec(b'n'));
    dbg!(bitvec_1);

    let bitvec_1 = BitVector::terminator_bitvec(9, ref_str.len());
    assert_eq!(bitvec_1.positions, [true, true, true]);
    dbg!(bitvec_1);
}
