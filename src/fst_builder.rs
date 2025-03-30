use crate::automaton::AutomatonState;
use crate::fst::Transducer;
use crate::fst_state::common_prefix_index;
use crate::{automaton::Automaton, fst::FstError};

use core::result::Result;
use std::fs::File;
use std::io;
use std::io::Write;

pub fn build_transducer_from_collection(
    collection: Vec<(Vec<u8>, Vec<u8>)>,
) -> Result<Transducer, FstError> {
    assert!(collection.len() > 0);

    let mut transducer = Transducer::new();

    let mut previous_key_i = 0;

    transducer.push_zinit_state();

    assert_eq!(transducer.states.len(), 1);

    for (i, key_value_pair) in collection.iter().enumerate() {
        let (key, value) = key_value_pair;

        //key common prefix len
        let prefix_len = common_prefix_index(&transducer.last_key, &key);

        //key common prefix
        let prefix = &key[0..prefix_len];

        //compute delta* from initial state with the common prefix as input.
        //The reached state will be the one after which the suffix will be added.
        //Since the dataset is ordered, no other word will "touch" states that come after last_state.
        let last_state_index = transducer
            .delta_star_from_initial_state(prefix)
            .expect(&format!("transitioned to err state after checking common prefix!\nWas processing {:?} and common prefix was {:?}.\nLast str is: {:?}", std::str::from_utf8(key).unwrap(), std::str::from_utf8(prefix).unwrap(), std::str::from_utf8(&collection[previous_key_i].0).unwrap()));

        assert!(last_state_index < transducer.states.len());

        let last_state = &transducer.states[last_state_index];
        if last_state.has_children() {
            transducer.replace_or_register(last_state_index);
        }

        let suffix = &key[prefix_len..];
        transducer.add_suffix_to_fsa(last_state_index, suffix);

        //insert current value associated with current key in the fst
        transducer.insert_value(&value, last_state_index);

        transducer.update_last_key(key);

        previous_key_i = i;
    }

    transducer.replace_or_register(0);

    return Ok(transducer);
}

pub fn dump_graph(fst: &Transducer, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;

    file.write(b"digraph DFA {")?;
    file.write(b"    {0} [shape=circle, color=blue, label=\"start\"];")?;

    //mark the accepting states
    for (i, state) in fst.states.iter().enumerate() {
        if state.is_final {
            let s = format!("    {} [shape=doublecircle, color=green];", i);
            file.write(s.as_bytes())?;
        }
    }

    //add transitions
    for (i, state) in fst.states.iter().enumerate() {
        let state_transitions = &state.transitions;
        let state_outputs = &state.outputs;

        println!("state {i}");

        assert!(state_transitions.len() >= state_outputs.len());
        for (transition, output) in state_transitions.iter().zip(state_outputs) {
            let (on_char, to_state) = transition;

            let label: String;
            if let Some(value) = output {
                label = format!(
                    "{}: {}",
                    *on_char as char,
                    std::str::from_utf8(value).unwrap()
                );
            } else {
                label = format!("{}", *on_char as char);
            }
            let s = format!("    {} -> {} [label=\"{}\"];", i, *to_state, label);
            file.write(s.as_bytes())?;
        }
    }

    file.write(b"}")?;

    Ok(())
}

#[allow(unused)]
#[cfg(test)]
mod fst_unit_tests {

    use crate::fst_state::FstState;
    use crate::fst_state::FstStateOutput;

    use super::build_transducer_from_collection;
    use super::dump_graph;

    #[test]
    fn two_pairs() {
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("colle").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        //dump_graph(&fst, "fst.dot");
    }

    #[test]
    fn three_pairs() {
        println!("-----------------------------\n\n");
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("colle").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("pizza").as_bytes().to_vec(),
            String::from("zappa").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair, third_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        //dump_graph(&fst, "fst.dot");
    }

    #[test]
    fn four_pairs() {
        println!("-----------------------------\n\n");
        let first_pair = (
            String::from("ciao").as_bytes().to_vec(),
            String::from("rat").as_bytes().to_vec(),
        );

        let second_pair = (
            String::from("ciaspola").as_bytes().to_vec(),
            String::from("ratto").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("come").as_bytes().to_vec(),
            String::from("ronda").as_bytes().to_vec(),
        );

        let third_pair = (
            String::from("zoccolo").as_bytes().to_vec(),
            String::from("picciotto").as_bytes().to_vec(),
        );

        let collection = vec![first_pair, second_pair, third_pair];

        let Ok(fst) = build_transducer_from_collection(collection);

        dump_graph(&fst, "fst.dot");
    }
}
