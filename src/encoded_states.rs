use bincode::{Decode, Encode};

use crate::automaton::AutomatonState;

///These are the actual types of states that can be written to file.

///transition on character
#[derive(Debug, Clone, Encode, Decode)]
pub struct Basic(u8);

///transition on character, next state
#[derive(Debug, Clone, Encode, Decode)]
pub struct BasicWithNext(u8, usize);

///vector of transitions
#[derive(Debug, Clone, Encode, Decode)]
pub struct MultiTransition(Vec<(u8, usize)>);

impl Basic {
    fn new(c: u8) -> Self {
        Basic(c)
    }
}

impl BasicWithNext {
    fn new(c: u8, to_addr: usize) -> Self {
        BasicWithNext(c, to_addr)
    }
}

impl MultiTransition {
    fn new(vec: Vec<(u8, usize)>) -> Self {
        MultiTransition(vec)
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum CompressedFsaState {
    BasicState(Basic, bool),
    BasicWithNextState(BasicWithNext, bool),
    MultiTransitionState(MultiTransition, bool),
}

impl CompressedFsaState {
    #[inline(always)]
    fn new_basic(c: u8, is_final: bool) -> Self {
        CompressedFsaState::BasicState(Basic(c), is_final)
    }

    #[inline(always)]
    fn new_basic_with_next(c: u8, to_state: usize, is_final: bool) -> Self {
        CompressedFsaState::BasicWithNextState(BasicWithNext(c, to_state), is_final)
    }

    #[inline(always)]
    fn new_multi_transition(vec: Vec<(u8, usize)>, is_final: bool) -> Self {
        CompressedFsaState::MultiTransitionState(MultiTransition(vec), is_final)
    }

    pub fn new_from_fsa_state<T: AutomatonState>(state: &T, is_next_state_multi: bool) -> Self {
        let is_final = state.is_final();

        match state.get_transitions() {
            [transition] => {
                if !is_next_state_multi {
                    let (on_char, _) = transition;
                    Self::new_basic(*on_char, is_final)
                } else {
                    let (on_char, to_state) = transition;
                    Self::new_basic_with_next(*on_char, *to_state, is_final)
                }
            }
            //NOTE: don't like this .to_vec()
            transitions => Self::new_multi_transition(transitions.to_vec(), is_final),
        }
    }
}

#[derive(Debug)]
pub enum CompressedStateTag {
    BasicTag,
    BasicWithNextStateTag,
    MultiTransitionTag,
    LastTag,
    NotAvailableTag,
}
