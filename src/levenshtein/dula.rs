use std::marker::PhantomData;

use crate::{
    bit_vector::{BitVector, BitVectorMask},
    nula::{Max1, NULA, NULADistance},
};

struct DULAState {
    n_errs: isize,
    lane: isize,

    //a dula transition is a pair:
    //  bit_vector_mask
    //  next_state
    transitions: Vec<(BitVectorMask, usize)>,
}

impl DULAState {
    fn new_no_transitions(errs: isize, l: isize) -> Self {
        Self {
            n_errs: errs,
            lane: l,
            transitions: Vec::with_capacity(2),
        }
    }

    fn new_with_transitions(errs: isize, l: isize, t: Vec<(BitVectorMask, usize)>) -> Self {
        Self {
            n_errs: errs,
            lane: l,
            transitions: t,
        }
    }

    ///It's an NFA so delta returns a set of states
    fn delta(&self, bit_vector: &BitVector) -> Option<Vec<usize>> {
        let mut res: Vec<usize> = vec![];
        for t in self.transitions.iter() {
            if t.0.mask_equal(bit_vector) {
                res.push(t.1);
            }
        }

        if res.len() > 0 {
            return Some(res);
        }
        None
    }
}

pub struct DULA<T>
where
    T: NULADistance,
{
    //TODO
    //distance fixed to one for now
    //max_distance: usize,
    //
    states: Vec<DULAState>,
    _marker: std::marker::PhantomData<T>,
}

impl DULA<Max1> {
    pub fn new() -> Self {
        let mut dfa = Self {
            states: Vec::with_capacity(8),
            _marker: PhantomData,
        };

        dfa.states.push(DULAState::new_no_transitions(0, 0));
        dfa
    }
}

//possible bit vectors for distance 1
const fixed_bit_vectors: [[bool; 3]; 8] = [
    [false, false, false],
    [false, false, true],
    [false, true, false],
    [true, false, false],
    [false, true, true],
    [true, false, true],
    [true, true, false],
    [true, true, true],
];

struct SetOfStates {
    set: Vec<usize>,
}

impl SetOfStates {
    fn new() -> Self {
        Self {
            set: Vec::with_capacity(4),
        }
    }

    fn contains(&self, s: usize) -> bool {
        self.set.iter().any(|x| *x == s)
    }
}

impl PartialEq for SetOfStates {
    fn eq(&self, other: &Self) -> bool {
        self.set.iter().all(|v| other.contains(*v))
    }
}

pub fn build_dula(nula: &NULA<Max1>) -> DULA<Max1> {
    //initialized with one state (0, 0) => 0 errors, 0th lane
    let mut dula = DULA::<Max1>::new();

    let mut marked_states: Vec<usize> = Vec::with_capacity(8);
    marked_states.push(0);

    loop {
        //while there exists an unmarked state
        let t = marked_states.pop() else {
            break;
        };

        for bitvec in fixed_bit_vectors.iter() {

            //
        }
    }

    dula
}
