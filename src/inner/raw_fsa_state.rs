use bincode::{Decode, Encode};

use crate::{
    automaton::{AutomatonState, TransitionOutput},
    compressed_states::{CompressedFsaState, encoded_len},
};

#[derive(Debug, Clone, Encode, Decode)]
pub struct InnerFsaState {
    pub is_final: bool,
    pub transitions: Vec<(u8, usize)>,
}

impl InnerFsaState {
    pub fn new(f: bool) -> Self {
        Self {
            is_final: f,
            transitions: Vec::new(),
        }
    }

    #[inline]
    pub fn new_with_transition(t: (u8, usize), f: bool) -> Self {
        Self {
            is_final: f,
            transitions: vec![t],
        }
    }

    ///Makes the last transition of current state point to other_state
    #[inline]
    pub fn merge_with(&mut self, other_state: usize) -> bool {
        self.set_last_transition_child(other_state)
    }

    ///All transitions must be equal for this to return true.
    ///This means that each character must be associated with the same state
    pub fn are_transitions_equal(&self, other_state: &InnerFsaState) -> bool {
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

    ///TODO should be used by merge_with
    pub fn set_last_transition_child(&mut self, to_state: usize) -> bool {
        assert!(!self.transitions.is_empty());
        if let Some(transition) = self.transitions.last_mut() {
            transition.1 = to_state;
            return true;
        }
        false
    }
}

///Implement AutomatonState trait for InnerFsaState
impl AutomatonState for InnerFsaState {
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

    ///must not be used
    ///fsa doesn't have any output associated with its transitions
    fn delta_with_output(&self, _: u8) -> Option<(usize, &TransitionOutput)> {
        unreachable!()
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

    fn add_transition(&mut self, y: u8, new_state_address: usize) {
        self.transitions.push((y, new_state_address));
        assert!(self.transitions.len() > 0);
    }

    fn clear_state(&mut self) {
        self.is_final = false;
        self.transitions.clear();
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

impl PartialEq for InnerFsaState {
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

impl From<CompressedFsaState> for InnerFsaState {
    fn from(value: CompressedFsaState) -> Self {
        match value {
            CompressedFsaState::NoTransitionsState(no_transitions, _) => Self::new(true),
            CompressedFsaState::SimpleState(simple, addr) => {
                let len = encoded_len(simple.get_symbol());
                let t = (simple.get_symbol(), addr - len);

                Self::new_with_transition(t, simple.is_final())
            }
            CompressedFsaState::SimpleWithNextState(simple_with_next, _) => {
                let t = (simple_with_next.get_symbol(), simple_with_next.get_dest());

                Self::new_with_transition(t, simple_with_next.is_final())
            }
            CompressedFsaState::ComplexState(complex, _) => Self {
                is_final: complex.is_final(),
                transitions: complex.take_transitions(),
            },
        }
    }
}
