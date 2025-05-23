use crate::{
    automaton::{AutomatonState, WithAddress},
    raw_fst_state::{InnerFstState, TransitionOutput},
};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct FstState {
    //address: usize,
    pub address: usize,

    //these two fields are "linked" together to model a transition of the transducer.
    //outputs[i] is the output associated to transition `i` of current state
    inner: InnerFstState,
}

impl FstState {
    pub fn new(addr: usize, f: bool) -> Self {
        FstState {
            address: addr,
            inner: InnerFstState::new(f),
            // is_final: false,
            // transitions: Vec::new(),
            // outputs: Vec::new(),
        }
    }

    pub fn set_last_transition_child(&mut self, to_state: usize) {
        self.inner.set_last_transition_child(to_state);
    }

    #[inline]
    pub fn merge_with(&mut self, other_state: usize) -> bool {
        self.inner.merge_with(other_state)
    }

    #[inline]
    pub fn get_last_output(&self) -> Option<&[u8]> {
        //if let Some(out) = self.outputs.last() {
        //    return out.get_value_as_deref();
        //}
        //return None;
        // return self.outputs.last().expect("outputs should never be empty!");
        self.inner.get_last_output()
    }

    //find common prefix between state last output and current str
    pub fn find_new_prefix_len(&self, current_str: &[u8]) -> usize {
        self.inner.find_new_prefix_len(current_str)
    }

    #[inline]
    pub fn set_last_output(&mut self, output: &[u8]) {
        self.inner.set_last_output(output);
    }

    /*
     * remove all characters that come after index `from` from last output.
     * Removed characters are returned. If no characters are removed, None is returned
     */
    #[inline]
    pub fn truncate_last_output(&mut self, from: usize) -> Option<Vec<u8>> {
        self.inner.truncate_last_output(from)
    }

    #[inline]
    pub fn clear_last_output(&mut self) -> Option<Vec<u8>> {
        self.inner.clear_last_output()
    }

    //prepend new_prefix to all outputs
    pub fn propagate_new_prefix(&mut self, new_prefix: Option<&[u8]>) {
        self.inner.propagate_new_prefix(new_prefix);
    }

    //prepend new_prefix to all outputs except the last one
    pub fn propagate_prefix_exc_last(&mut self, new_prefix: &Option<Vec<u8>>) {
        self.inner.propagate_prefix_exc_last(new_prefix);
    }

    fn are_transitions_equal(&self, other_state: &InnerFstState) -> bool {
        self.inner.are_transitions_equal(other_state)
    }

    fn are_outputs_equal(&self, other_state: &InnerFstState) -> bool {
        self.inner.are_outputs_equal(other_state)
    }

    fn add_output(&mut self) {
        self.inner.add_output();
    }
}

impl AutomatonState for FstState {
    #[inline]
    fn delta(&self, c: u8) -> Option<usize> {
        let transition = self
            .get_transitions()
            .iter()
            .find(|(transition_char, _)| *transition_char == c);

        return transition.map(|(_, next_state)| *next_state);
    }

    fn delta_with_output(&self, c: u8) -> Option<(usize, &TransitionOutput)> {
        let transition = self
            .get_transitions()
            .iter()
            .enumerate()
            .find(|(_, (transition_char, _))| *transition_char == c)
            .map(|(i, (ch, to_state))| (i, (*ch, *to_state)));

        //match the transition
        return match transition {
            //if a transition for char ch exists, then look for an output associated with it
            Some(edge) => {
                let (index, (_, to_state)) = edge;

                let outputs = self.inner.get_outputs().unwrap();
                let output_on_transition = &outputs[index];

                Some((to_state, &output_on_transition))
            }
            None => None,
        };
    }

    //check whether current state has outgoing transitions
    #[inline]
    fn has_children(&self) -> bool {
        !(self.get_transitions().is_empty())
    }

    //get last child of current state
    //last is based on lexicographic order
    #[inline]
    fn last_child(&self) -> Option<usize> {
        match self.get_transitions().last() {
            Some((_, child_state)) => Some(*child_state),
            None => None,
        }
    }

    #[inline]
    fn add_transition(&mut self, y: u8, new_state_address: usize) {
        self.inner.add_transition(y, new_state_address);
        assert!(self.get_transitions().len() > 0);

        //create new empty output

        self.add_output();
        assert_eq!(
            self.get_transitions().len(),
            self.get_outputs().unwrap().len()
        );
    }

