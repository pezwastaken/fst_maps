use std::{
    collections::{BTreeSet, HashSet},
    marker::PhantomData,
};

use crate::{
    bit_vector::{BitVector, BitVectorMask, FullBitVectors},
    nula::{Max1, NULA, NULADistance},
};

#[derive(Debug)]
struct DULAState {
    id: u64,

    //a dula transition is a pair:
    //  bit_vector_mask
    //  next_state
    transitions: Vec<(BitVectorMask, usize)>,
}

impl DULAState {
    fn new_no_transitions(i: u64) -> Self {
        Self {
            id: i,
            transitions: Vec::with_capacity(2),
        }
    }

    fn new_with_transitions(i: u64, t: Vec<(BitVectorMask, usize)>) -> Self {
        for (i1, transition) in t.iter().enumerate() {
            assert!(t.iter().enumerate().all(|(i2, v)| {
                if i1 != i2 {
                    return !transition.eq(v);
                } else {
                    return true;
                }
            }));
        }

        Self {
            id: i,
            transitions: t,
        }
    }

    ///It's an DFA so delta returns a single state
    fn delta(&self, bit_vector: &BitVector) -> Option<usize> {
        for t in self.transitions.iter() {
            if t.0.mask_equal(bit_vector) {
                return Some(t.1);
                // res.push(t.1);
            }
        }

        // if res.len() > 0 {
        //     return Some(res);
        // }
        None
    }
}

#[derive(Debug)]
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
        let dfa = Self {
            states: Vec::with_capacity(8),
            _marker: PhantomData,
        };
        dfa
    }

    //Starts from state 0
    pub fn delta_star(&self, s: &[u8], ref_str: &str) -> bool {
        let ref_str = ref_str.as_bytes();

        //compute all bit vectors for the ref_str
        // let bitvecs: FullBitVectors;

        // if s.len() < ref_str.len() {
        //     bitvecs = FullBitVectors::new(s);
        //     self.inner_delta_star(ref_str, &bitvecs)
        // }
        let bitvecs = FullBitVectors::new(ref_str);
        self.inner_delta_star(s, &bitvecs)
    }

    fn inner_delta_star(&self, current_str: &[u8], ref_bitvecs: &FullBitVectors) -> bool {
        let mut current_state: usize = 0;

        //process str
        for (i, c) in current_str.iter().enumerate() {
            let ref_bitvec = ref_bitvecs.get_bitvec(*c);
            let input_bitvec = BitVector::from_str(i, ref_bitvec);

            let reached_state = self.delta(current_state, &input_bitvec);

            match reached_state {
                Some(to_state) => current_state = to_state,
                None => return false,
            }
        }

        //there are still 2 terminators left to process (at the end of current_str)
        //NOTE: this is hardcoded since there's only k = 1

        //FIX: horrendous way to get the length of ref_str
        let ref_len = ref_bitvecs.get_bitvec(0).len() - 2;

        for i in 0..2 {
            let i = i + current_str.len();

            let input_bitvec = BitVector::terminator_bitvec(i, ref_len);

            let reached_state = self.delta(current_state, &input_bitvec);
            dbg!(reached_state);

            match reached_state {
                Some(to_state) => {
                    current_state = to_state;
                }
                None => return false,
            }
        }
        true
    }

    //each state in the dula is actually a combination of nula states
    fn add_state(&mut self, set: &SetOfStates) -> u64 {
        let id = hash(set);

        let transitions: Vec<(BitVectorMask, usize)> = Vec::with_capacity(3);

        let new_state = DULAState::new_with_transitions(id, transitions);
        self.states.push(new_state);
        id
    }

    fn contains_state_with_id(&self, h: u64) -> Option<usize> {
        self.states.iter().position(|v| v.id == h)
    }

    fn set_delta_for_id(&mut self, t_id: u64, bitvec: &BitVector, to_state: usize) {
        let state = self.states.iter_mut().find(|x| x.id == t_id).unwrap();
        state
            .transitions
            .push((BitVectorMask::from(bitvec), to_state));
    }

    pub fn delta(&self, current_state: usize, input_bitvec: &BitVector) -> Option<usize> {
        assert!(current_state < self.states.len());
        let state = &self.states[current_state];

        state.delta(input_bitvec)
    }
}

//possible bit vectors for distance 1
const FIXED_BIT_VECTORS: [BitVector; 8] = [
    BitVector::from_bool_arr([false, false, false]),
    BitVector::from_bool_arr([false, false, true]),
    BitVector::from_bool_arr([false, true, false]),
    BitVector::from_bool_arr([true, false, false]),
    BitVector::from_bool_arr([false, true, true]),
    BitVector::from_bool_arr([true, false, true]),
    BitVector::from_bool_arr([true, true, false]),
    BitVector::from_bool_arr([true, true, true]),
];

