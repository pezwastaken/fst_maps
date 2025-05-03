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

        let Some(current_state) = self.decoder.read_first_state::<InnerFsaState>() else {
            panic!("couldn't decode first state given mmapped file");
        };

        assert!(self.dula_state_stack.is_empty());
        self.dula_state_stack.push(0);

        for transition in current_state.get_transitions().iter() {
            self.visit(transition.0, transition.1, &dula, &mut accepted_str);
            assert_eq!(self.dula_state_stack.len(), 1);
        }

        assert_eq!(self.dula_state_stack.len(), 1);
        self.dula_state_stack.pop();

        true
    }

    fn visit(
        &mut self,
        c: u8,
        state_addr: usize,
        dula: &DULA<Max1>,
        output_str: &mut Vec<u8>,
    ) -> bool {
        let char_at = output_str.len();
        output_str.push(c);

        let input_bitvec = BitVector::from_str(char_at, self.ref_bitvec.get_bitvec(c));

        //TODO: there should be a check for final states
        //In case of a final state, the bitvec for the terminator char should
        // be generated instead
        let Some(dula_current_state) = self.dula_state_stack.last() else {
            unreachable!("should never have an empty stack of states");
        };

        //dula reached error state => prune the search tree
        let Some(dula_next_state) = dula.delta(*dula_current_state, &input_bitvec) else {
            self.dula_state_stack.pop();
            output_str.pop();
            return false;
        };

        self.dula_state_stack.push(dula_next_state);

        //get the current state (the acceptor one)
        let Some(current_state) = self.decoder.read_state_at_addr::<InnerFsaState>(state_addr)
        else {
            eprintln!("transition brought to an unreadable state at addr: {state_addr}");
            self.dula_state_stack.pop();
            output_str.pop();
            return false;
        };

        if current_state.is_final() {
            println!(
                "key: {} has distance <= 1",
                std::str::from_utf8(&output_str).unwrap()
            );
        }

        let tmp_1 = output_str.len();
        let tmp_2 = self.dula_state_stack.len();

        for out_transition in current_state.get_transitions() {
            let on_char = out_transition.0;
            let to_addr = out_transition.1;
            self.visit(on_char, to_addr, dula, output_str);
        }

        assert_eq!(output_str.len(), tmp_1);
        assert_eq!(self.dula_state_stack.len(), tmp_2);

        self.dula_state_stack.pop();
        output_str.pop();

        true
    }
}

#[test]
fn test_1() {
    let _ = build_fsa("sets_data/bag.txt", "sets/bag.fsa");
    let mut fuzzy_set = FuzzySet::new("sets/bag.fsa", "dog");
    println!("ciao");

    fuzzy_set.fuzzy_search();
}
