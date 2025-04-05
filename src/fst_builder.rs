use rand::distributions::Alphanumeric;
use rand::thread_rng;

use crate::automaton::AutomatonState;
use crate::fst::FstError;
use crate::fst::Transducer;
use crate::fst_state::common_prefix_index;

use core::result::Result;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;

struct StringGenerator {
    count: usize,
    n_of_pairs: usize,
    max_key_len: usize,
    max_value_len: usize,
    delimiter: u8,
}

impl StringGenerator {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    fn new(n: usize, key_len: usize, value_len: usize, delim: u8) -> Self {
        Self {
            count: 0,
            n_of_pairs: n,
            max_key_len: key_len,
            max_value_len: value_len,
            delimiter: delim,
        }
    }

    fn random_string_gen(&self, len: usize) -> Vec<u8> {
        //https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html#create-random-passwords-from-a-set-of-alphanumeric-characters
        return thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(u8::from)
            .collect();
    }
}

impl Iterator for StringGenerator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.n_of_pairs {
            //final random pair
            let mut pair: Vec<u8> = Vec::with_capacity(self.max_value_len + self.max_key_len + 2);
            pair.push(self.delimiter);

            //random key
            let mut rand_key = self.random_string_gen(self.max_key_len);
            pair.append(&mut rand_key);

            //delimiter
            pair.push(self.delimiter);

            //random value
            let mut rand_value = self.random_string_gen(self.max_value_len);
            pair.append(&mut rand_value);

            //println!(
            //    "random pair generated: {}",
            //    std::str::from_utf8(&pair).unwrap()
            //);

            self.count += 1;

            Some(pair)
        } else {
            None
        }
    }
}

struct FileReader<'a> {
    //this will contain a line
    buff: Vec<u8>,
    reader: Option<io::BufReader<File>>,
    filename: &'a str,
}

impl<'a> FileReader<'a> {
    fn new(f: &'a str) -> Self {
        Self {
            buff: Vec::new(),
            reader: None,
            filename: f,
        }
    }

    fn read_file(&mut self) {
        let file = File::open(self.filename).expect("couldn't open given file");
        self.reader = Some(io::BufReader::new(file));
    }
}

//impl<'a> Iterator for FileReader<'a> {
//    type Item = &'a [u8];
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.buff.clear();
//        let x = self.reader?.read_until(0x0A, &mut self.buff);
//        match x {
//            Ok(_) => Some(&self.buff),
//            _ => None,
//        }
//    }
//}

//TODO empty files are not handled !?
pub fn build_transducer_from_collection(
    //collection: Vec<(Vec<u8>, Vec<u8>)>,
    //input_stream: &mut T,
    filename: &str,
) -> Result<Transducer, FstError>
//where
    //T: Iterator<Item = Vec<u8>>,
{
    let mut transducer = Transducer::new();

    transducer.push_zinit_state();

    assert_eq!(transducer.states.len(), 1);

    //for item in input_stream {
    if let Ok(lines) = read_lines(filename) {
        //for line in lines.filter(|x| x.is_ok()) {
        for line in lines.map_while(Result::ok) {
            //convert lines to bytes
            let bytes: &[u8] = &line.as_bytes();

            let (key, value): (&[u8], &[u8]) =
                parse_line(bytes).expect("No lines should be empty!");

            //TODO
            //add check on key

            //key common prefix len
            let prefix_len = common_prefix_index(&transducer.last_key, &key);

            //key common prefix
            let prefix = &key[0..prefix_len];

            //compute delta* from initial state with the common prefix as input.
            //The reached state will be the one after which the suffix will be added.
            //Since the dataset is ordered, no other word will "touch" states that come after last_state.
            let (last_state_index, seen_final_state_index) =
                transducer.delta_star_final_state(0, prefix);

            let last_state_index = last_state_index.expect(
                "should never reach an error state by reading the prefix of an already added key",
            );

            assert!(last_state_index < transducer.states.len());

            let last_state = &transducer.states[last_state_index];
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

    return Ok(transducer);
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

        assert!(state_transitions.len() >= state_outputs.len());
        for (transition, output) in state_transitions.iter().zip(state_outputs) {
            let (on_char, to_state) = transition;

            let label: String;
            if let Some(value) = output.get_value_as_deref() {
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
mod fst_builder_tests {

    use std::io::BufRead;
    use std::rc::Rc;

    use rand::Rng;
    use rand::distributions::Alphanumeric;
    use rand::thread_rng;

    use crate::automaton::Automaton;
    use crate::fst::Transducer;
    use crate::fst_state::FstState;

    use super::StringGenerator;
    use super::build_transducer_from_collection;
    use super::dump_graph;
    use super::parse_line;
    use super::read_lines;

    //#[test]
    //fn small_file() {
    //    let filename = "maps/5.txt";
    //    //let Ok(fst) = build_transducer_from_collection(&filename);
    //
    //    //ensure_all_pairs(&fst, &filename);
    //    //
    //    //dump_graph(&fst, "fst.dot");
    //}
    //
    //
    //#[test]
    //fn random_strings_1() {
    //    let n = 5;
    //
    //    let mut generator = StringGenerator::new(n, 50, 50, '/' as u8);
    //
    //    //let Ok(fst) = build_transducer_from_collection(&mut generator);
    //
    //    //ensure_all_pairs(&fst, &filename);
    //
    //    //dump_graph(&fst, "fst.dot");
    //}

    //#[test]
    //fn random_strings_2() {
    //    let n = 20;
    //
    //    let mut generator = StringGenerator::new(n, 25, 25, '/' as u8);
    //
    //    //let Ok(fst) = build_transducer_from_collection(&mut generator);
    //
    //    //ensure_all_pairs(&fst, &filename);
    //
    //    //dump_graph(&fst, "fst.dot");
    //}

    #[test]
    fn file_100() {
        let Ok(fst) = build_transducer_from_collection("maps/100.txt");

        dump_graph(&fst, "fst.dot");

        ensure_all_pairs(&fst, "maps/100.txt");
    }

    fn ensure_all_pairs(fst: &Transducer, filename: &str) {
        if let Ok(lines) = read_lines(filename) {
            for line in lines.map_while(Result::ok) {
                //convert lines to bytes
                let bytes: &[u8] = &line.as_bytes();

                let (key, value): (&[u8], &[u8]) =
                    parse_line(bytes).expect("No lines should be empty!");

                println!(
                    "ensuring key: {} | has value: {}",
                    std::str::from_utf8(key).unwrap(),
                    std::str::from_utf8(value).unwrap()
                );

                //get value back from fst
                let value_from_fst = fst.get(&key);
                assert!(value_from_fst.is_some());

                assert_eq!(
                    std::str::from_utf8(value).unwrap(),
                    std::str::from_utf8(&value_from_fst.unwrap()).unwrap()
                );
            }
        } else {
            panic!("couldn't open file {filename}");
        }
    }
}
