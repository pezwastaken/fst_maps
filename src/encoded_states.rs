use bincode::{Decode, Encode};

use crate::automaton::{AutomatonState, WithAddress};

const BASIC_STATE_SIZE: usize = 3;

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
    pub fn delta(&self, c: u8, current_addr: usize) -> Option<usize> {
        match self {
            Self::NoTransitionsState(_, _) => None,
            Self::BasicState(Basic(on_char), _) => {
                if c == *on_char {
                    Some(current_addr - BASIC_STATE_SIZE)
                } else {
                    None
                }
            }
            Self::BasicWithNextState(BasicWithNext(on_char, to_state), _) => {
                if c == *on_char {
                    Some(*to_state)
                } else {
                    None
                }
            }
            Self::MultiTransitionState(MultiTransition(vec), _) => {
                let transition = vec
                    .iter()
                    .find(|(transition_char, _)| *transition_char == c);

                //return Some(transition?.1);
                return transition.map(|(_, next_state)| *next_state);
            }
        }
    }

    #[inline(always)]
    pub fn is_final(&self) -> bool {
        match self {
            CompressedFsaState::NoTransitionsState(_, _) => true,
            _ => false,
        }
    }

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

    pub fn new_from_fsa_state<T: AutomatonState>(
        state: &T,
        was_previous_merged: bool,
        next_state_tag: CompressedStateTag,
    ) -> Self {
        let is_final = state.is_final();

        match state.get_transitions() {
            [] => {
                assert_eq!(next_state_tag, CompressedStateTag::NotAvailableTag);
                assert!(is_final);
                Self::new_no_trans(is_final)
            }
            [transition] => match next_state_tag {
                CompressedStateTag::NoTransTag | CompressedStateTag::BasicTag => {
                    //if linear path => basic is enough
                    if !was_previous_merged {
                        let (on_char, _) = transition;
                        Self::new_basic(*on_char, is_final)
                    } else {
                        let (on_char, to_state) = transition;
                        Self::new_basic_with_next(*on_char, *to_state, is_final)
                    }
                }
                _ => {
                    let (on_char, to_state) = transition;
                    Self::new_basic_with_next(*on_char, *to_state, is_final)
                }
            },
            //NOTE: don't like this .to_vec(), rc during construction maybe?
            transitions => Self::new_multi_transition(transitions.to_vec(), is_final),
        }
    }
}

// impl AutomatonState for CompressedFsaState {
//     fn delta(&self, c: u8, current_addr: usize) -> Option<usize> {
//         match self {
//             Self::NoTransitionsState(_, _) => None,
//             Self::BasicState(Basic(on_char), _) => {
//                 if c == *on_char {
//                     Some(current_addr + BASIC_STATE_SIZE)
//                 } else {
//                     None
//                 }
//             }
//             Self::BasicWithNextState(BasicWithNext(on_char, to_state), _) => {
//                 if c == *on_char {
//                     Some(*to_state)
//                 } else {
//                     None
//                 }
//             }
//             Self::MultiTransitionState(MultiTransition(vec), _) => {
//                 let transition = vec
//                     .iter()
//                     .find(|(transition_char, _)| *transition_char == c);
//
//                 //return Some(transition?.1);
//                 return transition.map(|(_, next_state)| *next_state);
//             }
//         }
//     }
//
//     fn delta_with_output(&self, _: u8) -> Option<(usize, &crate::fst_state::TransitionOutput)> {
//         unreachable!("fsa transitions don't have output associated with them!");
//     }
//
//     fn has_children(&self) -> bool {
//         match self {
//             CompressedFsaState::NoTransitionsState(_, _) => false,
//             _ => true,
//         }
//     }
//
//     fn last_child(&self) -> Option<usize> {
//         match self {
//             CompressedFsaState::NoTransitionsState(_, _) => None,
//             CompressedFsaState::BasicState(basic, _) => todo!(),
//             CompressedFsaState::BasicWithNextState(basic_with_next, _) => todo!(),
//             CompressedFsaState::MultiTransitionState(multi_transition, _) => todo!(),
//         }
//     }
//
//     fn add_transition(&mut self, y: u8, new_state_address: usize) {
//         todo!()
//     }
//
//     fn clear_state(&mut self) {
//         todo!()
//     }
//
//     fn is_final(&self) -> bool {
//         todo!()
//     }
//
//     fn get_transitions(&self) -> &[(u8, usize)] {
//         todo!()
//     }
//
//     fn get_outputs(&self) -> Option<&[crate::fst_state::TransitionOutput]> {
//         todo!()
//     }
// }

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
