#![allow(dead_code)]

mod fsa;

#[allow(unused)]
#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::fsa::{Automata, build_fsa};
    use rand::prelude::*;
    use std::fs::File;
    use std::io;
    use std::io::BufRead;

    //#[test]
    //fn common_prefix_on_empty_fsa() {
    //    let automata = Automata::new(100);
    //
    //    let string = String::from("ciao bau");
    //
    //    let i = automata.common_prefix(&string);
    //    assert_eq!(i, 0);
    //}
    //
    //#[test]
    //fn common_prefix_0() {
    //    let mut automata = Automata::new(100);
    //
    //    let string = String::from("ciao bau");
    //    automata.add_suffix_to_fsa(0, string.as_bytes());
    //
    //    let i = automata.common_prefix("topo");
    //    assert_eq!(i, 0);
    //
    //    //let number_of_states = automata.get_n_of_states();
    //
    //    //assert_eq!(number_of_states, string.len() + 1);
    //}
    //
    //#[test]
    //fn common_prefix() {
    //    let mut automata = Automata::new(100);
    //
    //    build_fsa(&mut automata, &["ciaoooooooooo".to_string()]);
    //
    //    let i = automata.common_prefix("ciao");
    //    assert_eq!(i, 4);
    //}
    //
    #[test]
    fn process_string() {
        let mut automata = Automata::new(100);

        let string = String::from("ci");

        automata.add_suffix_to_fsa(0, string.as_bytes());
        //automata.add_suffix_to_fsa(0, "ratto".as_bytes());

        let res_state = automata
            .delta_star_from_initial_state(&string)
            .expect("transitioned to err state");
        assert_eq!(automata.is_final_state(res_state), true);

        automata.replace_or_register(0);
    }
    //
    //#[test]
    //fn double_add_suffix() {
    //    let mut automata = Automata::new(100);
    //
    //    //add first string
    //    let string = String::from("ci");
    //    let first_string_len = string.len();
    //    automata.add_suffix_to_fsa(0, string.as_bytes());
    //
    //    let res_state = automata
    //        .delta_star_from_initial_state(&string)
    //        .expect("moved to error state");
    //
    //    assert_eq!(automata.is_final_state(res_state), true);
    //
    //    //second string
    //    let string = String::from("topolon");
    //
    //    automata.add_suffix_to_fsa(0, string.as_bytes());
    //
    //    let res_state = automata
    //        .delta_star_from_initial_state(&string)
    //        .expect("moved to error state!");
    //    assert_eq!(automata.is_final_state(res_state), true);
    //}

    //#[test]
    //fn delta_star_on_empty() {
    //    let automata = Automata::new(100);
    //
    //    assert!(automata.delta_star(0, &String::from("ciao")).is_none());
    //}
    //
    #[test]
    fn small_word_list() {
        let mut automata = Automata::new(100);

        let string_collection = collection_from_file("words_small.txt").unwrap();

        //let string_collection = ["ciao", "miao", "x"];
        crate::fsa::build_fsa(&mut automata, &string_collection);
        are_strings_accepted(&automata, &string_collection);

        let not_valid = random_strings_from_collection(&string_collection);
        assert!(not_valid.len() != 0);
        are_strings_not_accepted(&automata, &not_valid);
    }

    #[test]
    fn word_list_len_100() {
        let mut automata = Automata::new(100);

        let string_collection = collection_from_file("100_words.txt").unwrap();

        crate::fsa::build_fsa(&mut automata, &string_collection);
        are_strings_accepted(&automata, &string_collection);

        let not_valid = random_strings_from_collection(&string_collection);
        assert!(not_valid.len() != 0);
        are_strings_not_accepted(&automata, &not_valid);
    }

    #[test]
    fn word_list_len_300() {
        let mut automata = Automata::new(200);

        let string_collection = collection_from_file("300_words.txt").unwrap();

        crate::fsa::build_fsa(&mut automata, &string_collection);
        are_strings_accepted(&automata, &string_collection);

        let not_valid = random_strings_from_collection(&string_collection);
        assert!(not_valid.len() != 0);
        are_strings_not_accepted(&automata, &not_valid);
    }

    #[test]
    fn word_list_len_1000() {
        let mut automata = Automata::new(200);

        let string_collection = collection_from_file("1000_words.txt").unwrap();

        crate::fsa::build_fsa(&mut automata, &string_collection);
        are_strings_accepted(&automata, &string_collection);

        let not_valid = random_strings_from_collection(&string_collection);
        assert!(not_valid.len() != 0);
        are_strings_not_accepted(&automata, &not_valid);
    }

    #[test]
    fn word_list_len_10000() {
        let mut automata = Automata::new(200);

        let string_collection = collection_from_file("10000_words.txt").unwrap();

        crate::fsa::build_fsa(&mut automata, &string_collection);
        are_strings_accepted(&automata, &string_collection);

        let not_valid = random_strings_from_collection(&string_collection);
        assert!(not_valid.len() != 0);
        are_strings_not_accepted(&automata, &not_valid);
    }
    //

    //
    //------------------------------------------
    //helper functions
    //

    //read a list of words from file and return the corresponding vector
    fn collection_from_file(filename: &str) -> Result<Vec<String>, &str> {
        let mut string_collection: Vec<String> = Vec::new();

        if let Ok(lines) = read_lines(filename) {
            for (i, line) in lines.filter(|x| x.is_ok()).enumerate() {
                string_collection.push(line.expect("couldn't read line"));
            }
            return Ok(string_collection);
        }
        return Err("couldn't read lines");
    }

    fn random_strings_from_collection(collection: &Vec<String>) -> Vec<String> {
        let mut res: Vec<String> = Vec::with_capacity(collection.len() * 3);

        let mut rng = StdRng::seed_from_u64(101);

        for (_, s) in collection.iter().enumerate() {
            let truncate_to = rng.gen_range(0..s.chars().count());
            //first one is a truncated string
            let x = s[0..truncate_to].to_string();
            if !is_duplicate(collection, &x) {
                res.push(s[0..truncate_to].to_string());
            }
        }
        return res;
    }

    fn is_duplicate(collection: &Vec<String>, s: &str) -> bool {
        return collection.iter().find(|string| **string == *s).is_some();
    }

    fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
        let file = File::open(filename)?;

        Ok(io::BufReader::new(file).lines())
    }

    fn are_strings_accepted(fsa: &Automata, str_collection: &Vec<String>) {
        for (i, s) in str_collection.iter().enumerate() {
            let reached_state = fsa
                .delta_star_from_initial_state(&s)
                .expect("reached error state starting from string {s}");

            assert!(fsa.is_final_state(reached_state));
        }
    }

    fn are_strings_not_accepted(fsa: &Automata, str_collection: &Vec<String>) {
        for (i, s) in str_collection.iter().enumerate() {
            let reached_state = fsa
                .delta_star_from_initial_state(&s)
                .expect("reached error state starting from string {s}");

            assert!(!fsa.is_final_state(reached_state));
        }
    }
}
