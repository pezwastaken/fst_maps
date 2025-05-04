use bincode::error::EncodeError;

use crate::{
    automaton::{Automaton, AutomatonState},
    error::MyResult,
    fst_state::{FstState, common_prefix_index},
    raw_writer::RawWriter,
    register::Register,
};

type States = Vec<FstState>;

#[derive(Debug)]
pub struct Transducer<'a> {
    //TODO REMOVE this
    //It's just to output the dot graph
    pub states: States,
    //TODO
    //vector of temporary states (not yet frozen)
    //This means they are still not in the register. The initial state will always be part of the
    //temp_states vec
    pub temp_states: States,
    pub last_key: [u8; 100],
    pub last_key_len: usize,
    pub last_value: [u8; 100],
    pub last_value_len: usize,
    register: Register<FstState>,
    writer: RawWriter<'a>,
}

impl<'a> Transducer<'a> {
    pub fn new(to_file: &'a str) -> Self {
        if let Ok(wr) = RawWriter::new(to_file) {
            Transducer {
                states: Vec::with_capacity(128),
                temp_states: Vec::with_capacity(128),
                last_key: [0; 100],
                last_key_len: 0,
                last_value: [0; 100],
                last_value_len: 0,
                register: Register::new(),
                writer: wr,
            }
        } else {
            panic!("couldn't create an fst mapped to the given file");
        }
    }

    pub fn push_zinit_state(&mut self) {
        self.temp_states.push(FstState {
            //TODO this is just a placeholder for the initial state
            address: 99999,
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
        });
    }

