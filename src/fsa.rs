use std::{cell::RefCell, collections::BTreeMap};

struct State {
    id: usize,
    is_final: bool,
    transitions: RefCell<BTreeMap<u8, usize>>,
}

const REGISTER_CAPACITY: usize = 256;

//array of frozen states (their identifiers)
struct Register {
    elements: [usize; REGISTER_CAPACITY],
    count: usize,
}

pub struct Automata {
    states: RefCell<Vec<State>>,
    last_str: [u8; 100],
    register: RefCell<Register>,
}

//strings are supposed to sorted
pub fn build_fsa(fsa: &mut Automata, strings: &[&str]) {
    for item in strings.iter() {
        //index of last character of common prefix between current item and the last
        //processed word
        let prefix_len = fsa.common_prefix(*item);

        let prefix = &item[0..prefix_len];

        let last_state_index = fsa
            .delta_star_from_initial_state(prefix)
            .expect("transitioned to err state after checking common prefix!");

        let suffix = &item[prefix_len..];

        assert!(last_state_index < fsa.states.borrow().len());

        //
        if fsa.state_has_children(last_state_index) {
            //let last_state = &fsa.states.borrow()[last_state_index];
            fsa.replace_or_register(last_state_index);
        }

        //TODO this should be a method of State
        //maybe?
        fsa.add_suffix_to_fsa(last_state_index, suffix.as_bytes());

        let suffix_len = suffix.len();
        fsa.last_str[..suffix_len].copy_from_slice(&suffix.as_bytes()[..suffix_len]);
    }
}

impl Automata {
    pub fn new(alphabet_dim: usize) -> Self {
        let mut tmp = Vec::with_capacity(alphabet_dim);

        tmp.push(State {
            id: 0,
            is_final: false,
            //transitions: BTreeMap::with_capacity(alphabet_dim),
            transitions: RefCell::new(BTreeMap::new()),
        });

        assert_eq!(tmp.len(), 1);

        Automata {
            states: tmp.into(),
            last_str: [0; 100],
            register: RefCell::new(Register {
                elements: [0; REGISTER_CAPACITY],
                count: 0,
            }),
        }
    }

    pub fn is_final_state(&self, state: usize) -> bool {
        self.states
            .borrow()
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    //TODO should be moved to state impl
    pub fn delta_star(&self, state: usize, input_tape: &str) -> Option<usize> {
        assert!(self.states.borrow().len() > 0);
        let mut current_state = state;

        for byte in input_tape.as_bytes().iter() {
            if let Some(new_state) = self.states.borrow()[current_state].delta(*byte) {
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
        self.states.borrow().len()
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
    pub fn add_suffix_to_fsa(&self, from_state: usize, y: &[u8]) {
        assert!(y.len() <= 100);

        let string_len = y.len();

        //let initial_number_of_states = self.states.borrow().len();

        let mut current_state = from_state;

        for (i, byte) in y.iter().enumerate() {
            //get the current state and add a new transition to it
            let new_state_id = self.states.borrow().len();

            self.states.borrow_mut()[current_state].add_transition(*byte, new_state_id);

            //if last loop iteration the state is to be marked as final
            let is_state_final = i == string_len - 1;

            let new_state: State = State {
                id: new_state_id,
                is_final: is_state_final,
                transitions: RefCell::new(BTreeMap::new()),
            };

            self.states.borrow_mut().push(new_state);

            current_state = new_state_id;
        }

        //assert!(initial_number_of_states <= self.states.borrow().len());
    }

    //assume state has children
    fn replace_or_register(&self, state_index: usize) {
        let mut automata_states = self.states.borrow_mut();

        let state = &automata_states[state_index];
        assert!(state.has_children());
        //let child = &automata_states[state.last_child().unwrap()];
        let child_index = automata_states[state.last_child().unwrap()].id;

        drop(automata_states);

        if self.state_has_children(child_index) {
            self.replace_or_register(child_index);
        }

        //equivalent state in register (cache)
        //if let Some(state_to_merge_with) = self.register_contains_equivalent(child_index) {
        //    state.merge_with(state_to_merge_with);
        //
        //    //let state_id = state.id;
        //
        //    automata_states.pop();
        //
        //    //assert_eq!(state_id, self.get_n_of_states() - 1);
        //
        //    return;
        //}
    }

    fn register_contains_equivalent(&self, state: &State) -> Option<usize> {
        let cached_states = &self.register.borrow().elements;

        let automata_states = self.states.borrow();

        let found = cached_states.iter().find(|x| {
            let tmp = &automata_states[**x];

            if tmp.is_equal(state) {
                return true;
            }
            false
        });

        let res = found.map(|x| automata_states[*x].id);
        return res;
    }

    fn state_has_children(&self, state_index: usize) -> bool {
        return self.states.borrow()[state_index].has_children();
    }
}

impl State {
    fn delta(&self, c: u8) -> Option<usize> {
        let state_transitions = self.transitions.borrow();

        state_transitions
            .get(&c)
            .copied()
            .map(|new_state| new_state)
    }

    //check whether current state has outgoing transitions
    fn has_children(&self) -> bool {
        !(self.transitions.borrow().is_empty())
    }

    //get last child of current state
    //last is based on lexicographic order
    fn last_child(&self) -> Option<usize> {
        match self.transitions.borrow().last_key_value() {
            Some((_, next_state)) => Some(*next_state),
            None => None,
        }
    }

    fn add_transition(&mut self, y: u8, new_state_id: usize) {
        assert!(self.id < new_state_id);
        //let mut next_state_id: usize = self.id + 1;
        //if let Some((key, value)) = self.transitions.borrow().last_key_value() {
        //    next_state_id = *value + 1;
        //    println!(
        //        "add transition from state {0} to future state {next_state_id}",
        //        self.id
        //    );
        //}
        self.transitions.borrow_mut().insert(y, new_state_id);
    }

    //are both states in the same equivalence class
    fn is_equal(&self, other_state: &State) -> bool {
        if self.is_final != other_state.is_final {
            return false;
        }

        match self.transitions.borrow().len() != other_state.transitions.borrow().len() {
            true => return false,
            false => (),
        }

        let self_transitions = self.transitions.borrow();
        let other_state_transitions = other_state.transitions.borrow();
        //actually check transitions
        //NOTE they are ordered by key using a BTreeMap
        self_transitions
            .iter()
            .zip(&*other_state_transitions)
            .all(|(x, y)| (*x.0 == *y.0) && (*x.1 == *y.1));

        return true;
    }

    fn merge_with(&self, other_state: usize) -> bool {
        assert_ne!(
            *self.transitions.borrow().iter().max().unwrap().1,
            other_state
        );

        if let Some(mut entry) = self.transitions.borrow_mut().last_entry() {
            entry.insert(other_state);
            return true;
        }

        return false;
    }
}

impl Register {
    fn add_state(&mut self, state_id: usize) {
        let i = self.count % REGISTER_CAPACITY;
        self.elements[i] = state_id;
        self.count += 1;
    }
}
