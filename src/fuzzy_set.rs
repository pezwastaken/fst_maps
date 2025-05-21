use crate::{
    automaton::AutomatonState,
    automaton_decoder::AutomatonDecoder,
    bit_vector::{BitVector, FullBitVectors},
    dula::{DULA, build_dula},
    fsa_builder::build_fsa,
    nula::{Max1, NULA},
    raw_fsa_state::InnerFsaState,
};

pub struct FuzzySet {
    decoder: AutomatonDecoder,
    ref_bitvec: FullBitVectors,
    dula_state_stack: Vec<usize>,
}

impl FuzzySet {
    pub fn new(fsa_file: &str, ref_str: &str) -> Self {
        match AutomatonDecoder::new(fsa_file) {
            Ok(tmp) => Self {
                decoder: tmp,
                ref_bitvec: FullBitVectors::new(ref_str.as_bytes()),
                dula_state_stack: Vec::with_capacity(4),
            },
            Err(e) => panic!("error while creating automaton decoder {e:?}"),
        }
    }

    pub fn fuzzy_search(&mut self) -> bool {
        let nfa = NULA::<Max1>::new();
        let dula = build_dula(&nfa);

        let mut accepted_str: Vec<u8> = Vec::new();

        let Some((current_state, at_addr)) = self.decoder.read_first_state::<InnerFsaState>()
        else {
            panic!("couldn't decode first state given mmapped file");
        };

        assert!(self.dula_state_stack.is_empty());
        self.dula_state_stack.push(0);

        //go through each transition belonging to state 0
        for transition in current_state.get_transitions().iter() {
            let on_char = transition.0;
            let to_state = transition.1;

            let associated_bitvec = BitVector::from_str(0, self.ref_bitvec.get_bitvec(on_char));

            //if reading char ``on_char`` brings the dula to a non-error state,
            //then continue visiting that subtree
            //
            if let Some(next_dula_state) = dula.delta(0, &associated_bitvec) {
                self.dula_state_stack.push(next_dula_state);
                accepted_str.push(on_char);

                self.visit(to_state, &dula, &mut accepted_str);

                self.backtrack(&mut accepted_str);
            };

            assert_eq!(self.dula_state_stack.len(), 1);
            assert_eq!(accepted_str.len(), 0);
        }

        //cleanup
        self.dula_state_stack.pop();
        // accepted_str.pop();

        true
    }

    #[inline]
    fn backtrack(&mut self, accepted_str: &mut Vec<u8>) {
        self.dula_state_stack.pop();
        accepted_str.pop();
    }

    fn visit(&mut self, state_addr: usize, dula: &DULA<Max1>, output_str: &mut Vec<u8>) {
        let char_at = output_str.len();

        //decode the current acceptor state
        let Some(current_state) = self.decoder.read_state_at_addr::<InnerFsaState>(state_addr)
        else {
            panic!("transition addr pointed to an invalid address!");
        };

        if current_state.is_final() {
            if self.check_terminator_char(char_at, dula) {
                dbg!(std::str::from_utf8(output_str).unwrap());
            }
        }

        for transition in current_state.get_transitions() {
            let on_char = transition.0;
            let to_state = transition.1;

            //get the bitvector associated with the transition character
            let associated_bitvec =
                BitVector::from_str(char_at, self.ref_bitvec.get_bitvec(on_char));

            let current_dula_state = self.dula_state_stack.last().unwrap();

            //if reading char ``on_char`` brings the dula to a non-error state,
            //then continue visiting this subtree
            if let Some(next_dula_state) = dula.delta(*current_dula_state, &associated_bitvec) {
                self.dula_state_stack.push(next_dula_state);
                output_str.push(on_char);

                self.visit(to_state, &dula, output_str);

                self.backtrack(output_str);
            };
        }
    }

    fn check_terminator_char(&self, str_len: usize, dula: &DULA<Max1>) -> bool {
        let mut dula_current_state = self.dula_state_stack.last().unwrap().clone();

        //FIX: horrendous way to get the length of ref_str
        let ref_len = self.ref_bitvec.get_bitvec(0).len() - 2;

        for i in 0..2 {
            let i = i + str_len;

            let input_bitvec = BitVector::terminator_bitvec(i, ref_len);

            let reached_state = dula.delta(dula_current_state, &input_bitvec);

            match reached_state {
                Some(to_state) => {
                    dula_current_state = to_state;
                }
                None => return false,
            }
        }
        true
    }
}
//
// #[test]
// fn test_1() {
//     let _ = build_fsa("sets_data/bag.txt", "sets/bag.fsa");
//     let mut fuzzy_set = FuzzySet::new("sets/bag.fsa", "dog");
//
//     fuzzy_set.fuzzy_search();
// }

#[test]
fn test_2() {
    let _ = build_fsa("sets_data/words_1.txt", "sets/words_1.fsa");
    let mut fuzzy_set = FuzzySet::new("sets/words_1.fsa", "kangaroo");

    fuzzy_set.fuzzy_search();
}
