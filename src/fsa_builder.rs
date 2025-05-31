use std::{
    fs::File,
    io::{self, BufRead},
};

use crate::fsa::Acceptor;

pub fn build_fsa<'a>(filename: &str, output_to: &'a str) -> bool {
    let mut fsa = Acceptor::new(30, output_to);

    if let Ok(lines) = read_lines(filename) {
        for line in lines.map_while(Result::ok) {
            fsa.prev_state_addr = None;
            let bytes = &line.as_bytes();

            assert_ne!(bytes.len(), 0);

            let item = bytes;
            //index of last character of common prefix between current item and the last
            //processed word
            let prefix_len = fsa.common_prefix(&item);

            let last_state_index = prefix_len;
            let suffix = &item[prefix_len..];

            assert!(last_state_index < fsa.states.len());

            if fsa.state_has_children(last_state_index) {
                fsa.replace_or_register(last_state_index);
            }

            //TODO this should be a method of State
            //maybe?
            fsa.add_suffix_to_fsa(last_state_index, suffix);

            let word_len = item.len();

            fsa.clear_last_str();

            fsa.last_str_len = word_len;
            fsa.last_str[..word_len].copy_from_slice(&item[..word_len]);
        }
    }

    fsa.replace_or_register(0);

    match fsa.persist_initial_state() {
        Err(e) => panic!("error while persisting initial state: {:?}", e),
        _ => {}
    }

    //println!("fsa output to file!");
    true
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;

    Ok(io::BufReader::new(file).lines())
}
