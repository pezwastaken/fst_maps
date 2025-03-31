use crate::{
    automaton::{Automaton, AutomatonState},
    fst_state::{FstState, common_prefix_index},
    register::{REGISTER_CAPACITY, REGISTER_COUNT_MASK, Register},
};

#[derive(Debug)]
pub struct Transducer {
    pub states: Vec<FstState>,
    pub last_key: [u8; 100],
    pub last_key_len: usize,
    pub last_value: [u8; 100],
    pub last_value_len: usize,
    //cache: Dictionary,
    pub register: Register,
}

impl Transducer {
    pub fn new() -> Self {
        Transducer {
            states: Vec::with_capacity(128),
            last_key: [0; 100],
            last_key_len: 0,
            last_value: [0; 100],
            last_value_len: 0,
            register: Register {
                elements: [0; REGISTER_CAPACITY],
                count: 0,
            },
        }
    }

    pub fn push_zinit_state(&mut self) {
        self.states.push(FstState {
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
        });
    }

    pub fn state_has_children(&self, state_index: usize) -> bool {
        return self.states[state_index].has_children();
    }

    //start keeping track of suffix y in the fsa
    //new states will originate from `last_state`
    //a suffix = "" will do nothing
    pub fn add_suffix_to_fsa(&mut self, from_state: usize, y: &[u8]) {
        assert!(y.len() <= 100);

        let string_len = y.len();

        let initial_number_of_states = self.states.len();

        let mut current_state = from_state;

        for (i, byte) in y.iter().enumerate() {
            //get the current state and add a new transition to it
            let new_state_id = self.states.len();

            self.states[current_state].add_transition(*byte, new_state_id);

            //if last loop iteration the state is to be marked as final
            let is_state_final = i == string_len - 1;

            let new_state: FstState = FstState {
                is_final: is_state_final,
                transitions: Vec::new(),
                outputs: Vec::new(),
            };

            assert!(new_state.transitions.is_empty());
            self.states.push(new_state);

            current_state = new_state_id;
        }

        assert!(initial_number_of_states <= self.states.len());
    }

    //assume state has children
    pub fn replace_or_register(&mut self, state_index: usize) {
        let state = &self.states[state_index];
        assert!(state.has_children());

        //get the state `id` of the last outgoing transition from current state
        let child_index = state.last_child().unwrap();

        //println!("last child index is {}", child_index);

        if self.state_has_children(child_index) {
            self.replace_or_register(child_index);
        }

        //merge states added by previous add_suffix_to_fsa that are equivalent to the ones in the register
        if let Some(state_to_merge_with) = self.register_contains_equivalent(child_index) {
            //
            //now get a mutable reference to the state who's child is to be merged
            let current_state = &mut self.states[state_index];

            //merge last child
            current_state.merge_with(state_to_merge_with);

            self.states.pop();
        } else {
            self.register.add_state(child_index);
        }
    }

    fn register_contains_equivalent(&self, state_index: usize) -> Option<usize> {
        let cached_states = &self.register.elements;

        let state = &self.states[state_index];

        let register_len: usize = self.register.count & REGISTER_COUNT_MASK;

        for i in 0..register_len {
            let cached_state_index = cached_states.get(i).expect(&format!(
                "last word was {}, state_index is {state_index}.\nnumber of cached states: {}, accessing index {} of register",
                &std::str::from_utf8(&self.last_key[0..self.last_key_len]).unwrap(), self.register.count, i
            ));

            let cached_state = &self.states.get(*cached_state_index).expect(&format!(
                "last word was {}, state_index is {state_index}.\nnumber of cached states: {}, accessed index {} of register and got state {}\nnumber of states in fsa is {}",
                &std::str::from_utf8(&self.last_key[0..self.last_key_len]).unwrap(),
                self.register.count,
                i,
                *cached_state_index,
                self.states.len()
            ));

            if state.eq(cached_state) {
                return Some(*cached_state_index);
            }
        }

        return None;
    }

    #[inline]
    pub fn clear_last_key(&mut self) {
        self.last_key[0..self.last_key_len].fill(0);
    }

    //return the index of the first state (starting from 0) which has some value on last output
    //stop and return none if last_state is reached
    fn first_output_on_path_to(&self, last_state: usize) -> Option<usize> {
        let mut current_state = 0;

        //keep going until last_state is reached
        while current_state != last_state {
            //get reference to current state
            let tmp = self
                .states
                .get(current_state)
                .expect("states {current_state} out of bounds.");

            //if state has children then it must have outgoing transitions and possibly output
            //values associated with them
            if tmp.has_children() {
                let last_output = tmp.get_last_output();

                //if no output on last transition => continue searching on `last path`
                if last_output == None {
                    current_state = tmp.last_child().expect(
                        "state {current_state} expected to have child but didn't found one.",
                    );
                    continue;
                }
                //transition has output
                //stop there
                else {
                    return Some(current_state);
                }
            //state doesn't have children
            } else {
                panic!(
                    "state {current_state} doesn't have children but is ancestor of {last_state}"
                );
            }
        }

        return None;
    }

