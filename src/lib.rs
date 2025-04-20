#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod automaton;
mod automaton_decoder;
mod fsa;
mod fsa_builder;
mod fst;
mod fst_builder;
mod fst_state;
mod map;
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

    use crate::{fst::Transducer, map::Map};

    #[test]
    fn ensure_5() {
        let mut map = Map::new("indices/temp_5.fst");

        ensure_all_pairs(&mut map, "maps/5.txt");
    }

    #[test]
    fn ensure_300() {
        let mut map = Map::new("indices/temp_300.fst");

        ensure_all_pairs(&mut map, "maps/300.txt");
    }

    #[test]
    fn ensure_1000_2() {
        let mut map = Map::new("indices/temp_1000_2.fst");

        ensure_all_pairs(&mut map, "maps/1000_2.txt");
    }

    #[test]
    fn ensure_1000() {
        let mut map = Map::new("indices/temp_1000.fst");

        ensure_all_pairs(&mut map, "maps/1000.txt");
    }

    #[test]
    fn ensure_5000() {
        let mut map = Map::new("indices/temp_5000.fst");

        ensure_all_pairs(&mut map, "maps/5000_a.txt");
    }

    #[test]
    fn ensure_tons() {
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
        ensure_all_values(&set, "sets_data/5.txt");
        //
        assert!(!set.contains(b"can"));
        assert!(!set.contains(b"canee"));
        assert!(!set.contains(b"caa"));
        assert!(!set.contains(b"cozz"));
    }

    #[test]
    fn ensure_100() {
        assert!(build_fsa("sets_data/100.txt", "sets/100.fsa").is_some());

        let mut set = Set::new("sets/100.fsa");
        //
        ensure_all_values(&set, "sets_data/100.txt");
        //
        assert!(!set.contains(b"can"));
        assert!(!set.contains(b"canee"));
        assert!(!set.contains(b"caa"));
        assert!(!set.contains(b"cozz"));
    }

    fn ensure_all_values(fsa: &Set, filename: &str) {
        if let Ok(lines) = read_lines(filename) {
            for line in lines.map_while(Result::ok) {
                //convert lines to bytes
                let key = &line.as_bytes();

                println!("ensuring key: {}", std::str::from_utf8(key).unwrap());

                //get value back from fst
                let is_in_set = fsa.contains(&key);
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
