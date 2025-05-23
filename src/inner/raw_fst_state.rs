use bincode::{Decode, Encode};

use crate::{automaton::AutomatonState, utils::common_prefix_index};

pub type TransitionOutput = Option<Vec<u8>>;

// #[derive(Debug, PartialEq, Clone, Encode, Decode)]
// pub struct TransitionOutput {
//     value: Option<Vec<u8>>,
// }
//
// impl TransitionOutput {
//     pub fn new() -> Self {
//         TransitionOutput { value: None }
//     }
//
//     fn new_with_value(value: Vec<u8>) -> Self {
//         TransitionOutput { value: Some(value) }
//     }
//
//     pub fn has_value(&self) -> bool {
//         return self.value.is_some();
//     }
//
//     pub fn get_value_as_deref(&self) -> Option<&[u8]> {
//         return self.value.as_deref();
//     }
//
//     ///panics if value is None
//     pub fn get_value_unwrapped(&self) -> &[u8] {
//         return self.get_value_as_deref().unwrap();
//     }
//
//     ///panics if value is None
//     pub fn drain_value(&mut self, from: usize) -> Vec<u8> {
//         if self.has_value() {
//             let suffix: Vec<_> = self.value.as_mut().unwrap().drain(from..).collect();
//             return suffix;
//         } else {
//             panic!("drain value called on state with output = None");
//         }
//     }
//
//     pub fn set_value(&mut self, value: Option<Vec<u8>>) {
//         self.value = value;
//     }
//
//     pub fn set_value_from_slice(&mut self, value: &[u8]) {
//         self.value = Some(value.to_vec());
//     }
//
//     ///panics if value is None
//     pub fn prepend_slice(&mut self, new_prefix: &[u8]) {
//         let value = self.value.as_mut().unwrap();
//         value.splice(..0, new_prefix.iter().cloned());
//     }
//
//     pub fn take(&mut self) -> Option<Vec<u8>> {
//         return self.value.take();
//     }
// }

#[derive(Debug, Clone, Encode, Decode)]
pub struct InnerFstState {
    is_final: bool,
    transitions: Vec<(u8, usize)>,
    outputs: Vec<TransitionOutput>,
}

impl InnerFstState {
    pub fn new(f: bool) -> Self {
        Self {
            is_final: f,
            transitions: Vec::new(),
            outputs: Vec::new(),
        }
    }

