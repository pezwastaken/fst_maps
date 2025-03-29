use crate::fst_state::FstState;
use core::result::Result;

#[derive(Debug)]
pub struct Dictionary {
    fst_states: Vec<usize>,
}

//TODO should also use outputs?
//are both states in the same equivalence class
impl PartialEq for FstState {
    fn eq(&self, other_state: &Self) -> bool {
        if self.is_final != other_state.is_final {
            return false;
        }

        match self.transitions.len() != other_state.transitions.len() {
            true => return false,
            false => (),
        }

        let self_transitions = &self.transitions;
        let other_state_transitions = &other_state.transitions;

        //actually check transitions
        //compare char of transition 1 with char of transition 2
        //same thing with destination states
        let all_transitions_equal = self_transitions
            .iter()
            .zip(other_state_transitions)
            .all(|((char1, state1), (char2, state2))| (*char1 == *char2) && (*state1 == *state2));

        return all_transitions_equal;
    }
}

impl Eq for FstState {}

/*
*
*
* Dictionary
*
*
*
*/

impl Dictionary {
    //new dict with capacity
    fn new() -> Self {
        Dictionary {
            fst_states: Vec::with_capacity(128),
        }
    }
}
//
//    fn is_member(&self, state: &State) -> Option<&State>{
//        let equivalent_state = self.fst_states.get
//    }
//
//    //start tracking state
//    //TODO make it circular
//    //right now it's size keeps increasing
//    fn insert(&mut self, state: usize) {
//        self.fst_states.insert(state);
//    }
//}

/*
*
*
* Transducer
*
*
*
*/

#[derive(Debug)]
pub struct Transducer {
    states: Vec<FstState>,
    last_str: [u8; 100],
    last_str_len: usize,
    //cache: Dictionary,
    cache: Vec<usize>,
}

impl Transducer {
    fn new() -> Self {
        Transducer {
            states: Vec::with_capacity(128),
            last_str: [0; 100],
            last_str_len: 0,
            cache: Vec::with_capacity(128),
        }
    }

    fn push_zinit_state(&mut self) {
        self.states.push(FstState {
            is_final: false,
            transitions: Vec::with_capacity(10),
            outputs: Vec::with_capacity(10),
            state_output: FstStateOutput::None,
        });
    }

    //get the index of the last prefix character inside y
    pub fn common_prefix(&self, y: &str) -> usize {
        let y_bytes = y.as_bytes();

        //TODO SIMD it using chunk_exact
        let prefix_len = self
            .last_str
            .iter()
            .zip(y_bytes)
            .take_while(|(x, y)| x == y)
            .count();

        assert!(prefix_len <= y.len());
        return prefix_len;
    }

    /*
     * TODO
     * this function and insert_in_cache should be methods of dictionary.
     * they are here for simplicity right now
     *
     */
    //check whether state of index `state` is has an equivalent one in cache
    //if so return it
    fn is_cached(&mut self, state: usize) -> Option<usize> {
        assert!(state < self.states.len());
        let other_state = &self.states[state];

        for cached_state_index in self.cache.iter() {
            let cached_state = &self.states[*cached_state_index];

            if cached_state.eq(other_state) {
                return Some(*cached_state_index);
            }
        }

        return None;
    }

    fn insert_in_cache(&mut self, state: usize) {
        assert!(state < self.states.len());
        self.cache.push(state);
    }
    /*
     * TODO END of functions that should be moved
     */

    //return an equivalent state from the cache
    //if not present push it to the dictionary
    fn find_minimized(&mut self, state_index: usize) -> usize {
        assert!(state_index < self.states.len());

        match self.is_cached(state_index) {
            Some(equivalent_state_index) => return equivalent_state_index,
            None => {
                self.insert_in_cache(state_index);
                return state_index;
            }
        }
    }

    //check current word against previous word by comparing each byte
    fn is_word_equal_to_previous(&self, current_word: &[u8]) -> bool {
        if self.last_str[0] == 0 {
            assert!(self.last_str_len == 0);
        }

        //lengths do not match
        if self.last_str_len != current_word.len() {
            return false;
        }
        //same length => can use .zip
        let previous_word = &self.last_str[0..self.last_str_len];

        //check for equality between slices of u8
        let res = previous_word
            .iter()
            .zip(current_word)
            .all(|(x, y)| *x == *y);

        return res;
    }
}

