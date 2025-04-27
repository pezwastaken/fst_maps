const VECTOR_MAX_SIZE: usize = 7;

pub struct BitVector {
    width: usize,
    positions: [i8; VECTOR_MAX_SIZE],
}

impl BitVector {
    //TODO
    //new from (char, string)

    #[inline]
    pub fn new(w: usize) -> Self {
        Self {
            width: w,
            positions: [0; VECTOR_MAX_SIZE],
        }
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
        assert!(at_index < self.width);
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
        for i in 0..bit_vector.width {
            res = res && ((self.0[i] == -1) || (self.0[i] == bit_vector.positions[i]));
        }
        res
    }
}

#[test]
fn test_mask_1() {
    let mut bit_vector = BitVector::new(5);
    bit_vector.set_one(2);

    assert_eq!(bit_vector.width, 5);

    let mask = BitVectorMask([-1; 7]);
    assert!(mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_2() {
    let mut bit_vector = BitVector::new(5);
    bit_vector.set_one(2);

    assert_eq!(bit_vector.width, 5);

    let mask = BitVectorMask([-1, -1, 1, -1, -1, -1, -1]);
    assert!(mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_3() {
    let mut bit_vector = BitVector::new(5);
    bit_vector.set_one(2);

    assert_eq!(bit_vector.width, 5);

    let mask = BitVectorMask([-1, -1, 0, -1, -1, -1, -1]);
    assert!(!mask.mask_equal(&bit_vector));
}

#[test]
fn test_mask_4() {
    let mut bit_vector = BitVector::new(5);
    bit_vector.set_one(0);
    bit_vector.set_one(4);

    assert_eq!(bit_vector.width, 5);

    let mask = BitVectorMask([1, 0, 0, 0, 1, -1, -1]);
    assert!(mask.mask_equal(&bit_vector));
}
