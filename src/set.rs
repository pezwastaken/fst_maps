use crate::{
    automaton::AutomatonState, automaton_decoder::AutomatonDecoder, raw_fsa_state::InnerFsaState,
};

pub struct Set {
    decoder: AutomatonDecoder,
}

impl Set {
    pub fn new(fsa_file: &str) -> Self {
        match AutomatonDecoder::new(fsa_file) {
            Ok(tmp) => Self { decoder: tmp },
            Err(e) => panic!("error while creating automaton decoder {e:?}"),
        }
    }

    pub fn contains(&mut self, key: &[u8]) -> bool {
        self.delta_hat(key)
    }

    fn delta_hat(&mut self, input_tape: &[u8]) -> bool {
        let Some(mut current_state) = self.decoder.read_first_state() else {
            panic!("couldn't decode first state given mmapped file");
        };

        for c in input_tape {
            let Some(next_state_addr) = current_state.delta(*c) else {
                return false;
            };

            match self.decoder.read_state_at_addr(next_state_addr as usize) {
                Some(state) => current_state = state,
                None => return false,
            }
        }

        current_state.is_final()
    }
}
