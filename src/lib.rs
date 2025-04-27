#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod automaton;
mod automaton_decoder;
#[path = "levenshtein/bit_vector.rs"]
mod bit_vector;
mod error;
mod fsa;
mod fsa_builder;
mod fst;
mod fst_builder;
mod fst_state;
#[path = "levenshtein/levenshtein_nfa.rs"]
mod levenshtein_nfa;
mod map;
#[path = "levenshtein/nula.rs"]
mod nula;
#[path = "inner/raw_fsa_state.rs"]
mod raw_fsa_state;
mod raw_writer;
mod register;
mod set;

#[allow(unused)]
#[cfg(test)]
mod get_from_fst {
    use std::{
        fs::File,
        io::{self, BufRead},
    };

    use crate::{
        fst::Transducer,
        fst_builder::{build_transducer_from_collection, dump_graph},
        map::Map,
    };

    #[test]
    fn ensure_5() {
        let Ok(fst) = build_transducer_from_collection("maps/5.txt", "indices/temp_5.fst");

        dump_graph(&fst, "5.dot");

        let mut map = Map::new("indices/temp_5.fst");

        ensure_all_pairs(&mut map, "maps/5.txt");
    }

    #[test]
    fn ensure_300() {
        let Ok(fst) = build_transducer_from_collection("maps/300.txt", "indices/temp_300.fst");
        let mut map = Map::new("indices/temp_300.fst");

        ensure_all_pairs(&mut map, "maps/300.txt");
    }

    #[test]
    fn ensure_1000_2() {
        build_transducer_from_collection("maps/1000_2.txt", "indices/temp_1000_2.fst");
        let mut map = Map::new("indices/temp_1000_2.fst");

        ensure_all_pairs(&mut map, "maps/1000_2.txt");
    }

    #[test]
    fn ensure_1000() {
        build_transducer_from_collection("maps/1000.txt", "indices/temp_1000.fst");
        let mut map = Map::new("indices/temp_1000.fst");

        ensure_all_pairs(&mut map, "maps/1000.txt");
    }

    #[test]
    fn ensure_5000() {
        build_transducer_from_collection("maps/5000_a.txt", "indices/temp_5000.fst");
        let mut map = Map::new("indices/temp_5000.fst");

        ensure_all_pairs(&mut map, "maps/5000_a.txt");
        assert!(map.get(b"giggley").is_none());
        assert!(map.get(b"gil").is_none());
        assert!(map.get(b"infiel").is_none());
        assert!(map.get(b"infieldd").is_none());
    }

    #[test]
    fn ensure_tons() {
        build_transducer_from_collection("maps/tons_of_pairs.txt", "indices/tons.fst");
        let mut map = Map::new("indices/tons.fst");

        ensure_all_pairs(&mut map, "maps/tons_of_pairs.txt");
    }

    fn ensure_all_pairs(fst: &mut Map, filename: &str) {
        if let Ok(lines) = read_lines(filename) {
            for line in lines.map_while(Result::ok) {
                //convert lines to bytes
                let bytes: &[u8] = &line.as_bytes();

                let (key, value): (&[u8], &[u8]) =
                    parse_line(bytes).expect("No lines should be empty!");

                //println!("ensuring key: {}", std::str::from_utf8(key).unwrap());

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
}

#[allow(unused)]
#[cfg(test)]
mod get_from_fsa {
    use std::{
        fs::File,
        io::{self, BufRead},
    };

    use crate::{fsa_builder::build_fsa, set::Set};

    #[test]
    fn ensure_5() {
        assert!(build_fsa("sets_data/5.txt", "sets/5.fsa").is_some());

        let mut set = Set::new("sets/5.fsa");
        //
        ensure_all_values(&mut set, "sets_data/5.txt");
        //
        assert!(!set.contains(b"canee"));
        assert!(!set.contains(b"caa"));
        assert!(!set.contains(b"cozz"));
    }

    #[test]
    fn ensure_100() {
        assert!(build_fsa("sets_data/100.txt", "sets/100.fsa").is_some());

        let mut set = Set::new("sets/100.fsa");
        //
        ensure_all_values(&mut set, "sets_data/100.txt");
        //
        assert!(!set.contains(b"can"));
        assert!(!set.contains(b"canee"));
        assert!(!set.contains(b"caa"));
        assert!(!set.contains(b"cozz"));
    }

    #[test]
    fn ensure_300() {
        assert!(build_fsa("sets_data/300.txt", "sets/300.fsa").is_some());

        let mut set = Set::new("sets/300.fsa");
        ensure_all_values(&mut set, "sets_data/300.txt");
    }

    #[test]
    fn ensure_1000() {
        assert!(build_fsa("sets_data/1000.txt", "sets/1000.fsa").is_some());

        let mut set = Set::new("sets/1000.fsa");
        ensure_all_values(&mut set, "sets_data/1000.txt");
    }

    #[test]
    fn ensure_1000_2() {
        assert!(build_fsa("sets_data/1000_2.txt", "sets/1000_2.fsa").is_some());

        let mut set = Set::new("sets/1000_2.fsa");
        ensure_all_values(&mut set, "sets_data/1000_2.txt");
    }

    #[test]
    fn ensure_3000() {
        assert!(build_fsa("sets_data/3000_rand.txt", "sets/3000_rand.fsa").is_some());

        let mut set = Set::new("sets/3000_rand.fsa");
        ensure_all_values(&mut set, "sets_data/3000_rand.txt");
    }

    #[test]
    fn ensure_tons() {
        assert!(build_fsa("sets_data/tons.txt", "sets/tons.fsa").is_some());

        let mut set = Set::new("sets/tons.fsa");
        ensure_all_values(&mut set, "sets_data/tons.txt");
    }

    #[test]
    fn ensure_words_alpha() {
        assert!(build_fsa("sets_data/words_alpha.txt", "sets/words_alpha.fsa").is_some());

        let mut set = Set::new("sets/words_alpha.fsa");
        ensure_all_values(&mut set, "sets_data/words_alpha.txt");
    }

    fn ensure_all_values(fsa: &mut Set, filename: &str) {
        if let Ok(lines) = read_lines(filename) {
            for line in lines.map_while(Result::ok) {
                //convert lines to bytes
                let key = &line.as_bytes();

                //println!("ensuring key: {}", std::str::from_utf8(key).unwrap());

                //get value back from fst
                let is_in_set = fsa.contains(&key);
                if !is_in_set {
                    println!("key: {} is not in set", std::str::from_utf8(key).unwrap());
                }
                assert!(is_in_set);
            }
        } else {
            panic!("couldn't open file {filename}");
        }
    }

    fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
        let file = File::open(filename)?;

        Ok(io::BufReader::new(file).lines())
    }
}
