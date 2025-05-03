use crate::{
    automaton::AutomatonState,
    automaton_decoder::AutomatonDecoder,
    dula::{DULA, build_dula},
    nula::{Max1, NULA},
    raw_fsa_state::InnerFsaState,
    set::Set,
};

pub struct FuzzySet<'a> {
    decoder: AutomatonDecoder,
    ref_str: &'a str,
}

impl<'a> FuzzySet<'a> {
    pub fn new(fsa_file: &str) -> Self {
        match AutomatonDecoder::new(fsa_file) {
            Ok(tmp) => Self {
                decoder: tmp,
                ref_str: "",
            },
            Err(e) => panic!("error while creating automaton decoder {e:?}"),
        }
    }

    pub fn set_ref_word(&mut self, ref_str: &'a str) {
        self.ref_str = ref_str;
    }

    pub fn fuzzy_search(&mut self, ref_word: &str) -> bool {
        let nfa = NULA::<Max1>::new();
        let dula = build_dula(&nfa);

        let mut accepted_str: Vec<u8> = Vec::with_capacity(ref_word.len());

        let Some(current_state) = self.decoder.read_first_state::<InnerFsaState>() else {
            panic!("couldn't decode first state given mmapped file");
        };

        for transition in current_state.get_transitions().iter() {
            self.visit(transition.0, transition.1, &dula, &mut accepted_str);
        }

        // for c in input_tape {
        //     let Some(next_state_addr) = current_state.delta(*c) else {
        //         return false;
        //     };
        //
        //     match self.decoder.read_state_at_addr(next_state_addr as usize) {
        //         Some(state) => current_state = state,
        //         None => return false,
        //     }
        //     //dbg!(&current_state);
        // }
        true
    }

    fn visit(
        &mut self,
        c: u8,
        state_addr: usize,
        dula: &DULA<Max1>,
        output_str: &mut Vec<u8>,
    ) -> bool {
        let Some(state) = self.decoder.read_state_at_addr::<InnerFsaState>(state_addr) else {
            return false;
        };

        output_str.push(c);

        //if state is final then output_str must contain a valid word.
        //Let the dula check its distance to the ref_word.
        if state.is_final() {
            dula.delta_star(&output_str, self.ref_str);
        }

        output_str.pop();

        true
    }
}
