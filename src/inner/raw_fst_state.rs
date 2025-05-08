#[derive(Debug, Clone, Encode, Decode)]
pub struct InnerFstState {
    pub is_final: bool,
    pub transitions: Vec<(u8, usize)>,
    pub outputs: Vec<TransitionOutput>,
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

    //find common prefix between state last output and current str
    pub fn find_new_prefix_len(&self, current_str: &[u8]) -> usize {
        //find common prefix between initial state last output and current str
        if let Some(state_last_output) = self.get_last_output().get_value_as_deref() {
            let prefix_index = common_prefix_index(current_str, state_last_output);

            return prefix_index;
        }
        return 0;
    }

    #[inline]
    pub fn set_last_output(&mut self, output: &[u8]) {
        assert_eq!(self.transitions.len(), self.outputs.len());

        if let Some(last_output) = self.outputs.last_mut() {
            *last_output = TransitionOutput::new_with_value(output.to_vec());
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

        //println!("called truncate_last_output on state: {:?}", self);

        //println!("should truncate from char: {from}");

        if let Some(last_output_edge) = self.outputs.last_mut() {
            if last_output_edge.has_value() {
                //truncate
                let suffix: Vec<_> = last_output_edge.drain_value(from);
                //println!("after truncating state is: {:?}", self);

                if suffix.len() != 0 {
                    return Some(suffix);
                } else {
                    return None;
                }
            }
        } else {
            panic!("called truncate last output on empty vec!");
        }
        assert_eq!(self.transitions.len(), self.outputs.len());
        None
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
            if out_edge.has_value() {
                out_edge.prepend_slice(new_prefix);
            } else {
                //replace entirely with new prefix is no edge has no value
                out_edge.set_value_from_slice(new_prefix);
            }
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
            if out_edge.has_value() {
                //if output edge has some value then we prepend new_prefix to it
                out_edge.prepend_slice(new_prefix);
            } else {
                //if output edge is None then just copy the new prefix into it
                out_edge.set_value_from_slice(new_prefix);
            }
        }
    }

    fn are_transitions_equal(&self, other_state: &Self) -> bool {
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

    fn are_outputs_equal(&self, other_state: &FstState) -> bool {
        let self_outputs = &self.outputs;
        let other_state_outputs = &other_state.outputs;

        let all_transitions_equal = self_outputs
            .iter()
            .zip(other_state_outputs)
            .all(|(t1, t2)| t1.eq(t2));

        return all_transitions_equal;
    }
}
