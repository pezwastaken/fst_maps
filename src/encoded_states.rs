use bincode::{Decode, Encode};

use crate::automaton::{AutomatonState, WithAddress};

///These are the actual types of states that can be written to file.

///transition on character
#[derive(Debug, Clone, Encode, Decode)]
///u8 is padding to have the same size as `Basic(u8)`
pub struct NoTransitions(u8);

#[derive(Debug, Clone, Encode, Decode)]
pub struct Basic(u8);

///transition on character, next state
#[derive(Debug, Clone, Encode, Decode)]
pub struct BasicWithNext(u8, usize);

///vector of transitions
#[derive(Debug, Clone, Encode, Decode)]
pub struct MultiTransition(Vec<(u8, usize)>);

const _: () = assert!(
    size_of::<NoTransitions>() == size_of::<Basic>(),
    "size of NoTransitions must be equal to that of Basic"
);

// impl NoTransitions {
//     fn new() -> Self {
//         NoTransitions(0)
//     }
// }
//
// impl Basic {
//     fn new(c: u8) -> Self {
//         Basic(c)
//     }
// }
//
// impl BasicWithNext {
//     fn new(c: u8, to_addr: usize) -> Self {
//         BasicWithNext(c, to_addr)
//     }
// }
//
// impl MultiTransition {
//     fn new(vec: Vec<(u8, usize)>) -> Self {
//         MultiTransition(vec)
//     }
// }

#[derive(Debug, Clone, Encode, Decode)]
pub enum CompressedFsaState {
    NoTransitionsState(NoTransitions, bool),
    BasicState(Basic, bool),
    BasicWithNextState(BasicWithNext, bool),
    MultiTransitionState(MultiTransition, bool),
}

impl CompressedFsaState {
    #[inline(always)]
    fn new_no_trans(is_final: bool) -> Self {
        CompressedFsaState::NoTransitionsState(NoTransitions(0), is_final)
    }

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

    //
    //FIX: the tag should depend on whether the `to_state` is equal to next_addr
    //, where next_addr is the last written address.
    //If they are equal then it's a linear kind of path (basic shall be chosen)
    pub fn new_from_fsa_state<T: AutomatonState>(
        state: &T,
        next_state_addr: usize,
        next_state_tag: CompressedStateTag,
    ) -> Self {
        let is_final = state.is_final();

        match state.get_transitions() {
            [] => {
                assert_eq!(next_state_tag, CompressedStateTag::NotAvailableTag);
                assert!(is_final);
                Self::NoTransitionsState(NoTransitions(0), is_final)
            }
            [transition] => match next_state_tag {
                CompressedStateTag::NoTransTag | CompressedStateTag::BasicTag => {
                    let (on_char, to_state) = transition;

                    if *to_state == next_state_addr {
                        Self::new_basic(*on_char, is_final)
                    } else {
                        Self::new_basic_with_next(*on_char, *to_state, is_final)
                    }
                }
                _ => {
                    assert!(!is_final);
                    let (on_char, to_state) = transition;
                    Self::new_basic_with_next(*on_char, *to_state, is_final)
                }
            },
            //NOTE: don't like this .to_vec()
            transitions => Self::new_multi_transition(transitions.to_vec(), is_final),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompressedStateTag {
    NoTransTag,
    BasicTag,
    BasicWithNextStateTag,
    MultiTransitionTag,
    NotAvailableTag,
}

impl CompressedStateTag {
    pub fn get_tag_from_state<T: AutomatonState + WithAddress>(
        state: &T,
        last_written_addr: usize,
    ) -> Self {
        match state.get_transitions() {
            [] => Self::NoTransTag,
            [(_, _)] => {
                if state.get_address() == last_written_addr {
                    Self::BasicTag
                } else {
                    Self::BasicWithNextStateTag
                }
            }
            [_, _, ..] => Self::MultiTransitionTag,
        }
    }
}

#[test]
fn compressed_state_size() {
    let mut slice = [0u8; 10];

    let state = CompressedFsaState::new_no_trans(true);

    let length =
        bincode::encode_into_slice(&state, &mut slice, bincode::config::standard()).unwrap();

    assert_eq!(length, 3);

    let state = CompressedFsaState::new_basic(255, true);

    let length =
        bincode::encode_into_slice(&state, &mut slice, bincode::config::standard()).unwrap();

    assert_eq!(length, 3);
}
