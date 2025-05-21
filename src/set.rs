use crate::{automaton_decoder::AutomatonDecoder, encoded_states::CompressedFsaState};

pub struct Set {
    decoder: AutomatonDecoder,
    current_addr: usize,
}

impl Set {
    pub fn new(fsa_file: &str) -> Self {
        match AutomatonDecoder::new(fsa_file) {
            Ok(tmp) => Self {
                decoder: tmp,
                current_addr: 0,
            },
            Err(e) => panic!("error while creating automaton decoder {e:?}"),
        }
    }

    pub fn contains(&mut self, key: &[u8]) -> bool {
        self.delta_hat(key)
    }

    fn delta_hat(&mut self, input_tape: &[u8]) -> bool {
        self.current_addr = 0;

        let Some((mut current_state, at_addr)) =
            self.decoder.read_first_state::<CompressedFsaState>()
        else {
            panic!("couldn't decode first state given mmapped file");
        };

        dbg!(&current_state);
        self.current_addr = at_addr;

        for c in input_tape {
            let Some(next_state_addr) = current_state.delta(*c, self.current_addr) else {
                return false;
            };

            dbg!(next_state_addr);

            match self.decoder.read_state_at_addr(next_state_addr as usize) {
                Some(state) => current_state = state,
                None => return false,
            }

            dbg!(&current_state);
            self.current_addr = next_state_addr;
        }

        current_state.is_final()
    }
}
