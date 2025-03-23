use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Write},
};

#[derive(Debug)]
struct State {
    id: usize,
    is_final: bool,
    transitions: BTreeMap<u8, usize>,
}

const REGISTER_CAPACITY: usize = 256;
const REGISTER_COUNT_MASK: usize = 255;

//array of frozen states (their identifiers)
#[derive(Debug)]
struct Register {
    elements: [usize; REGISTER_CAPACITY],
    count: usize,
}

#[derive(Debug)]
pub struct Automata {
    states: Vec<State>,
    last_str: [u8; 100],
    register: Register,
}

//strings are supposed to sorted
pub fn build_fsa(fsa: &mut Automata, strings: &[String]) {
    let mut previous_str = "";
    for item in strings.iter() {
        //index of last character of common prefix between current item and the last
        //processed word
        let prefix_len = fsa.common_prefix(&item);

        //let curr_item = &item[..];
        let prefix = &item[0..prefix_len];

        let last_state_index = fsa
            .delta_star_from_initial_state(prefix)
            .expect(&format!("transitioned to err state after checking common prefix!\nWas processing {0} and common prefix was {1}.\nLast str is: {2}", item, prefix, previous_str));

        let suffix = &item[prefix_len..];

        assert!(last_state_index < fsa.states.len());

        //
        if fsa.state_has_children(last_state_index) {
            //let last_state = &fsa.states.borrow()[last_state_index];
            fsa.replace_or_register(last_state_index);
        }

        //TODO this should be a method of State
        //maybe?
        fsa.add_suffix_to_fsa(last_state_index, suffix.as_bytes());

        let word_len = item.len();

        fsa.clear_last_str();
        fsa.last_str[..word_len].copy_from_slice(&item.as_bytes()[..word_len]);

        previous_str = &item;
    }

    fsa.replace_or_register(0);
}

pub fn dump_graph(fsa: &Automata, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    file.write(b"digraph DFA {")?;
    file.write(b"    {0} [shape=circle, color=blue, label=\"start\"];")?;

    //mark the accepting states
    for state in &fsa.states {
        if state.is_final {
            let s = format!("    {} [shape=doublecircle, color=green];", state.id);
            file.write(s.as_bytes())?;
        }
    }

    //add transitions
    for state in &fsa.states {
        let state_transitions = &state.transitions;

        for (on_char, to_state) in state_transitions.iter() {
            let s = format!(
                "    {} -> {} [label=\"{}\"];",
                state.id, to_state, *on_char as char
            );
            file.write(s.as_bytes())?;
        }
    }

    file.write(b"}")?;

    Ok(())
}

impl Automata {
    pub fn new(alphabet_dim: usize) -> Self {
        let mut tmp = Vec::with_capacity(alphabet_dim);

        tmp.push(State {
            id: 0,
            is_final: false,
            //transitions: BTreeMap::with_capacity(alphabet_dim),
            transitions: BTreeMap::new(),
        });

        assert_eq!(tmp.len(), 1);

        Automata {
            states: tmp.into(),
            last_str: [0; 100],
            register: Register {
                elements: [0; REGISTER_CAPACITY],
                count: 0,
            },
        }
    }

    pub fn is_final_state(&self, state: usize) -> bool {
        self.states
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    //TODO should be moved to state impl
    pub fn delta_star(&self, state: usize, input_tape: &str) -> Option<usize> {
        assert!(self.states.len() > 0);
        let mut current_state = state;

        for byte in input_tape.as_bytes().iter() {
            if let Some(new_state) = self.states[current_state].delta(*byte) {
                //move to new state
                current_state = new_state;
            } else {
                return None;
            }
        }

        return Some(current_state);
    }

    pub fn delta_star_from_initial_state(&self, input_tape: &str) -> Option<usize> {
        self.delta_star(0, input_tape)
    }

    pub fn get_n_of_states(&self) -> usize {
        self.states.len()
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

            let new_state: State = State {
                id: new_state_id,
                is_final: is_state_final,
                transitions: BTreeMap::new(),
            };

            self.states.push(new_state);

            current_state = new_state_id;
        }

        assert!(initial_number_of_states <= self.states.len());
    }

    //assume state has children
    fn replace_or_register(&mut self, state_index: usize) {
        let state = &self.states[state_index];
        assert!(state.has_children());

        let child_index = self.states[state.last_child().unwrap()].id;
        //println!("r_o_r processing node {state_index}. child index: {child_index}");

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

            let state_id = current_state.id;

            self.states.pop();
        } else {
            self.register.add_state(child_index);
        }
    }

    fn register_contains_equivalent(&self, state_index: usize) -> Option<usize> {
        let cached_states = &self.register.elements;

        let state = &self.states[state_index];
        for i in 0..self.register.count {
            let cached_state_index = cached_states[i];

            let cached_state = &self.states[cached_state_index];
            if state.is_equal(cached_state) {
                return Some(cached_state_index);
            }
        }

        return None;
    }

    fn state_has_children(&self, state_index: usize) -> bool {
        return self.states[state_index].has_children();
    }

    //SIMD the fucker
    //TODO also get rid of the unwrap
    fn clear_last_str(&mut self) {
        let last_str_real_len = self.last_str.iter().position(|x| *x == 0).unwrap();
        for i in 0..last_str_real_len {
            self.last_str[i] = 0;
        }
    }
}

impl State {
    fn delta(&self, c: u8) -> Option<usize> {
        self.transitions.get(&c).copied().map(|new_state| new_state)
    }

    //check whether current state has outgoing transitions
    fn has_children(&self) -> bool {
        !(self.transitions.is_empty())
    }

    //get last child of current state
    //last is based on lexicographic order
    fn last_child(&self) -> Option<usize> {
        match self.transitions.last_key_value() {
            Some((_, next_state)) => Some(*next_state),
            None => None,
        }
    }

    fn add_transition(&mut self, y: u8, new_state_id: usize) {
        assert!(self.id < new_state_id);
        self.transitions.insert(y, new_state_id);
    }

    //are both states in the same equivalence class
    fn is_equal(&self, other_state: &State) -> bool {
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
        //NOTE they are ordered by key using a BTreeMap
        let all_transitions_equal = self_transitions
            .iter()
            .zip(other_state_transitions)
            .all(|(x, y)| (*x.0 == *y.0) && (*x.1 == *y.1));

        return all_transitions_equal;
    }

    fn merge_with(&mut self, other_state: usize) -> bool {
        if let Some(mut entry) = self.transitions.last_entry() {
            entry.insert(other_state);
            return true;
        }

        return false;
    }
}

impl Register {
    //TODO get rid of mod
    fn add_state(&mut self, state_id: usize) {
        let i = self.count % REGISTER_CAPACITY;
        self.elements[i] = state_id;
        self.count += 1;
        self.count = self.count & REGISTER_COUNT_MASK;
    }
}
