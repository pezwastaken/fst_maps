use crate::fst_state::TransitionOutput;

pub trait Automaton {
    fn get(&self, input_tape: &[u8]) -> Option<Vec<u8>>;

    fn is_final_state(&self, state: usize) -> bool;
}

pub trait AutomatonState {
    fn delta(&self, c: u8) -> Option<usize>;

    fn delta_with_output(&self, c: u8) -> Option<(usize, &TransitionOutput)>;

    fn has_children(&self) -> bool;
    fn last_child(&self) -> Option<usize>;

    fn add_transition(&mut self, y: u8, new_state_id: usize);

    fn clear_state(&mut self);
}
