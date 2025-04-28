const VECTOR_MAX_SIZE: usize = 3;

///This struct is used to represent a word as bit vectors
///
///each character c in word w will be used to create a bit vector describing
///the indices at which the character c can be found (within w).
pub struct FullBitVectors {
    inner_bit_vectors: Vec<Vec<bool>>,

    //this contains an entry for each letter `c`(at index i) in word `w` and tells
    //the position where the bit vector for that letter resides (in inner_bit_vectors).
    //It's simply to avoid duplication of bit vectors.
    bit_vector_refs: Vec<usize>,
}

impl FullBitVectors {
    pub fn new(word: &str) -> Self {
        let word = word.as_bytes();
        let mut cache: [Option<usize>; 255] = [None; 255];

        //TODO: this may be a bit too much
        let mut inner_bitvecs: Vec<Vec<bool>> = Vec::with_capacity(word.len());
        let mut bitvec_refs: Vec<usize> = Vec::with_capacity(word.len());

        for c in word.iter() {
            //cache miss, the character still doesn't have a bit vector
            match cache[*c as usize] {
                None => {
                    //index at which the new bitvec will be found
                    let at_index = inner_bitvecs.len();

                    let bitvec = Self::new_bitvec(*c, word);
                    inner_bitvecs.push(bitvec);

                    //FIX: these two operations may be merged

                    //add to cache
                    cache[*c as usize] = Some(at_index);

                    //add the ref
                    bitvec_refs.push(at_index);
                }
                //bit vector is already in cache => no need to re-compute it
                Some(at_index) => {
                    bitvec_refs.push(at_index);
                }
            }
        }

        Self {
            inner_bit_vectors: inner_bitvecs,
            bit_vector_refs: bitvec_refs,
        }
    }

    ///Get bitvec at index i.
    pub fn get_bitvec(&self, i: usize) -> &[bool] {
        let at = self.bit_vector_refs[i];
        &self.inner_bit_vectors[at]
    }

    #[inline]
    fn new_bitvec(c: u8, word: &[u8]) -> Vec<bool> {
        let mut bitvec = vec![false; word.len()];
        for (i, x) in word.iter().enumerate() {
            if c == *x {
                bitvec[i] = true;
            }
        }
        bitvec
    }
}

///This struct represents a bit vector used by transitions of the nula.
///NOTE: this is not used by data type `FullBitVectors` since this is an optimized
///version that doesn't capture all occurrences of a character `c` in a word `w`.
pub struct BitVector {
    positions: [i8; VECTOR_MAX_SIZE],
}

impl BitVector {
    //TODO
    //new from (char, string)

    #[inline]
    pub fn new() -> Self {
        Self {
            positions: [0; VECTOR_MAX_SIZE],
        }
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
            self.positions[i] = 1;
        }
        //offset out of bounds, no op
    }

    #[inline]
    pub fn one_at_pos(&self, x: usize) -> bool {
        self.positions[x] == 1
    }

    #[inline]
    pub fn zero_at_pos(&self, x: usize) -> bool {
        self.positions[x] == 0
    }

    pub fn set_one(&mut self, at_index: usize) {
        self.positions[at_index] = 1;
    }
}

pub struct BitVectorMask([i8; VECTOR_MAX_SIZE]);

impl BitVectorMask {
    pub fn new(mask: [i8; VECTOR_MAX_SIZE]) -> Self {
        Self { 0: mask }
    }

    #[inline]
    pub fn mask_equal(&self, bit_vector: &BitVector) -> bool {
        let mut res = true;
        for i in 0..VECTOR_MAX_SIZE {
            res = res && ((self.0[i] == -1) || (self.0[i] == bit_vector.positions[i]));
        }
        res
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
    assert_eq!(bit_vector.positions, [1, 0, 0]);
}

#[test]
fn from_str_1() {
    let ref_str = [false, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(2, &ref_str);
    assert_eq!(bit_vector.positions, [1, 0, 0]);
}

#[test]
fn from_str_2() {
    let ref_str = [false, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(0, &ref_str);
    assert_eq!(bit_vector.positions, [0, 0, 1]);
}

#[test]
fn from_str_3() {
    let ref_str = [true, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(1, &ref_str);
    assert_eq!(bit_vector.positions, [1, 1, 0]);
}

#[test]
fn from_str_4() {
    let ref_str = [true, true, false];

    //0 0 1
    let bit_vector = BitVector::from_str(1, &ref_str);
    assert_eq!(bit_vector.positions, [1, 1, 0]);
}

#[test]
fn fullbitvec_1() {
    let word = "ciao";
    let bitvecs = FullBitVectors::new(word);

    assert_eq!(bitvecs.get_bitvec(0), [true, false, false, false]);
    assert_eq!(bitvecs.get_bitvec(1), [false, true, false, false]);
    assert_eq!(bitvecs.get_bitvec(2), [false, false, true, false]);
    assert_eq!(bitvecs.get_bitvec(3), [false, false, false, true]);

    assert_eq!(bitvecs.bit_vector_refs.len(), 4);
    assert_eq!(bitvecs.inner_bit_vectors.len(), 4);
}

#[test]
fn fullbitvec_2() {
    let word = "kiwi";
    let bitvecs = FullBitVectors::new(word);

    assert_eq!(bitvecs.bit_vector_refs.len(), 4);
    assert_eq!(bitvecs.inner_bit_vectors.len(), 3);

    assert_eq!(bitvecs.get_bitvec(0), [true, false, false, false]);
    assert_eq!(bitvecs.get_bitvec(1), [false, true, false, true]);
    assert_eq!(bitvecs.get_bitvec(2), [false, false, true, false]);
    assert_eq!(bitvecs.get_bitvec(3), [false, true, false, true]);
}
