use bincode::{Decode, Encode, error::EncodeError};

use crate::{
    automaton::{AutomatonState, WithAddress},
    fst_state::TransitionOutput,
    raw_fsa_state::InnerFsaState,
    raw_writer::RawWriter,
    register::Register,
};

///Represents a single state of a FSA (Finite State Automaton)
///TODO address should be address and the rest of the fields should be moved to 'InnerState'
#[derive(Debug, Clone, Encode, Decode)]
pub struct FsaState {
    address: usize,
    inner: InnerFsaState,
}

impl FsaState {
    pub fn new() -> Self {
        FsaState {
            address: 0,
            inner: InnerFsaState::new(false),
        }
    }

    ///Makes the last transition of current state point to other_state
    #[inline]
    pub fn merge_with(&mut self, other_state: usize) -> bool {
        self.inner.merge_with(other_state)
    }

    ///All transitions must be equal for this to return true.
    ///This means that each character must be associated with the same state
    fn are_transitions_equal(&self, other_state: &FsaState) -> bool {
        self.inner.are_transitions_equal(&other_state.inner)
    }

    ///TODO should be used by merge_with
    pub fn set_last_transition_child(&mut self, to_state: usize) -> bool {
        self.inner.set_last_transition_child(to_state)
    }
}

impl WithAddress for FsaState {
    fn get_address(&self) -> usize {
        self.address
    }
}

///Implement AutomatonState trait for FsaState
impl AutomatonState for FsaState {
    #[inline]
    fn delta(&self, c: u8) -> Option<usize> {
        self.inner.delta(c)
    }

    ///must not be used
    ///fsa doesn't have any output associated with its transitions
    fn delta_with_output(&self, c: u8) -> Option<(usize, &TransitionOutput)> {
        self.inner.delta_with_output(c)
    }

    fn has_children(&self) -> bool {
        self.inner.has_children()
    }

    fn last_child(&self) -> Option<usize> {
        self.inner.last_child()
    }

    fn add_transition(&mut self, y: u8, new_state_address: usize) {
        self.inner.add_transition(y, new_state_address);
    }

    fn clear_state(&mut self) {
        self.inner.clear_state();
    }

    fn is_final(&self) -> bool {
        self.inner.is_final()
    }

    fn get_transitions(&self) -> &[(u8, usize)] {
        self.inner.get_transitions()
    }

    fn get_outputs(&self) -> Option<&[TransitionOutput]> {
        None
    }
}

///are both states in the same equivalence class
impl PartialEq for FsaState {
    fn eq(&self, other_state: &Self) -> bool {
        self.inner.eq(&other_state.inner)
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
    ///Return a new DFA that is serialized to file 'output_to'
    ///alphabet_dim is just used as a hint to amortize later adding of transitions
    pub fn new(alphabet_dim: usize, output_to: &'a str) -> Self {
        let mut tmp = Vec::with_capacity(alphabet_dim);

        tmp.push(FsaState::new());

        if let Ok(wr) = RawWriter::new(output_to) {
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

    ///get the index of the last prefix character insaddresse y
    pub fn common_prefix(&self, y: &[u8]) -> usize {
        //TODO SIMD it using chunk_exact
        let prefix_len = self
            .last_str
            .iter()
            .zip(y)
            .take_while(|(x, y)| x == y)
            .count();

        assert!(prefix_len <= y.len());
        return prefix_len;
    }

    ///start keeping track of suffix y in the fsa
    ///new states will originate from `last_state`
    ///a suffix = "" will do nothing
    pub fn add_suffix_to_fsa(&mut self, from_state: usize, y: &[u8]) {
        assert!(y.len() <= 100);

        let string_len = y.len();

        let initial_number_of_states = self.states.len();

        let mut current_state = from_state;

        for (i, byte) in y.iter().enumerate() {
            //get the current state and add a new transition to it
            let new_state_address = self.states.len();

            self.states[current_state].add_transition(*byte, new_state_address);

            //if last loop iteration the state is to be marked as final
            let is_state_final = i == string_len - 1;

            let new_state: FsaState = FsaState {
                address: self.states.len(),
                inner: InnerFsaState::new(is_state_final),
            };

            assert!(new_state.inner.transitions.is_empty());
            self.states.push(new_state);

            current_state = new_state_address;
        }

        assert!(initial_number_of_states <= self.states.len());
    }

    //assume state has children
    pub fn replace_or_register(&mut self, state_index: usize) {
        let state = &self.states[state_index];
        assert!(state.has_children());

        //get the state `address` of the last outgoing transition from current state
        let child_index = state.last_child().unwrap();

        if self.state_has_children(child_index) {
            self.replace_or_register(child_index);
        }

        //merge states added by previous add_suffix_to_fsa that are equivalent to the ones in the register
        if let Some(state_to_merge_with) = self.register_contains_equivalent(child_index) {
            //now get a mutable reference to the state who's child is to be merged
            let current_state = &mut self.states[state_index];

            //merge last child
            current_state.merge_with(state_to_merge_with);

            self.states.pop();
        } else {
            match self.add_to_register(child_index) {
                Ok(written_bytes) => {
                    let current_state = &mut self.states[state_index];

                    let ok = current_state.set_last_transition_child(written_bytes);

                    if !ok {
                        unreachable!();
                    }
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
        state.inner.transitions.shrink_to_fit();
        state.address = address;

        self.register.add_state(state);

        Ok(address)
    }

    ///Writes the given FsaState and then its length (in bytes).
    ///Returns the address to the start of the state len.
    pub fn persist_state(&mut self, i: usize) -> Result<usize, EncodeError> {
        let state = &self.states[i];

        let wrote = self.writer.write_state::<InnerFsaState>(&state.inner)?;

        //this is the address that is to be returned
        let total_written_bytes = self.writer.get_written_bytes();

        match self.writer.write_int(wrote) {
            Ok(_) => Ok(total_written_bytes),
            Err(e) => panic!("error while writing fixed int: {e:?}"),
        }
    }

    pub fn persist_initial_state(&mut self) -> Result<usize, EncodeError> {
        let state = &self.states[0];

        let wrote = self.writer.write_state::<InnerFsaState>(&state.inner)?;

        //this is the address that is to be returned
        let total_written_bytes = self.writer.get_written_bytes();

        match self.writer.write_fixed_int(wrote) {
            Ok(_) => Ok(total_written_bytes),
            Err(e) => panic!("error while writing fixed int: {e:?}"),
        }

        ////write initial state
        //self.persist_state(0)
        //    .expect("couldn't persist initial state to file");
    }

    pub fn state_has_children(&self, state_index: usize) -> bool {
        return self.states[state_index].has_children();
    }

    #[inline]
    pub fn clear_last_str(&mut self) {
        self.last_str[0..self.last_str_len].fill(0);
    }
}