#[derive(PartialOrd, PartialEq, Ord, Eq)]
struct SetOfStates {
    set: BTreeSet<usize>,
}

impl SetOfStates {
    fn new() -> Self {
        Self {
            set: BTreeSet::new(),
        }
    }

    fn new_with_state(i: usize) -> Self {
        let mut set = BTreeSet::new();
        set.insert(i);
        Self { set }
    }

    fn insert(&mut self, i: usize) {
        self.set.insert(i);
    }

    fn contains(&self, s: usize) -> bool {
        // self.set.contains(s)
        self.set.contains(&s)
    }

    fn reachable_states(&self, nula: &NULA<Max1>, input: &BitVector) -> HashSet<usize> {
        let mut reachable_states = HashSet::<usize>::new();

        //compute delta starting from each state and add elements to the set of
        // reachable states
        for state in self.set.iter() {
            match nula.delta(*state, input) {
                Some(states) => {
                    reachable_states.extend(states.into_iter());
                }
                None => {}
            }
        }

        reachable_states
    }
}

pub fn build_dula(nula: &NULA<Max1>) -> DULA<Max1> {
    //initialized with one state (0, 0) => 0 errors, 0th lane
    let mut dula = DULA::<Max1>::new();
    let h = dula.add_state(&SetOfStates::new_with_state(0));

    let mut unmarked_states: BTreeSet<(u64, SetOfStates)> = BTreeSet::new();
    unmarked_states.insert((h, SetOfStates::new_with_state(0)));

    loop {
        //while there exists an unmarked state
        let Some(t) = unmarked_states.pop_last() else {
            break;
        };

        for bitvec in FIXED_BIT_VECTORS.iter() {
            let t_id = t.0;
            let t_set_of_states = &t.1;

            let s = t_set_of_states.reachable_states(nula, bitvec);

            let s_prime = reduce(nula, &s);
            if s_prime.set.len() == 0 {
                continue;
            }
            let s_prime_hash = hash(&s_prime);

            match dula.contains_state_with_id(s_prime_hash) {
                Some(s_prime_index) => dula.set_delta_for_id(t_id, &bitvec, s_prime_index),
                None => {
                    let s_prime_index = dula.states.len();

                    dula.add_state(&s_prime);
                    unmarked_states.insert((s_prime_hash, s_prime));

                    dula.set_delta_for_id(t_id, &bitvec, s_prime_index);
                }
            }
        }
    }

    dula
}

fn reduce(nula: &NULA<Max1>, states: &HashSet<usize>) -> SetOfStates {
    let mut result = SetOfStates::new();
    for state in states.iter() {
        let not_subsumed = states.iter().all(|v| !nula.subsumes(*v, *state));
        if not_subsumed {
            result.insert(*state);
        }
    }
    result
}

//simple fnv-1a hash to check equality
fn hash(states: &SetOfStates) -> u64 {
    const FNV_PRIME: u64 = 1099511628211;

    let mut h: u64 = 14695981039346656037;

    for elem in states.set.iter() {
        h = h ^ (*elem as u64);
        h = h.wrapping_mul(FNV_PRIME);
    }

    h
}

#[test]
fn test_build_dula() {
    let nfa = NULA::<Max1>::new();

    let _ = build_dula(&nfa);
}

#[test]
fn delta_star_1() {
    let nfa = NULA::<Max1>::new();

    let dfa = build_dula(&nfa);

    let ref_word = "piza";
    let word = b"piza";

    assert!(dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_2() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);

    let ref_word = "piza";
    let word = b"pizo";
    assert!(dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_3() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);
    let ref_word = "pita";

    let word = b"pizo";
    assert!(!dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_4() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);
    let ref_word = "pizza";

    let word = b"pizzo";
    assert!(dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_5() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);

    let ref_word = "ci";

    let word = b"c";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"i";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"ci";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"di";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"ca";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"cc";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"cic";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"cii";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"cia";
    assert!(dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_6() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);

    let ref_word = "hey";

    let word = b"e";
    assert!(!dfa.delta_star(word, ref_word));

    let word = b"hye";
    assert!(!dfa.delta_star(word, ref_word));

    let word = b"heya";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"heyy";
    assert!(dfa.delta_star(word, ref_word));

    let word = b"heyyy";
    assert!(!dfa.delta_star(word, ref_word));

    let word = b"h";
    assert!(!dfa.delta_star(word, ref_word));

    let word = b"heh";
    assert!(dfa.delta_star(word, ref_word));
}

#[test]
fn delta_star_7() {
    let nfa = NULA::<Max1>::new();
    let dfa = build_dula(&nfa);

    let ref_word = "pangolino";

    let word = b"pagolinon";
    assert!(!dfa.delta_star(word, ref_word));
}
