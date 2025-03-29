#[derive(Hash, Debug)]

pub enum FstStateOutput {
    None,
    SetOfStrings(String),
}

#[derive(Debug, Hash)]
pub struct FstState {
    //id: usize,
    pub is_final: bool,

    //these two fields are "linked" together to model a transition of the transducer.
    //outputs[i] is the output associated to transition `i` of current state
    pub transitions: Vec<(u8, usize)>,
    pub outputs: Vec<String>,
    pub state_output: FstStateOutput,
}

impl FstState {
    fn new() -> Self {
        FstState {
            is_final: false,
            transitions: Vec::new(),
            outputs: Vec::new(),
            state_output: FstStateOutput::None,
        }
    }

    #[inline]
    pub fn delta(&self, c: u8) -> Option<usize> {
        //self.transitions.iter().position(|x| x.0 == c);

        let transition = self
            .transitions
            .iter()
            .find(|(transition_char, _)| *transition_char == c);

        //return Some(transition?.1);
        return transition.map(|(_, next_state)| *next_state);
    }

    //check whether current state has outgoing transitions
    #[inline]
    pub fn has_children(&self) -> bool {
        !(self.transitions.is_empty())
    }

    //get last child of current state
    //last is based on lexicographic order
    #[inline]
    pub fn last_child(&self) -> Option<usize> {
        match self.transitions.last() {
            Some((_, child_state)) => Some(*child_state),
            None => None,
        }
        //match self.transitions.last_key_value() {
        //    Some((_, next_state)) => Some(*next_state),
        //    None => None,
        //}
    }

    #[inline]
    pub fn add_transition(&mut self, y: u8, new_state_id: usize) {
        //assert!(self.id < new_state_id);
        self.transitions.push((y, new_state_id));
        assert!(self.transitions.len() > 0);
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

    //TODO should push or set
    #[inline]
    pub fn set_output(&mut self, transition_index: usize, output: String) {
        assert_eq!(self.transitions.len(), self.outputs.len());
        self.outputs[transition_index] = output;
    }

    //only for final states
    //TODO
    #[inline]
    pub fn set_state_output(&mut self, s: String) {
        assert!(self.is_final);
        self.state_output = FstStateOutput::SetOfStrings(s);
    }

    #[inline]
    pub fn clear_state(&mut self) {
        self.is_final = false;
        self.transitions.clear();
        self.outputs.clear();
    }
}