fn init_temp_states(temp_states: &mut Vec<FstState>) {
    for i in 0..N_TEMP_STATES {
        temp_states.push(FstState {
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
            state_output: FstStateOutput::None,
        });
    }
}

//initialize tail states for the current word
fn init_tail_states(temp_states: &mut [FstState], current_word: &[u8], prefix_len_plus_1: usize) {
    let i = prefix_len_plus_1;

    assert!(i <= temp_states.len());
    assert!(current_word.len() <= temp_states.len());

    let word_len = current_word.len();

    for i in prefix_len_plus_1..word_len + 1 {
        assert!(i > 0);
        let state = &mut temp_states[i - 1];

        state.clear_state();

        let current_char: u8 = current_word[i - 1];
        state.add_transition(current_char, i);
    }
}

const N_TEMP_STATES: usize = 256;
pub enum FstError {}

pub fn build_transducer_from_collection(
    collection: Vec<(String, String)>,
) -> Result<Transducer, FstError> {
    assert!(collection.len() > 0);

    let mut transducer = Transducer::new();
    //TODO should be fixed size?
    let mut temp_states: Vec<FstState> = Vec::with_capacity(N_TEMP_STATES);
    init_temp_states(&mut temp_states);

    assert_eq!(temp_states[0].is_final, false);
    let initial_state = 0;

    //let mut previous_str = "";

    transducer.push_zinit_state();

    assert_eq!(transducer.states.len(), 1);

    for key_value_pair in collection.iter() {
        let (key, value) = key_value_pair;

        let i: usize = 1;

        //len of common prefix
        //
        let prefix_len = transducer.common_prefix(&key);
        let prefix = &key[0..prefix_len];

        //TODO
        //minimize states from the suffix of previous word
        assert!(prefix_len <= transducer.last_str_len);
        let previous_suffix = &transducer.last_str[prefix_len..transducer.last_str_len];

        //iter through each byte (in reverse)
        //should also have `i`
        for _byte in previous_suffix.iter().rev() {
            //TODO set transition temp_states
        }

        //initialize tail states for current word
        let key_as_bytes = key.as_bytes();
        let prefix_len_plus_one = prefix_len + 1;
        init_tail_states(&mut temp_states, key_as_bytes, prefix_len_plus_one);

        //current word is different from the previous one
        //new states must be added for the suffix of the new word
        if !transducer.is_word_equal_to_previous(key_as_bytes) {
            let last_state_index = key_as_bytes.len();
            let final_tail_state = &temp_states[last_state_index];
            assert_eq!(final_tail_state.has_children(), false);
        }
    }

    return Ok(transducer);
}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {

    use crate::fst::FstStateOutput;
    use crate::fst::init_tail_states;
    use crate::fst::{FstState, N_TEMP_STATES};

    #[test]
    fn init_tail_states_normal() {
        let mut temp_states: Vec<FstState> = Vec::with_capacity(N_TEMP_STATES);
        for i in 0..N_TEMP_STATES {
            temp_states.push(FstState {
                is_final: false,
                transitions: Vec::new(),
                outputs: Vec::new(),
                state_output: FstStateOutput::None,
            });
        }

        let test_string = b"topo";
        init_tail_states(&mut temp_states, test_string, 1);

        for i in 0..test_string.len() {
            assert!(&temp_states[i].has_children());
            assert!(temp_states[i].transitions[0] == (test_string[i], i + 1));
            //println!("{:?}", temp_states[i]);
        }
    }

    #[test]
    fn init_tail_states_empty() {
        let mut temp_states: Vec<FstState> = Vec::with_capacity(N_TEMP_STATES);
        for i in 0..N_TEMP_STATES {
            temp_states.push(FstState {
                is_final: false,
                transitions: Vec::new(),
                outputs: Vec::new(),
                state_output: FstStateOutput::None,
            });
        }

        let test_string = b"";
        init_tail_states(&mut temp_states, test_string, 1);

        assert!(!temp_states[0].has_children());
        assert!(!temp_states[1].has_children());
    }
}
