use std::marker::PhantomData;

use crate::bit_vector::{BitVector, BitVectorMask, FullBitVectors};

///Non-Deterministic Universal Levenshtein Automaton
pub struct NULAState {
    n_errs: isize,
    lane: isize,

    //a nula transition is a pair:
    //  bit_vector_mask
    //  next_state
    transitions: Vec<(BitVectorMask, usize)>,
}

impl NULAState {
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

    ///Used by the reduction algorithm to find redundant states
    fn subsumes(&self, other: &Self) -> bool {
        let tmp1 = self.lane + self.n_errs - other.n_errs;
        let tmp2 = self.lane + other.lane - self.lane;

        if self.n_errs < other.n_errs && (tmp1 <= other.lane && other.lane <= tmp2) {
            return true;
        }
        false
    }
}

pub trait NULADistance {}

#[derive(Debug)]
pub struct Max1 {}

impl NULADistance for Max1 {}

pub struct NULA<T>
where
    T: NULADistance,
{
    //TODO
    //distance fixed to one for now
    //max_distance: usize,
    //
    states: Vec<NULAState>,
    _marker: std::marker::PhantomData<T>,
}

//NOTE: Only implemented for constant distance = 1
//TODO: This could computed at compile time
impl NULA<Max1> {
    pub fn new() -> Self {
        let mut nfa = Self {
            states: Vec::with_capacity(4),
            _marker: PhantomData,
        };

        let self_loop = (BitVectorMask::new([-1, 1, -1]), 0);
        let t_1 = (BitVectorMask::new([-1, 0, -1]), 1);
        let t_2 = (BitVectorMask::new([-1, 0, -1]), 2);
        let t_3 = (BitVectorMask::new([-1, 0, 1]), 3);
        nfa.states.push(NULAState::new_with_transitions(
            0,
            0,
            vec![self_loop, t_1, t_2, t_3],
        ));

        let self_loop = (BitVectorMask::new([1, -1, -1]), 1);
        nfa.states
            .push(NULAState::new_with_transitions(1, -1, vec![self_loop]));

        let self_loop = (BitVectorMask::new([-1, 1, -1]), 2);
        nfa.states
            .push(NULAState::new_with_transitions(1, 0, vec![self_loop]));

        let self_loop = (BitVectorMask::new([-1, -1, 1]), 3);
        nfa.states
            .push(NULAState::new_with_transitions(1, 1, vec![self_loop]));

        nfa
    }

    pub fn delta(&self, state: usize, input: &BitVector) -> Option<Vec<usize>> {
        assert!(!self.states.is_empty());
        let state = &self.states[state];

        state.delta(input)
    }

    ///`s` is the current word that's being processed for which equality
    /// (within distance 1) must be checked.
    ///
    ///`ref_str` is the reference string. This is needed to compute
    /// the bit vectors that are going to be fed to the automaton
    pub fn delta_hat(&self, s: &[u8], ref_str: &str) -> bool {
        let ref_str = ref_str.as_bytes();

        //compute all bit vectors for the ref_str
        let bitvecs: FullBitVectors;

        if s.len() < ref_str.len() {
            bitvecs = FullBitVectors::new(s);
            self.inner_delta_hat(0, 0, ref_str, &bitvecs)
        } else {
            let bitvecs = FullBitVectors::new(ref_str);
            self.inner_delta_hat(0, 0, s, &bitvecs)
        }
    }

    fn inner_delta_hat(
        &self,
        current_state: usize,
        i: usize,
        current_str: &[u8],
        ref_bitvecs: &FullBitVectors,
    ) -> bool {
        let str_len = current_str.len();
        dbg!(current_state, i);

        //+ 1 since the last bitvec to be fed is the one calculated on the terminator
        //character at the end of the string
        //
        //this is needed to have the automaton reject certain strings
        if i == str_len + 1 {
            dbg!("got to i == str_len + 1 with  current_state: {current_state}");
            return true;
        }

        let c = if i < str_len { current_str[i] } else { 0 };

        dbg!(c);

        assert!(i <= str_len);
        assert!(current_state < self.states.len());

        dbg!(ref_bitvecs.get_bitvec(c));

        let ref_bitvec = ref_bitvecs.get_bitvec(c);

        //single input
        let bitvec = BitVector::from_str(i, ref_bitvec);
        //assert_eq!(bitvec.get_positions(), [false, true, false]);

        println!("bitvec for char {}: {:?}", c as char, &bitvec);

        let tmp = current_state;
        let current_state = &self.states[current_state];
        //NOTE: as soon as an error state is reached => false is returned
        let Some(states) = current_state.delta(&bitvec) else {
            println!(
                "delta returned None starting from state: {} on bitvec: {:?}",
                tmp, &bitvec
            );
            return false;
        };

        let mut res = false;
        for state in states.iter() {
            res = res || self.inner_delta_hat(*state, i + 1, current_str, ref_bitvecs);
            //
        }

        println!();
        res
    }

    pub fn subsumes(&self, i: usize, other: usize) -> bool {
        assert!(other < self.states.len());
        let state = &self.states[i];
        state.subsumes(&self.states[other])
    }

    pub fn get_transitions(&self, i: usize) -> &[(BitVectorMask, usize)] {
        &self.states[i].transitions
    }
}
#[test]
fn delta_hat_1() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "piza";

    let word = b"piza";
    assert!(nfa.delta_hat(word, ref_word));
}

#[test]
fn delta_hat_2() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "piza";

    let word = b"pizo";
    assert!(nfa.delta_hat(word, ref_word));
}

#[test]
fn delta_hat_3() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "pita";

    let word = b"pizo";
    assert!(!nfa.delta_hat(word, ref_word));
}

#[test]
fn delta_hat_4() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "pizza";

    let word = b"pizzo";
    assert!(nfa.delta_hat(word, ref_word));
}

#[test]
fn delta_hat_5() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "ci";

    let word = b"c";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"i";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"ci";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"di";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"ca";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"cc";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"cic";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"cii";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"cia";
    assert!(nfa.delta_hat(word, ref_word));
}

#[test]
fn delta_hat_6() {
    let nfa = NULA::<Max1>::new();
    let ref_word = "hey";

    let word = b"hye";
    assert!(!nfa.delta_hat(word, ref_word));

    let word = b"heya";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"heyy";
    assert!(nfa.delta_hat(word, ref_word));

    let word = b"heyyy";
    assert!(!nfa.delta_hat(word, ref_word));

    let word = b"h";
    assert!(!nfa.delta_hat(word, ref_word));

    let word = b"heh";
    assert!(nfa.delta_hat(word, ref_word));
}
