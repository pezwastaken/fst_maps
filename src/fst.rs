use crate::{
    automaton::AutomatonState,
    fst_state::FstState,
    fst_state::common_prefix_index,
    register::Register,
    register::{REGISTER_CAPACITY, REGISTER_COUNT_MASK},
};
use core::result::Result;
use std::fs::File;
use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Transducer {
    states: Vec<FstState>,
    last_key: [u8; 100],
    last_key_len: usize,
    last_value: [u8; 100],
    last_value_len: usize,
    //cache: Dictionary,
    register: Register,
}

impl Transducer {
    fn new() -> Self {
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

    fn push_zinit_state(&mut self) {
        self.states.push(FstState {
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
        });
    }

    pub fn is_final_state(&self, state: usize) -> bool {
        self.states
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    pub fn delta_star(&self, state: usize, input_tape: &[u8]) -> Option<usize> {
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
    pub fn delta_star_from_initial_state(&self, input_tape: &[u8]) -> Option<usize> {
        self.delta_star(0, input_tape)
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

    fn state_has_children(&self, state_index: usize) -> bool {
        return self.states[state_index].has_children();
    }

    #[inline]
    fn clear_last_key(&mut self) {
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
    fn insert_value(&mut self, value: &[u8], last_state_index: usize) {
        assert!(value.len() > 0);

        if self.states[last_state_index].is_final {
            todo!();
        }

        //output value present in the given path
        if let Some(first_state_with_output) = self.first_output_on_path_to(last_state_index) {
            //find the common prefix and diff between current value and the output on the just found
            //state
            let state = &self.states[first_state_with_output];

            let prefix_len = state.find_new_prefix_len(&value);
            println!("prefix len (of value) is: {prefix_len}");

            //set output prefix as new output for first_state_with_output.
            //also get diff. Note that diff may be None
            let diff: Option<Vec<u8>>;

            if prefix_len != 0 {
                let state = &mut self.states[first_state_with_output];
                diff = state.truncate_last_output(prefix_len);
            } else {
                let state = &mut self.states[first_state_with_output];
                diff = Some(state.clear_last_output());
                println!("called clear on last_output\nstate is now: {:?}", state);
            }

            println!("diff is: {:?}", diff);
            let last_state = &mut self.states[last_state_index];
            last_state.propagate_new_prefix(diff);

            last_state.set_last_output(&value[prefix_len..]);
        }
        //TODO this is a bit weird
        //maybe move it in the `then` branch
        //
        //new value with no prefix on path
        //should just add the entire value to the last output of last_state
        else {
            let last_state = &mut self.states[last_state_index];
            last_state.set_last_output(&value);
            println!("in else: last state is {:?}", last_state);
        }
    }

    /*
     * replace current key `last_key` with `new_key`
     * the array should first be filled with 0s
     */
    fn update_last_key(&mut self, new_key: &[u8]) {
        let word_len = new_key.len();
        self.clear_last_key();

        self.last_key_len = word_len;
        self.last_key[..word_len].copy_from_slice(&new_key[..word_len]);
    }
}

pub enum FstError {}

pub fn build_transducer_from_collection(
    collection: Vec<(Vec<u8>, Vec<u8>)>,
) -> Result<Transducer, FstError> {
    assert!(collection.len() > 0);

    let mut transducer = Transducer::new();

    let mut previous_key_i = 0;

    transducer.push_zinit_state();

    assert_eq!(transducer.states.len(), 1);

    for (i, key_value_pair) in collection.iter().enumerate() {
        let (key, value) = key_value_pair;

        //key common prefix len
        let prefix_len = common_prefix_index(&transducer.last_key, &key);

        //key common prefix
        let prefix = &key[0..prefix_len];

        //compute delta* from initial state with the common prefix as input.
        //The reached state will be the one after which the suffix will be added.
        //Since the dataset is ordered, no other word will "touch" states that come after last_state.
        let last_state_index = transducer
            .delta_star_from_initial_state(prefix)
            .expect(&format!("transitioned to err state after checking common prefix!\nWas processing {:?} and common prefix was {:?}.\nLast str is: {:?}", std::str::from_utf8(key).unwrap(), std::str::from_utf8(prefix).unwrap(), std::str::from_utf8(&collection[previous_key_i].0).unwrap()));

        assert!(last_state_index < transducer.states.len());

        let last_state = &transducer.states[last_state_index];
        if last_state.has_children() {
            transducer.replace_or_register(last_state_index);
        }

        let suffix = &key[prefix_len..];
        transducer.add_suffix_to_fsa(last_state_index, suffix);

        //insert current value associated with current key in the fst
        transducer.insert_value(&value, last_state_index);

        transducer.update_last_key(key);

        previous_key_i = i;
    }

    transducer.replace_or_register(0);

    return Ok(transducer);
}

pub fn dump_graph(fst: &Transducer, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    file.write(b"digraph DFA {")?;
    file.write(b"    {0} [shape=circle, color=blue, label=\"start\"];")?;

    //mark the accepting states
    for (i, state) in fst.states.iter().enumerate() {
        if state.is_final {
            let s = format!("    {} [shape=doublecircle, color=green];", i);
            file.write(s.as_bytes())?;
        }
    }

    //add transitions
    for (i, state) in fst.states.iter().enumerate() {
        let state_transitions = &state.transitions;
        let state_outputs = &state.outputs;

        println!("state {i}");

        assert!(state_transitions.len() >= state_outputs.len());
        for (transition, output) in state_transitions.iter().zip(state_outputs) {
            let (on_char, to_state) = transition;

            let label: String;
            if let Some(value) = output {
                label = format!(
                    "{}: {}",
                    *on_char as char,
                    std::str::from_utf8(value).unwrap()
                );
            } else {
                label = format!("{}", *on_char as char);
            }
            let s = format!("    {} -> {} [label=\"{}\"];", i, *to_state, label);
            file.write(s.as_bytes())?;
        }
    }

    file.write(b"}")?;

    Ok(())
}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {

    use crate::fst_state::FstState;
    use crate::fst_state::FstStateOutput;

    use super::build_transducer_from_collection;
    use super::dump_graph;

    #[test]
    fn two_pairs() {
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("colle").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        //dump_graph(&fst, "fst.dot");
    }

    #[test]
    fn three_pairs() {
        println!("-----------------------------\n\n");
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("colle").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("pizza").as_bytes().to_vec(),
            String::from("zappa").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair, third_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        //dump_graph(&fst, "fst.dot");
    }

    #[test]
    fn four_pairs() {
        println!("-----------------------------\n\n");
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("ciaspola").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("come").as_bytes().to_vec(),
            String::from("ronda").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("zoccolo").as_bytes().to_vec(),
            String::from("picciotto").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair, third_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        dump_graph(&fst, "fst.dot");
    }
}