    ///This should always just follow the last path (states in temp_states vec)
    ///
    ///process input tape starting from `state`.
    ///Returns a tuple containing the reached state (if no errors) and the last seen final state if any, otherwise None.
    pub fn delta_star_final_state(&self, input_tape: &[u8]) -> (Option<usize>, Option<usize>) {
        assert!(!self.temp_states.is_empty());

        //a vector containing the output associated with the current final state
        let mut current_output: Vec<u8> = Vec::new();

        //a single output built while computing delta star. Once a final state is reached it is
        //pushed onto tmp_outputs and reset
        let mut tmp: Vec<u8> = Vec::with_capacity(input_tape.len());

        let mut current_state = 0;

        //final state found on path
        let mut last_seen_final_state = None;

        for (i, byte) in input_tape.iter().enumerate() {
            //compute delta and get both the reached state and the possible output on the just made transition

            current_state = i;

            if let Some((_new_state, output)) = self.temp_states[i].delta_with_output(*byte) {
                //Keep track of current output
                if output.has_value() {
                    let len_before_extend = tmp.len();
                    tmp.extend_from_slice(&output.get_value_unwrapped());
                    //TODO output should never be Some but empty!
                    assert!(tmp.len() > len_before_extend);
                }

                //new state is final.
                //Update the last seen final state to some(current_state)
                if self.temp_states[i].is_final {
                    last_seen_final_state = Some(i);

                    //push tmp to tmp_outputs and reset tmp
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
        //println!("called insert pair with from_state: {from_state}");
        //dbg!(seen_final_state);

        let suffix = &key[key_prefix_len..];
        self.add_suffix_to_fsa(from_state, suffix);

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

        let initial_number_of_states = self.temp_states.len();

        let mut current_state = from_state;

        for (i, byte) in y.iter().enumerate() {
            //get the current state and add a new transition to it
            let new_state_address = self.temp_states.len();

            //0 means that the address to which this transition is pointing will be set later
            self.temp_states[current_state].add_transition(*byte, 1);
            //self.temp_states[current_state].add_transition(*byte, new_state_address);

            //if last loop iteration the state is to be marked as final
            let is_state_final = i == string_len - 1;

            let new_state: FstState = FstState {
                //address: self.states.len(),
                address: 1,
                is_final: is_state_final,
                transitions: Vec::new(),
                outputs: Vec::new(),
            };

            assert!(new_state.transitions.is_empty());
            self.temp_states.push(new_state);

            assert!(self.temp_states.len() > 1);

            current_state = new_state_address;
        }

        assert!(initial_number_of_states <= self.temp_states.len());
    }

    //assume state has children
    pub fn replace_or_register(&mut self, i: usize) {
        //get the state `address` of the last outgoing transition from current state

        if self.temp_states[i + 1].has_children() {
            self.replace_or_register(i + 1);
        }

        //merge states added by previous add_suffix_to_fsa that are equivalent to the ones in the register
        if let Some(state_to_merge_with) = self.register_contains_equivalent(i + 1) {
            //now get a mutable reference to the state whose child is to be merged
            let current_state = &mut self.temp_states[i];

            //merge last child
            current_state.merge_with(state_to_merge_with);

            self.temp_states.pop();
        } else {
            //TODO handle error
            //Should return it

            match self.add_to_register(i + 1) {
                Ok(written_bytes) => {
                    let current_state = &mut self.temp_states[i];
                    current_state.set_last_transition_child(written_bytes);
                }

                Err(e) => panic!("error during add to register: {e:?}"),
            }
        }
    }

    ///look for an equivalent state in the register.
    fn register_contains_equivalent(&self, state_index: usize) -> Option<usize> {
        let state = &self.temp_states[state_index];

        self.register.get_equivalent(state)
    }

    fn add_to_register(&mut self, i: usize) -> Result<usize, EncodeError> {
        //let address = self.persist_state(i)?;
        let address = self.writer.get_written_bytes();

        let state = &mut self.temp_states[i];
        state.address = address;

        //this is just temporary
        self.states.push(state.clone());

        //TODO move (not copy) the state to the register
        self.register.add_state(state.clone());

        //moving a state to the register means that we have effectively frozen that state.
        //A frozen state will never be 'touched' again.
        //Thus, we can write it to file.
        //self.writer.write_state(state)?;

        self.persist_state(i)?;
        self.temp_states.pop();

        Ok(address)
    }

    ///Writes the given FstState and then its length (in bytes).
    ///Returns the address to the start of the state len.
    pub fn persist_state(&mut self, i: usize) -> Result<(), EncodeError> {
        let state = &self.temp_states[i];

        let _ = self.writer.write_state::<FstState>(state)?;

        //this is the address that is to be returned
        Ok(())
    }

    pub fn persist_initial_state(&mut self) -> MyResult<()> {
        let state = &self.temp_states[0];

        let wrote = self.writer.write_state::<FstState>(&state)?;

        //this is the address that is to be returned
        //let total_written_bytes = self.writer.get_written_bytes();

        match self.writer.write_fixed_int(wrote) {
            Ok(_) => {
                self.writer.flush()?;
                Ok(())
            }
            Err(e) => Err(e.into()),
        }

        ////write initial state
        //self.persist_state(0)
        //    .expect("couldn't persist initial state to file");
        //
        //dbg!(&self.temp_states[0]);
        //
        ////write its len (in terms of bytes) to file.
        //match self.writer.write_fixed_int(wrote) {
        //    Ok(_) => {}
        //    Err(e) => panic!("error while writing fixed int: {e:?}"),
        //}
    }

    #[inline]
    pub fn clear_last_key(&mut self) {
        self.last_key[0..self.last_key_len].fill(0);
    }

    //TODO this probably contains as many errors as lines of code
    fn first_output_on_path_to(&self, last_state: usize) -> Option<usize> {
        //keep going until last_state is reached
        for (i, state) in self.temp_states[..last_state].iter().enumerate() {
            //if state has children then it must have outgoing transitions and possibly output
            //values associated with them
            if state.has_children() {
                let last_output = state.get_last_output();

                //if no output on last transition => continue searching on `last path`
                match last_output.get_value_as_deref() {
                    Some(_) => return Some(i),
                    None => {
                        continue;
                    }
                }
            //state doesn't have children
            } else {
                assert_eq!(i, self.temp_states.len() - 1);
                //panic!("state {state:?} doesn't have children");
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
            } else {
                //add value to last output of last state
                let last_state = &mut self.temp_states[last_state_index];
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
        //println!("first state index: {first_state_index}");
        //println!("last state index: {last_state_index}");
        assert!(first_state_index <= last_state_index);

        let mut diff = None;

        let mut trimmed_characters = 0;

        let mut done_with_prefix = false;

        for (i, state) in self
            .temp_states
            .iter_mut()
            .enumerate()
            .skip(first_state_index)
        {
            if i == last_state_index {
                break;
            }

            assert!(state.has_children());

            //propagate diff from previous iteration if and only if the current state is not final.
            //A final state means that output on previous edges constitutes its total output
            //Propagating would mean modifying the value associated with that final state (key)
            if !state.is_final {
                state.propagate_new_prefix(diff.as_deref());
            } else {
                if i != first_state_index {
                    unreachable!("diff_propagation should never be called on a final state {i}");
                }
            }

            //Take last output.
            let current_last_output = state.get_last_output();

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
                    diff = state.truncate_last_output(prefix_up_to);
                } else {
                    diff = state.clear_last_output();
                }

                //no output on edge
            } else {
                diff = None;
            }
        }

        //Now propagate diff to ALL edges EXCEPT last one. Diff will be prepended to their
        //outputs
        let last_state = &mut self.temp_states[last_state_index];
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

    ///Returns the last index of a final state insaddresse the temp_states vec (if any)
    ///Returns None in case no state is marked as final
    pub fn temp_contains_final(&self, up_to: usize) -> Option<usize> {
        self.temp_states[..=up_to]
            .iter()
            .rev()
            .position(|state| state.is_final)
            .map(|i| up_to - i)
    }
}

//automaton implementation
impl<'a> Automaton for Transducer<'a> {
    fn is_final_state(&self, state: usize) -> bool {
        self.temp_states
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    fn get(&self, _input_tape: &[u8]) -> Option<Vec<u8>> {
        todo!();
    }

    //process input tape (key) and return output collected along the path to a final state.
    //Returns None if no final state is reached or an error state is reached
    //fn get(&self, input_tape: &[u8]) -> Option<Vec<u8>> {
    //    assert!(self.temp_states.len() > 0);
    //
    //    let mut current_state = 0;
    //
    //    let mut tmp: Vec<u8> = Vec::new();
    //    let mut output: Vec<u8> = Vec::new();
    //
    //    for byte in input_tape.iter() {
    //        let state = &self.temp_states[current_state];
    //
    //        if state.is_final {
    //            output = tmp;
    //            tmp = Vec::new();
    //        }
    //
    //        //there exists a transition from state with byte
    //        if let Some(transition) = state.delta_with_output(*byte) {
    //            let (new_state, tmp_value) = transition;
    //            //move to new state
    //            assert!(current_state != new_state);
    //            current_state = new_state;
    //
    //            //process output of transition.
    //            //Not all transitions have output associated with them
    //            if let Some(v) = tmp_value.get_value_as_deref() {
    //                //if a value exists => extend final (possible) output with it
    //                tmp.extend(v);
    //            }
    //        }
    //        //no outgoing transition from state with byte
    //        else {
    //            return None;
    //        }
    //    }
    //
    //    //if a final state was reached => return the value
    //    if self.is_final_state(current_state) {
    //        output = tmp;
    //        return Some(output);
    //    }
    //    //else return None => input_tape wasn't a valaddress key
    //    else {
    //        return None;
    //    }
    //}
}

pub enum FstError {}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {}