    #[inline]
    fn clear_state(&mut self) {
        self.inner.clear_state();
    }

    //fn get_address(&self) -> usize {
    //    self.address
    //}

    fn is_final(&self) -> bool {
        self.inner.is_final()
    }

    fn get_transitions(&self) -> &[(u8, usize)] {
        self.inner.get_transitions()
    }

    fn get_outputs(&self) -> Option<&[TransitionOutput]> {
        self.inner.get_outputs()
    }
}

impl WithAddress for FstState {
    fn get_address(&self) -> usize {
        self.address
    }
}

//are both states in the same equivalence class
impl PartialEq for FstState {
    fn eq(&self, other_state: &Self) -> bool {
        if self.is_final() != other_state.is_final() {
            return false;
        }

        match self.get_transitions().len() != other_state.get_transitions().len() {
            true => return false,
            false => (),
        }

        let transitions_equal = self.inner.are_transitions_equal(&other_state.inner);

        let outputs_equal = self.inner.are_outputs_equal(&other_state.inner);

        let res = transitions_equal && outputs_equal;

        return res;
    }
}

impl Eq for FstState {}

pub fn bytes_to_utf8(bytes: &[u8]) -> &str {
    return std::str::from_utf8(bytes).unwrap();
}

#[allow(unused)]
#[cfg(test)]
mod fst_state_unit_tests {
    use super::AutomatonState;
    use super::FstState;
    use super::TransitionOutput;

    #[test]
    fn output_eq_1() {
        let t_out_1 = Some(vec![b"ciao"]);
        let t_out_2 = Some(vec![b"ciao"]);

        assert!(t_out_1.eq(&t_out_2));
    }

    #[test]
    fn output_eq_2() {
        let t_out_1 = Some(vec![b"cia"]);
        let t_out_2 = Some(vec![b"cza"]);

        assert_ne!(t_out_1, t_out_2);
    }

    #[test]
    fn state_eq_1() {
        let mut state_1 = FstState::new(0, false);

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new(0, false);

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
        let mut state_1 = FstState::new(0, false);

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new(0, false);

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
        let mut state_1 = FstState::new(0, false);

        state_1.add_transition(100, 1);
        state_1.set_last_output("ciao".as_bytes());
        state_1.add_transition(102, 2);
        state_1.set_last_output("topo".as_bytes());
        state_1.add_transition(103, 3);
        state_1.set_last_output("pizza".as_bytes());

        let mut state_2 = FstState::new(0, false);

        state_2.add_transition(100, 1);
        state_2.set_last_output("ciao".as_bytes());
        state_2.add_transition(102, 2);
        state_2.set_last_output("topo".as_bytes());

        assert_ne!(state_1, state_2);
    }

    #[test]
    fn prefix_len() {
        let mut state = FstState::new(0, false);

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
        let mut state = FstState::new(0, false);

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());

        let old_out_vec = state.clear_last_output().unwrap();
        assert!(old_out_vec.len() == 4);
        assert_eq!(old_out_vec[0] as char, 'c');
        assert_eq!(old_out_vec[3] as char, 'o');
        assert_eq!(state.get_last_output(), None);
    }

    #[test]
    fn drain_last_out() {
        let mut state = FstState::new(0, false);

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());

        let suffix = state.truncate_last_output(2).unwrap();

        assert!(suffix.len() == 2);
        assert_eq!(suffix[0] as char, 'a');
        assert_eq!(suffix[1] as char, 'o');
    }

    #[test]
    fn propagate_prefix() {
        let mut state = FstState::new(0, false);

        state.add_transition(100, 1);
        state.set_last_output("ciao".as_bytes());
        state.add_transition(102, 2);
        state.set_last_output("topo".as_bytes());
        state.add_transition(103, 3);
        state.set_last_output("pizza".as_bytes());

        state.propagate_new_prefix(Some("ratto ".as_bytes()));

        let output = &state.get_outputs().unwrap()[0];
        assert_eq!(output.as_deref().unwrap(), b"ratto ciao");
        let output = &state.get_outputs().unwrap()[1];
        assert_eq!(output.as_deref().unwrap(), b"ratto topo");
        let output = &state.get_outputs().unwrap()[2];
        assert_eq!(output.as_deref().unwrap(), b"ratto pizza");
    }
}
