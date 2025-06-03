use crate::automaton::AutomatonState;
use crate::fst::Transducer;
use crate::utils::common_prefix_index;

use core::result::Result;
use std::fs::File;
use std::io;
use std::io::BufRead;

//TODO empty files are not handled !?
//
///Calling this function results in a fst being built and persisted to file
pub fn build_transducer_from_collection<'a>(
    //collection: Vec<(Vec<u8>, Vec<u8>)>,
    //input_stream: &mut T,
    filename: &str,
    output_to: &'a str,
) -> bool {
    let mut transducer = Transducer::new(output_to);

    transducer.push_zinit_state();

    assert_eq!(transducer.temp_states.len(), 1);

    //for item in input_stream {
    if let Ok(lines) = read_lines(filename) {
        for line in lines.map_while(Result::ok) {
            //convert lines to bytes
            let bytes: &[u8] = &line.as_bytes();

            assert_ne!(bytes.len(), 0);

            let (key, value): (&[u8], &[u8]) =
                parse_line(bytes).expect("No lines should be empty!");

            //println!("processing key: {}", std::str::from_utf8(key).unwrap());

            //TODO should also make sure the key has not been processed in the past
            assert_ne!(key, &transducer.last_key[..transducer.last_key_len]);

            //key common prefix len
            let prefix_len = common_prefix_index(&transducer.last_key, &key);

            let last_state_index = prefix_len;

            let seen_final_state_index = transducer.temp_contains_final(last_state_index);

            assert!(last_state_index < transducer.temp_states.len());

            let last_state = &transducer.temp_states[last_state_index];
            if last_state.has_children() {
                transducer.replace_or_register(last_state_index);
            }

            transducer.insert_pair(
                key,
                value,
                prefix_len,
                last_state_index,
                seen_final_state_index,
            );
        }
    }

    transducer.replace_or_register(0);
    transducer.states.push(transducer.temp_states[0].clone());

    transducer
        .persist_initial_state()
        .expect("couldn't serialize initial state");

    true
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;

    Ok(io::BufReader::new(file).lines())
}

///parse a line and return two slices:
/// - key
/// - value
///they are separated by the character at the beginning of the line, which should never appear in
///the actual content.
///
///Example
/// /hello, this is the key/here's the value
fn parse_line(line: &[u8]) -> Option<(&[u8], &[u8])> {
    assert!(!line.is_empty());
    let delimiter = line.get(0)?;

    let line = &line[1..];
    let key_up_to = line.iter().position(|x| *x == *delimiter)?;

    assert!(key_up_to < line.len());

    let key = &line[..key_up_to];
    let value = &line[key_up_to + 1..];

    Some((key, value))
}

// pub fn dump_graph(fst: &Transducer<'_>, file_path: &str) -> io::Result<()> {
//     println!("states in dump graph");
//     //dbg!(&fst.temp_states[0]);
//     //dbg!(&fst.states);
//
//     let mut file = File::create(file_path)?;
//
//     file.write(b"digraph DFA {")?;
//     file.write(b"    {0} [shape=circle, color=blue, label=\"start\"];")?;
//
//     //mark the accepting states
//     for state in fst.states.iter() {
//         if state.is_final() {
//             let s = format!("    {} [shape=doublecircle, color=green];", state.address);
//             file.write(s.as_bytes())?;
//         }
//     }
//
//     //add transitions
//     for state in fst.states.iter() {
//         let state_transitions = state.get_transitions();
//         let state_outputs = state.get_outputs();
//
//         assert!(state_transitions.len() >= state_outputs.len());
//         for (transition, output) in state_transitions.iter().zip(state_outputs) {
//             let (on_char, to_state) = transition;
//
//             let label: String;
//             if let Some(value) = output.get_value_as_deref() {
//                 label = format!(
//                     "{}: {}",
//                     *on_char as char,
//                     std::str::from_utf8(value).unwrap()
//                 );
//             } else {
//                 label = format!("{}", *on_char as char);
//             }
//             let s = format!(
//                 "    {} -> {} [label=\"{}\"];",
//                 state.address, *to_state, label
//             );
//             file.write(s.as_bytes())?;
//         }
//     }
//
//     file.write(b"}")?;
//
//     Ok(())
// }

#[allow(unused)]
#[cfg(test)]
mod fst_builder_tests {

    use std::io::BufRead;
    use std::os::unix::thread;
    use std::rc::Rc;

    use crate::automaton::Automaton;
    use crate::fst::Transducer;
    use crate::fst_state::FstState;

    use super::build_transducer_from_collection;
    //use super::dump_graph;
    use super::parse_line;
    use super::read_lines;
}
