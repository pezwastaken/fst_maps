use bincode::{Decode, Encode, error::EncodeError};

use crate::{
    automaton::AutomatonState, fst_state::TransitionOutput, raw_writer::RawWriter,
    register::Register,
};
//use crate::register::{REGISTER_CAPACITY, REGISTER_COUNT_MASK};

#[derive(Debug, Clone, Encode, Decode)]
pub struct FsaState {
    id: usize,
    pub is_final: bool,
    pub transitions: Vec<(u8, usize)>,
}

impl FsaState {
    pub fn new() -> Self {
        FsaState {
            id: 0,
            is_final: false,
            transitions: Vec::new(),
        }
    }

    #[inline]
    pub fn merge_with(&mut self, other_state: usize) -> bool {
        if let Some(transition) = self.transitions.last_mut() {
            //field 1 is next state
            transition.1 = other_state;
            return true;
        }

        return false;
    }

    fn are_transitions_equal(&self, other_state: &FsaState) -> bool {
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

    pub fn set_last_transition_child(&mut self, to_state: usize) {
        assert!(!self.transitions.is_empty());
        self.transitions.last_mut().unwrap().1 = to_state;
    }
}

impl AutomatonState for FsaState {
    #[inline]
    fn delta(&self, c: u8) -> Option<usize> {
        //self.transitions.iter().position(|x| x.0 == c);

        let transition = self
            .transitions
            .iter()
            .find(|(transition_char, _)| *transition_char == c);

        //return Some(transition?.1);
        return transition.map(|(_, next_state)| *next_state);
    }

    ///should not be used
    fn delta_with_output(&self, _: u8) -> Option<(usize, &TransitionOutput)> {
        unimplemented!()
    }

    fn has_children(&self) -> bool {
        !(self.transitions.is_empty())
    }

    fn last_child(&self) -> Option<usize> {
        match self.transitions.last() {
            Some((_, child_state)) => Some(*child_state),
            None => None,
        }
    }

    fn add_transition(&mut self, y: u8, new_state_id: usize) {
        self.transitions.push((y, new_state_id));
        assert!(self.transitions.len() > 0);
    }

    fn clear_state(&mut self) {
        self.is_final = false;
        self.transitions.clear();
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn is_final(&self) -> bool {
        self.is_final
    }

    fn get_transitions(&self) -> &[(u8, usize)] {
        &self.transitions
    }

    fn get_outputs(&self) -> Option<&[TransitionOutput]> {
        None
    }
}

//are both states in the same equivalence class
impl PartialEq for FsaState {
    fn eq(&self, other_state: &Self) -> bool {
        if self.is_final != other_state.is_final {
            return false;
        }

        match self.transitions.len() != other_state.transitions.len() {
            true => return false,
            false => (),
        }

        let transitions_equal = self.are_transitions_equal(other_state);

        let res = transitions_equal;

        return res;
    }
}

impl Eq for FsaState {}

type States = Vec<FsaState>;

#[derive(Debug)]
pub struct Acceptor<'a> {
    pub states: States,
    pub last_str: [u8; 100],
    pub last_str_len: usize,
    pub register: Register<FsaState>,
    writer: RawWriter<'a>,
}

impl<'a> Acceptor<'a> {
    pub fn new(alphabet_dim: usize) -> Self {
        let mut tmp = Vec::with_capacity(alphabet_dim);

        tmp.push(FsaState {
            id: 0,
            is_final: false,
            transitions: Vec::new(),
        });

        if let Ok(wr) = RawWriter::new("test_fsa.fsa") {
            Acceptor {
                states: tmp.into(),
                last_str: [0; 100],
                last_str_len: 0,
                register: Register::new(),
                writer: wr,
            }
        } else {
            panic!("couldn't create a writer to the given file");
        }
    }

    pub fn is_final_state(&self, state: usize) -> bool {
        self.states
            .get(state)
            .expect("no state with index: {state}")
            .is_final
    }

    pub fn delta_star(&self, state: usize, input_tape: &str) -> Option<usize> {
        unimplemented!()
        //assert!(self.states.len() > 0);
        //let mut current_state = state;
        //
        //for byte in input_tape.as_bytes().iter() {
        //    if let Some(new_state) = self.states[current_state].delta(*byte) {
        //        //move to new state
        //        assert!(current_state != new_state);
        //        current_state = new_state;
        //    } else {
        //        return None;
        //    }
        //}
        //
        //return Some(current_state);
    }

    #[inline]
    pub fn delta_star_from_initial_state(&self, input_tape: &str) -> Option<usize> {
        self.delta_star(0, input_tape)
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

            let new_state: FsaState = FsaState {
                id: self.states.len(),
                is_final: is_state_final,
                transitions: Vec::new(),
            };

            assert!(new_state.transitions.is_empty());
            self.states.push(new_state);

            current_state = new_state_id;
        }

        assert!(initial_number_of_states <= self.states.len());
    }

    //assume state has children
    pub fn replace_or_register(&mut self, state_index: usize) {
        //println!("called ror on state {state_index}");
        let state = &self.states[state_index];
        //println!("state:  {:?}", state);
        assert!(state.has_children());

        //println!("has children is {}", state.has_children());

        //let last_transition = self.states[state.last_child().unwrap()]
        //    .transitions
        //    .last()
        //    .expect("replace or register called on state without outgoing transitions");
        //let last_transition = &self.states[state.last_child().unwrap()];

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

            //let state_id = current_state.id;

            self.states.pop();
        } else {
            //let child = self.states[child_index].clone();
            ////self.register.add_state(child, child_index);
            //self.register.add_state(child);
            match self.add_to_register(child_index) {
                Ok(written_bytes) => {
                    let current_state = &mut self.states[state_index];
                    current_state.set_last_transition_child(written_bytes);
                }

                Err(e) => panic!("error during add to register: {e:?}"),
            }
        }
    }

    fn register_contains_equivalent(&self, state_index: usize) -> Option<usize> {
        let state = &self.states[state_index];

        self.register.get_equivalent(state)
    }

    fn add_to_register(&mut self, i: usize) -> Result<usize, EncodeError> {
        assert_eq!(i, self.states.len() - 1);
        let address = self.persist_state(i)?;

        let mut state = self.states.pop().unwrap();
        state.id = address;

        self.register.add_state(state);

        //self.states.pop();

        Ok(address)
    }

    ///Writes the given FsaState and then its length (in bytes).
    ///Returns the address to the start of the state len.
    pub fn persist_state(&mut self, i: usize) -> Result<usize, EncodeError> {
        let state = &self.states[i];

        let wrote = self.writer.write_state::<FsaState>(state)?;

        //this is the address that is to be returned
        let total_written_bytes = self.writer.get_written_bytes();

        match self.writer.write_fixed_int(wrote) {
            Ok(_) => Ok(total_written_bytes),
            Err(e) => panic!("error while writing fixed int: {e:?}"),
        }
    }

    pub fn persist_initial_state(&mut self) {
        //write initial state
        self.persist_state(0)
            .expect("couldn't persist initial state to file");
    }

    pub fn state_has_children(&self, state_index: usize) -> bool {
        return self.states[state_index].has_children();
    }

    #[inline]
    pub fn clear_last_str(&mut self) {
        self.last_str[0..self.last_str_len].fill(0);
    }
}