    /*
     *insert value in the fst. last_state_index is the index of the last state reachable by invoking
     *delta* on prefix of key
     *
     * First walk the path from initial state to last_state. If no output is found then adding the value is as simple as
     * Taking the last outgoing transition from last_state and setting it's output to value
     *
     * In some cases the path from q0 to last_state already contains some output that may (or may not) share
     * a prefix with value => in that case only the common prefix should be kept. The rest must be
     * propagated forward to all transitions (but the last one) of last_state.
     */
    pub fn insert_value(&mut self, value: &[u8], last_state_index: usize) {
        assert!(value.len() > 0);

        //not sufficent. You may have to use subtraction on values even though the suffix doesn't
        //start from a final state.
        if self.states[last_state_index].is_final {
            todo!();
        }

        //output value present in the given path
        if let Some(first_state_with_output) = self.first_output_on_path_to(last_state_index) {
            //
            let output_on_path =
                self.diff_propagation(first_state_with_output, value, last_state_index);

            let prefix_on_path_len = common_prefix_index(&output_on_path, value);

            //add value[prefix_len..] to last output of last state
            let last_state = &mut self.states[last_state_index];
            last_state.set_last_output(&value[prefix_on_path_len..]);
        } else {
            //add value to last output of last state
            let last_state = &mut self.states[last_state_index];
            last_state.set_last_output(&value);
        }
    }

    /*
     * propagate diff on the entire path from next_state to last_state.
     * Returns
     */
    fn diff_propagation(
        &mut self,
        first_state_index: usize,
        value: &[u8],
        last_state_index: usize,
    ) -> Vec<u8> {
        assert!(first_state_index <= last_state_index);

        //get a mut reference to next state (last child)
        let mut current_state_index = first_state_index;

        let mut diff = None;

        //this buff will be used to keep track of what output is encountered by walking the path
        //from first_state_index up to last_state.
        let mut buff: Vec<u8> = Vec::with_capacity(value.len());

        //while loop until current state != last_state
        while current_state_index != last_state_index {
            //
            //propagate diff to all outgoing edges. Diff will be prepended to their outputs
            let current_state = &mut self.states[current_state_index];
            assert!(current_state.has_children());

            //propagate diff from previous iteration if and only if the current state is not final.
            //A final state means that output on previous edges constitutes its total output
            //Propagating would mean modifying the value associated with that final state (key)
            if !current_state.is_final {
                current_state.propagate_new_prefix(&diff);
            }

            //Take last output.
            let current_last_output = current_state.get_last_output();

            //find common prefix between output and value
            if let Some(output) = current_last_output {
                //output on last edge has some value. It is NOT None

                let prefix_up_to = common_prefix_index(output, value);
                diff = current_state.truncate_last_output(prefix_up_to);
            } else {
                diff = None;
            }

            //update current state to last_child
            current_state_index = current_state.last_child().unwrap();
            if let Some(s) = current_state.get_last_output() {
                buff.append(&mut s.to_vec());
            }
        }

        //Now propagate diff to ALL edges EXCEPT last one. Diff will be prepended to their
        //outputs

        let last_state = &mut self.states[last_state_index];
        last_state.propagate_prefix_exc_last(&diff);

        //add value to last output of last_state
        last_state.set_last_output(value);

        return buff;
    }

    /*
     * replace current key `last_key` with `new_key`
     * the array should first be filled with 0s
     */
    pub fn update_last_key(&mut self, new_key: &[u8]) {
        let word_len = new_key.len();
        self.clear_last_key();

        self.last_key_len = word_len;
        self.last_key[..word_len].copy_from_slice(&new_key[..word_len]);
    }
}

//automaton implementation
impl Automaton for Transducer {
    fn is_final_state(&self, state: usize) -> bool {
        self.states
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    fn delta_star(&self, state: usize, input_tape: &[u8]) -> Option<usize> {
        assert!(self.states.len() > 0);
        let mut current_state = state;

        for byte in input_tape.iter() {
            if let Some(new_state) = self.states[current_state].delta(*byte) {
                //move to new state
                assert!(current_state != new_state);
                current_state = new_state;
            } else {
                return None;
            }
        }

        return Some(current_state);
    }

    #[inline]
    fn delta_star_from_initial_state(&self, input_tape: &[u8]) -> Option<usize> {
        self.delta_star(0, input_tape)
    }

    fn delta_star_with_output(&self, state: usize, input_tape: &[u8]) -> Option<usize> {
        todo!()
    }
}

pub enum FstError {}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {}
