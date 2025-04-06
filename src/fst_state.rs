use crate::automaton::AutomatonState;

#[derive(Hash, Debug, PartialEq)]
pub struct TransitionOutput {
    value: Option<Vec<u8>>,
}

impl TransitionOutput {
    pub fn new() -> Self {
        TransitionOutput { value: None }
    }

    fn new_with_value(value: Vec<u8>) -> Self {
        TransitionOutput { value: Some(value) }
    }

    pub fn has_value(&self) -> bool {
        return self.value.is_some();
    }

    pub fn get_value_as_deref(&self) -> Option<&[u8]> {
        return self.value.as_deref();
    }

    ///panics if value is None
    pub fn get_value_unwrapped(&self) -> &[u8] {
        return self.get_value_as_deref().unwrap();
    }

    ///panics if value is None
    pub fn drain_value(&mut self, from: usize) -> Vec<u8> {
        if self.has_value() {
            let suffix: Vec<_> = self.value.as_mut().unwrap().drain(from..).collect();
            return suffix;
        } else {
            panic!("drain value called on state with output = None");
        }
    }

    pub fn set_value(&mut self, value: Option<Vec<u8>>) {
        self.value = value;
    }

    pub fn set_value_from_slice(&mut self, value: &[u8]) {
        self.value = Some(value.to_vec());
    }

    ///panics if value is None
    pub fn prepend_slice(&mut self, new_prefix: &[u8]) {
        let value = self.value.as_mut().unwrap();
        value.splice(..0, new_prefix.iter().cloned());
    }

    pub fn take(&mut self) -> Option<Vec<u8>> {
        return self.value.take();
    }
}

#[derive(Debug, Hash)]
pub struct FstState {
    //id: usize,
    pub is_final: bool,

    //these two fields are "linked" together to model a transition of the transducer.
    //outputs[i] is the output associated to transition `i` of current state
    pub transitions: Vec<(u8, usize)>,
    //pub outputs: Vec<Option<Vec<u8>>>,
    pub outputs: Vec<TransitionOutput>,
}