    ///Makes the last transition of current state point to other_state
    #[inline]
    pub fn merge_with(&mut self, other_state: usize) -> bool {
        self.set_last_transition_child(other_state)
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

    pub fn get_last_output(&self) -> Option<&[u8]> {
        assert!(!self.outputs.is_empty());
        let len = self.outputs.len();
        self.outputs[len - 1].as_deref()
    }

    //find common prefix between state last output and current str
    pub fn find_new_prefix_len(&self, current_str: &[u8]) -> usize {
        //find common prefix between initial state last output and current str
        if let Some(state_last_output) = self.get_last_output() {
            let prefix_index = common_prefix_index(current_str, state_last_output);

            return prefix_index;
        }
        return 0;
    }

    //get the index of the last prefix character insaddresse y
    fn common_prefix_index(previous_str: &[u8], y: &[u8]) -> usize {
        //TODO SIMD it using chunk_exact
        let prefix_len = previous_str
            .iter()
            .zip(y)
            .take_while(|(x, y)| x == y)
            .count();

        assert!(prefix_len <= y.len());
        return prefix_len;
    }

    #[inline]
    pub fn set_last_output(&mut self, output: &[u8]) {
        assert_eq!(self.transitions.len(), self.outputs.len());

        if let Some(last_output) = self.outputs.last_mut() {
            // *last_output = TransitionOutput::new_with_value(output.to_vec());
            *last_output = Some(output.to_vec());
        } else {
            panic!("output vec is empty!");
        }
        assert_eq!(self.transitions.len(), self.outputs.len());
    }

    /*
     * remove all characters that come after index `from` from last output.
     * Removed characters are returned. If no characters are removed, None is returned
     */
    #[inline]
    pub fn truncate_last_output(&mut self, from: usize) -> Option<Vec<u8>> {
        assert_eq!(self.transitions.len(), self.outputs.len());

        if let Some(last_output_edge) = self.outputs.last_mut() {
            match last_output_edge {
                Some(vec) => {
                    let suffix: Vec<_> = vec.drain(from..).collect();
                    //println!("after truncating state is: {:?}", self);

                    if suffix.len() != 0 {
                        return Some(suffix);
                    } else {
                        return None;
                    }
                }
                None => {
                    unreachable!();
                }
            }
        }
        assert_eq!(self.transitions.len(), self.outputs.len());
        None

        // if last_output_edge.is_some() {
        //     //truncate
        //     let suffix: Vec<_> = last_output_edge.drain(from);
        //     //println!("after truncating state is: {:?}", self);
        //
        //     if suffix.len() != 0 {
        //         return Some(suffix);
        //     } else {
        //         return None;
        //     }
        // }
        // } else {
        //     panic!("called truncate last output on empty vec!");
        // }
    }

    #[inline]
    pub fn clear_last_output(&mut self) -> Option<Vec<u8>> {
        assert_eq!(self.transitions.len(), self.outputs.len());
        if let Some(last_output_edge) = self.outputs.last_mut() {
            //take out the old value (to then return it) and replace it with None
            return last_output_edge.take();
        } else {
            panic!("output vec is empty!");
        }
    }

    //prepend new_prefix to all outputs
    pub fn propagate_new_prefix(&mut self, new_prefix: Option<&[u8]>) {
        //nothing to prepend
        if new_prefix == None {
            return;
        }

        let new_prefix = new_prefix.as_deref().unwrap();
        assert!(new_prefix.len() > 0);
        assert_eq!(self.transitions.len(), self.outputs.len());

        for out_edge in self.outputs.iter_mut() {
            //if output edge has some value then we prepend new_prefix to it
            match out_edge {
                Some(edge) => {
                    edge.splice(..0, new_prefix.iter().cloned());
                }
                None => {
                    *out_edge = Some(new_prefix.to_vec());
                }
            }
            // if out_edge.is_some() {
            //     // out_edge.prepend_slice(new_prefix);
            //
            //     let value = out_edge.as_mut().unwrap();
            //     value.splice(..0, new_prefix.iter().cloned());
            // } else {
            //     //replace entirely with new prefix is no edge has no value
            //     out_edge.set_value_from_slice(new_prefix);
            // }
        }
    }

    //prepend new_prefix to all outputs except the last one
    pub fn propagate_prefix_exc_last(&mut self, new_prefix: &Option<Vec<u8>>) {
        //nothing to prepend
        if *new_prefix == None {
            return;
        }

        let new_prefix = new_prefix.as_deref().unwrap();
        assert!(new_prefix.len() > 0);
        assert_eq!(self.transitions.len(), self.outputs.len());

        //skip the last outgoing edge
        for out_edge in self.outputs.iter_mut().rev().skip(1) {
            match out_edge {
                Some(edge) => {
                    edge.splice(..0, new_prefix.iter().cloned());
                }
                None => {
                    *out_edge = Some(new_prefix.to_vec());
                }
            }
            // if out_edge.has_value() {
            //     //if output edge has some value then we prepend new_prefix to it
            //     out_edge.prepend_slice(new_prefix);
            // } else {
            //     //if output edge is None then just copy the new prefix into it
            //     out_edge.set_value_from_slice(new_prefix);
            // }
        }
    }

    ///All transitions must be equal for this to return true.
    ///This means that each character must be associated with the same state
    pub fn are_transitions_equal(&self, other_state: &InnerFstState) -> bool {
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

    pub fn are_outputs_equal(&self, other_state: &InnerFstState) -> bool {
        let self_outputs = &self.outputs;
        let other_state_outputs = &other_state.outputs;

        let all_transitions_equal = self_outputs
            .iter()
            .zip(other_state_outputs)
            .all(|(t1, t2)| t1.eq(t2));

        return all_transitions_equal;
    }

    pub fn add_output(&mut self) {
        self.outputs.push(None);
    }
}

///Implement AutomatonState trait for InnerFsaState
impl AutomatonState for InnerFstState {
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
        Some(&self.outputs)
    }
}

impl PartialEq for InnerFstState {
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
