use crate::{
    automaton::{Automaton, AutomatonState},
    fst_state::{FstState, common_prefix_index},
    register::Register,
};

type States = Vec<FstState>;

#[derive(Debug)]
pub struct Transducer {
    pub states: States,
    pub last_key: [u8; 100],
    pub last_key_len: usize,
    pub last_value: [u8; 100],
    pub last_value_len: usize,
    //cache: Dictionary,
    register: Register,
}

impl Transducer {
    pub fn new() -> Self {
        Transducer {
            states: Vec::with_capacity(128),
            last_key: [0; 100],
            last_key_len: 0,
            last_value: [0; 100],
            last_value_len: 0,
            register: Register::new(),
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

    ///process input tape starting from `state`.
    ///Returns a tuple containing the reached state (if no errors) and the last seen final state if any, otherwise None.
    pub fn delta_star_final_state(
        &self,
        state: usize,
        input_tape: &[u8],
    ) -> (Option<usize>, Option<usize>) {
        assert!(self.states.len() > 0);

        //a vector containing the output associated with the current final state
        let mut current_output: Vec<u8> = Vec::new();

        //a single output built while computing delta star. Once a final state is reached it is
        //pushed onto tmp_outputs and reset
        let mut tmp: Vec<u8> = Vec::with_capacity(input_tape.len());

        let mut current_state = state;

        //final state found on path
        let mut last_seen_final_state = None;

        for byte in input_tape.iter() {
            //compute delta and get both the reached state and the possible output on the just made transition
            if let Some((new_state, output)) = self.states[current_state].delta_with_output(*byte) {
                //move to new state
                assert!(current_state != new_state);

                //Keep track of current output
                if output.has_value() {
                    let len_before_extend = tmp.len();
                    tmp.extend_from_slice(&output.get_value_unwrapped());
                    //TODO output should never be Some but empty!
                    assert!(tmp.len() > len_before_extend);
                }

                //update state
                current_state = new_state;

                //new state is final.
                //Update the last seen final state to some(current_state)
                if self.states[current_state].is_final {
                    last_seen_final_state = Some(current_state);

                    //push tmp to tmp_outputs and reset tmp
                    //tmp_outputs.push(tmp);
                    current_output = tmp;
                    tmp = Vec::new();
                }
            }
            //transition led to an error state
            else {
                return (None, None);
            }
        }

        if last_seen_final_state.is_some() {
            assert_ne!(current_output.len(), 0);
        }
        return (Some(current_state), last_seen_final_state);
    }

    pub fn insert_pair(
        &mut self,
        key: &[u8],
        value: &[u8],
        key_prefix_len: usize,
        from_state: usize,
        seen_final_state: Option<usize>,
    ) {
        let suffix = &key[key_prefix_len..];
        self.add_suffix_to_fsa(from_state, suffix);

        //println!("suffix of key is: {}", std::str::from_utf8(suffix).unwrap());

        //insert current value associated with current key in the fst
        self.insert_value(&value, from_state, seen_final_state);

        //update buffer containing last processed key
        self.update_last_key(key);
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

        //get the state `id` of the last outgoing transition from current state
        let child_index = state.last_child().unwrap();

        if self.state_has_children(child_index) {
            self.replace_or_register(child_index);
        }

        //merge states added by previous add_suffix_to_fsa that are equivalent to the ones in the register
        if let Some(state_to_merge_with) = self.register_contains_equivalent(child_index) {
            //
            //now get a mutable reference to the state who's child is to be merged

            {
                let tmp1 = &self.states[child_index];
                let tmp2 = &self.states[state_to_merge_with];
                assert_eq!(tmp1, tmp2);

                //dbg!(&tmp1);
                //println!("equivalent to:");
                //dbg!(&tmp2);
            }

            let current_state = &mut self.states[state_index];

            //merge last child
            current_state.merge_with(state_to_merge_with);

            self.states.pop();
        } else {
            let child = &self.states[child_index];
            self.register.add_state(child, child_index);
        }
        println!();
    }

    ///look for an equivalent state.
    ///First get the bucket where state is mapped (and other states whose hashes are the same are
    ///mapped).
    ///Then linear search the bucket
    fn register_contains_equivalent(&self, state_index: usize) -> Option<usize> {
        let state = &self.states[state_index];

        //find the bucket where an equivalent state could be stored.
        //If no bucket is found then immediately return None
        let bucket = self.register.get_bucket(state)?.get_elements();

        bucket.iter().filter_map(|x| *x).find(|elem| {
            assert!(*elem < self.states.len());
            let tmp = &self.states[*elem];
            return state.eq(tmp);
        })
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
                match last_output.get_value_as_deref() {
                    Some(_) => return Some(current_state),
                    None => {
                        current_state = tmp.last_child().expect(
                            "state {current_state} expected to have child but didn't found one.",
                        );
                        continue;
                    }
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
     * insert value in the fst. last_state_index is the index of the last state reachable by invoking
     * delta* on prefix of key.
     *
     * last_seen_final_state is an option containing the last reached final state during the
     * computation of delta star given the common prefix of the current key.
     *
     * output is the output obtained by starting from q0 and processing delta star of the common
     * prefix of the current key
     *
     * First walk the path from initial state to last_state. If no output is found then adding the value is as simple as
     * Taking the last outgoing transition from last_state and setting it's output to value
     *
     * In some cases the path from q0 to last_state already contains some output that may (or may not) share
     * a prefix with value => in that case only the common prefix should be kept. The rest must be
     * propagated forward to all transitions (but the last one) of last_state.
     */
    pub fn insert_value(
        &mut self,
        value: &[u8],
        last_state_index: usize,
        last_seen_final_state: Option<usize>,
    ) {
        assert!(value.len() > 0);

        //final state found on path.
        //Start from that final_state
        if let Some(final_state_index) = last_seen_final_state {
            self.diff_propagation(final_state_index, last_state_index, value);
        }
        //no final state on path to last state
        else {
            //output value present in the given path
            if let Some(first_state_with_output) = self.first_output_on_path_to(last_state_index) {
                //

                self.diff_propagation(first_state_with_output, last_state_index, value);

                //let prefix_on_path_len = common_prefix_index(&output_on_path, value);

                //add value[prefix_len..] to last output of last state
                //let last_state = &mut self.states[last_state_index];
                //last_state.set_last_output(&value[prefix_on_path_len..]);
            } else {
                //add value to last output of last state
                let last_state = &mut self.states[last_state_index];
                last_state.set_last_output(&value);
            }
        }
    }

    /*
     * propagate diff on the entire path from next_state to last_state.
     * Returns
     */
    fn diff_propagation(
        &mut self,
        first_state_index: usize,
        last_state_index: usize,
        value: &[u8],
        //) -> Vec<u8> {
    ) {
        assert!(first_state_index <= last_state_index);

        //get a mut reference to next state (last child)
        let mut current_state_index = first_state_index;

        let mut diff = None;

        //this buff will be used to keep track of what output is encountered by walking the path
        //from first_state_index up to last_state.
        //let mut buff: Vec<u8> = Vec::with_capacity(value.len());

        let mut trimmed_characters = 0;

        let mut done_with_prefix = false;

        //println!("\ndiff propagation with value: {}", bytes_to_utf8(value));

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
                current_state.propagate_new_prefix(diff.as_deref());
            } else {
                if current_state_index != first_state_index {
                    unreachable!("diff_propagation should never be called on a final state");
                }
            }

            //Take last output.
            let current_last_output = current_state.get_last_output();

            //find common prefix between output and value
            if let Some(output) = current_last_output.get_value_as_deref() {
                //output on last edge has some value. It is NOT None

                let prefix_up_to: usize;
                if !done_with_prefix {
                    prefix_up_to = common_prefix_index(output, &value[trimmed_characters..]);
                } else {
                    prefix_up_to = 0;
                }

                trimmed_characters += prefix_up_to;

                let output_len = output.len();

                //whenever the first diff is found we stop computing prefix between value and edge
                //output.
                if prefix_up_to < output_len {
                    done_with_prefix = true;
                }

                if prefix_up_to > 0 {
                    diff = current_state.truncate_last_output(prefix_up_to);
                } else {
                    diff = current_state.clear_last_output();
                }

                //no output on edge
            } else {
                diff = None;
            }

            //update current state to last_child
            current_state_index = current_state.last_child().unwrap();
        }

        //Now propagate diff to ALL edges EXCEPT last one. Diff will be prepended to their
        //outputs

        let last_state = &mut self.states[last_state_index];
        last_state.propagate_prefix_exc_last(&diff);

        //add value to last output of last_state
        last_state.set_last_output(&value[trimmed_characters..]);
    }

    /*
     * replace current key `last_key` with `new_key`
     * the array should first be filled with 0s
     */
    pub fn update_last_key(&mut self, new_key: &[u8]) {
        let word_len = new_key.len();
        self.clear_last_key();

        self.last_key_len = word_len;
        self.last_key[..word_len].copy_from_slice(&new_key[..]);
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

    //process input tape (key) and return output collected along the path to a final state.
    //Returns None if no final state is reached or an error state is reached
    fn get(&self, input_tape: &[u8]) -> Option<Vec<u8>> {
        assert!(self.states.len() > 0);

        let mut current_state = 0;

        let mut tmp: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();

        for byte in input_tape.iter() {
            let state = &self.states[current_state];

            if state.is_final {
                output = tmp;
                tmp = Vec::new();
            }

            //there exists a transition from state with byte
            if let Some(transition) = state.delta_with_output(*byte) {
                let (new_state, tmp_value) = transition;
                //move to new state
                assert!(current_state != new_state);
                current_state = new_state;

                //process output of transition.
                //Not all transitions have output associated with them
                if let Some(v) = tmp_value.get_value_as_deref() {
                    //if a value exists => extend final (possible) output with it
                    tmp.extend(v);
                }
            }
            //no outgoing transition from state with byte
            else {
                return None;
            }
        }

        //if a final state was reached => return the value
        if self.is_final_state(current_state) {
            output = tmp;
            return Some(output);
        }
        //else return None => input_tape wasn't a valid key
        else {
            return None;
        }
    }
}

pub enum FstError {}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {}