impl FstState {
    fn new() -> Self {
        FstState {
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
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

    #[inline]
    pub fn get_last_output(&self) -> &TransitionOutput {
        //if let Some(out) = self.outputs.last() {
        //    return out.get_value_as_deref();
        //}
        //return None;
        return self.outputs.last().expect("outputs should never be empty!");
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

impl AutomatonState for FstState {
    #[inline]
    fn delta(&self, c: u8) -> Option<usize> {
        let transition = self
            .transitions
            .iter()
            .find(|(transition_char, _)| *transition_char == c);

        return transition.map(|(_, next_state)| *next_state);
    }

    fn delta_with_output(&self, c: u8) -> Option<(usize, &TransitionOutput)> {
        let transition = self
            .transitions
            .iter()
            .enumerate()
            .find(|(_, (transition_char, _))| *transition_char == c)
            .map(|(i, (ch, to_state))| (i, (*ch, *to_state)));

        //match the transition
        return match transition {
            //if a transition for char ch exists, then look for an output associated with it
            Some(edge) => {
                let (index, (_, to_state)) = edge;

                let outputs = &self.outputs;
                let output_on_transition = &outputs[index];

                Some((to_state, &output_on_transition))
            }
            None => None,
        };
    }

    //check whether current state has outgoing transitions
    #[inline]
    fn has_children(&self) -> bool {
        !(self.transitions.is_empty())
    }

    //get last child of current state
    //last is based on lexicographic order
    #[inline]
    fn last_child(&self) -> Option<usize> {
        match self.transitions.last() {
            Some((_, child_state)) => Some(*child_state),
            None => None,
        }
    }

    #[inline]
    fn add_transition(&mut self, y: u8, new_state_id: usize) {
        self.transitions.push((y, new_state_id));
        assert!(self.transitions.len() > 0);

        //create new empty output
        self.outputs.push(TransitionOutput::new());
        assert_eq!(self.transitions.len(), self.outputs.len());
    }

    #[inline]
    fn clear_state(&mut self) {
        self.is_final = false;
        self.transitions.clear();
        self.outputs.clear();
    }
}

//TODO should also use outputs?
//are both states in the same equivalence class
impl PartialEq for FstState {
    fn eq(&self, other_state: &Self) -> bool {
        if self.is_final != other_state.is_final {
            return false;
        }

        match self.transitions.len() != other_state.transitions.len() {
            true => return false,
            false => (),
        }

        let transitions_equal = self.are_transitions_equal(other_state);

        let outputs_equal = self.are_outputs_equal(other_state);

        return transitions_equal && outputs_equal;
    }
}

impl Eq for FstState {}

//get the index of the last prefix character inside y
pub(crate) fn common_prefix_index(previous_str: &[u8], y: &[u8]) -> usize {
    //TODO SIMD it using chunk_exact
    let prefix_len = previous_str
        .iter()
        .zip(y)
        .take_while(|(x, y)| x == y)
        .count();

    assert!(prefix_len <= y.len());
    return prefix_len;
}

pub fn bytes_to_utf8(bytes: &[u8]) -> &str {
    return std::str::from_utf8(bytes).unwrap();
}

pub fn common_prefix<'a>(previous_str: &'a [u8], y: &'a [u8]) -> &'a [u8] {
    //TODO SIMD it using chunk_exact
    let prefix_len = previous_str
        .iter()
        .zip(y)
        .take_while(|(x, y)| x == y)
        .count();

    assert!(prefix_len <= y.len());

    if previous_str.len() >= y.len() {
        return &previous_str[..prefix_len];
    }

    return &y[..prefix_len];
}

#[allow(unused)]
#[cfg(test)]
mod fst_state_unit_tests {
    use super::AutomatonState;
    use super::FstState;
    use super::TransitionOutput;

    #[test]
    fn output_eq_1() {
        let t_out_1 = TransitionOutput::new_with_value("ciao".as_bytes().into());
        let t_out_2 = TransitionOutput::new_with_value("ciao".as_bytes().into());

        assert!(t_out_1.eq(&t_out_2));
    }

    #[test]
    fn output_eq_2() {
        let t_out_1 = TransitionOutput::new_with_value("cia".as_bytes().into());
        let t_out_2 = TransitionOutput::new_with_value("cza".as_bytes().into());

        assert_ne!(t_out_1, t_out_2);
    }

    #[test]
    fn state_eq_1() {
        let mut state_1 = FstState::new();

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new();

        state_2.add_transition(100, 1);
        state_2.set_last_output("ciao".as_bytes());
        state_2.add_transition(102, 2);
        state_2.set_last_output("topo".as_bytes());
        state_2.add_transition(103, 3);
        state_2.set_last_output("pizza".as_bytes());

        assert_eq!(state_1, state_2);
    }

    #[test]
    fn state_eq_2() {
        let mut state_1 = FstState::new();

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new();

        state_2.add_transition(100, 1);
        state_2.set_last_output("ciao".as_bytes());
        state_2.add_transition(102, 2);
        state_2.set_last_output("top".as_bytes());
        state_2.add_transition(103, 3);
        state_2.set_last_output("pizza".as_bytes());

        assert_ne!(state_1, state_2);
    }

    #[test]
    fn state_eq_3() {
        let mut state_1 = FstState::new();

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new();

        state_2.add_transition(100, 1);
        state_2.set_last_output("ciao".as_bytes());
        state_2.add_transition(102, 2);
        state_2.set_last_output("topo".as_bytes());

        assert_ne!(state_1, state_2);
    }

    #[test]
    fn prefix_len() {
        let mut state = FstState::new();

        state.add_transition(100, 1);
        state.set_last_output(&"ciao".as_bytes());
        state.add_transition(101, 2);
        state.set_last_output(&"topo".as_bytes());

        //let suffix = state.truncate_last_output(2).unwrap();
        let prefix_len = state.find_new_prefix_len("to".as_bytes());
        assert_eq!(prefix_len, 2);
        let prefix_len = state.find_new_prefix_len("topo".as_bytes());
        assert_eq!(prefix_len, 4);
        let prefix_len = state.find_new_prefix_len("topone".as_bytes());
        assert_eq!(prefix_len, 4);
    }

    #[test]
    fn clear_last_out() {
        let mut state = FstState::new();

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());

        let old_out_vec = state.clear_last_output().unwrap();
        assert!(old_out_vec.len() == 4);
        assert_eq!(old_out_vec[0] as char, 'c');
        assert_eq!(old_out_vec[3] as char, 'o');
        assert_eq!(state.get_last_output().get_value_as_deref(), None);
    }

    #[test]
    fn drain_last_out() {
        let mut state = FstState::new();

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());

        let suffix = state.truncate_last_output(2).unwrap();

        assert!(suffix.len() == 2);
        assert_eq!(suffix[0] as char, 'a');
        assert_eq!(suffix[1] as char, 'o');
    }

    #[test]
    fn propagate_prefix() {
        let mut state = FstState::new();

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());
        state.add_transition(102, 2);
        state.set_last_output("topo".as_bytes());
        state.add_transition(103, 3);
        state.set_last_output("pizza".as_bytes());

        state.propagate_new_prefix(Some("ratto ".as_bytes()));

        let output = state.outputs[0].get_value_unwrapped();
        assert_eq!(output, "ratto ciao".as_bytes());
        let output = state.outputs[1].get_value_unwrapped();
        assert_eq!(output, "ratto topo".as_bytes());
        let output = state.outputs[2].get_value_unwrapped();
        assert_eq!(output, "ratto pizza".as_bytes());
    }
}
