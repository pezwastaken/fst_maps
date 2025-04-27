use crate::bit_vector::{BitVector, BitVectorMask};

///Non-Deterministic Universal Levenshtein Automaton

pub struct NULAState {
    n_errs: usize,
    lane: isize,

    //a nula transition is a pair:
    //  bit_vector_mask
    //  next_state
    transitions: Vec<(BitVectorMask, usize)>,
}

impl NULAState {
    fn new_no_transitions(errs: usize, l: isize) -> Self {
        Self {
            n_errs: errs,
            lane: l,
            transitions: Vec::with_capacity(2),
        }
    }

    fn new_with_transitions(errs: usize, l: isize, t: Vec<(BitVectorMask, usize)>) -> Self {
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

pub struct NULA {
    //TODO
    //distance fixed to one for now
    //max_distance: usize,
    //
    states: Vec<NULAState>,
}

impl NULA {
    ///distance = 1
    pub fn new() -> Self {
        let mut nfa = Self {
            states: Vec::with_capacity(4),
        };

        let self_loop = (BitVectorMask::new([-1, -1, 1, -1, -1, 0, 0]), 0);
        let t_1 = (BitVectorMask::new([-1, -1, 0, -1, -1, 0, 0]), 1);
        let t_2 = (BitVectorMask::new([-1, -1, 0, -1, -1, 0, 0]), 2);
        let t_3 = (BitVectorMask::new([-1, -1, 0, 1, -1, 0, 0]), 3);
        nfa.states.push(NULAState::new_with_transitions(
            0,
            0,
            vec![self_loop, t_1, t_2, t_3],
        ));

        let self_loop = (BitVectorMask::new([-1, 1, -1, -1, -1, 0, 0]), 0);
        nfa.states
            .push(NULAState::new_with_transitions(1, -1, vec![self_loop]));

        let self_loop = (BitVectorMask::new([-1, -1, 1, -1, -1, 0, 0]), 0);
        nfa.states
            .push(NULAState::new_with_transitions(1, 0, vec![self_loop]));

        let self_loop = (BitVectorMask::new([-1, -1, -1, 1, -1, 0, 0]), 0);
        nfa.states
            .push(NULAState::new_with_transitions(1, 1, vec![self_loop]));

        nfa
    }
}

#[test]
fn build_nula() {
    let nfa = NULA::new();

    assert_eq!(nfa.states.len(), 4);
    assert_eq!(nfa.states[0].transitions.len(), 4);
    assert_eq!(nfa.states[1].transitions.len(), 1);
    assert_eq!(nfa.states[2].transitions.len(), 1);
    assert_eq!(nfa.states[3].transitions.len(), 1);
}

#[test]
fn state_delta_from_init_1() {
    let nfa = NULA::new();

    let init_state = &nfa.states[0];

    let mut bit_vector: BitVector = BitVector::new(5);
    bit_vector.set_one(2);
    let res = init_state.delta(&bit_vector).unwrap();

    assert_eq!(res.len(), 1);
    assert_eq!(res[0], 0);
}

#[test]
fn state_delta_from_init_2() {
    let nfa = NULA::new();

    let init_state = &nfa.states[0];

    let bit_vector: BitVector = BitVector::new(5);
    let res = init_state.delta(&bit_vector).unwrap();

    assert_eq!(res.len(), 2);
    assert_eq!(res[0], 1);
    assert_eq!(res[1], 2);
}

#[test]
fn state_delta_from_init_3() {
    let nfa = NULA::new();

    let init_state = &nfa.states[0];

    let mut bit_vector: BitVector = BitVector::new(5);
    bit_vector.set_one(3);
    bit_vector.set_one(4);

    let res = init_state.delta(&bit_vector).unwrap();

    assert_eq!(res.len(), 3);
    assert_eq!(res[0], 1);
    assert_eq!(res[1], 2);
    assert_eq!(res[2], 3);
}
