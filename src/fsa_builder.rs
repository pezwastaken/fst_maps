use std::{
    fs::File,
    io::{self, BufRead},
};

use crate::fsa::Acceptor;

pub fn build_fsa<'a>(filename: &str, output_to: &'a str) -> Option<Acceptor<'a>> {
    //just for expect string
    //let mut previous_str = "";
    let mut fsa = Acceptor::new(30, output_to);

    if let Ok(lines) = read_lines(filename) {
        for line in lines.map_while(Result::ok) {
            let bytes = &line.as_bytes();

            let item = bytes;
            //index of last character of common prefix between current item and the last
            //processed word
            let prefix_len = fsa.common_prefix(&item);

            //let curr_item = &item[..];
            //let prefix = &item[0..prefix_len];

            //let last_state_index = fsa.delta_star_from_initial_state(prefix).unwrap();

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

            //previous_str = &item;
        }
    }

    fsa.replace_or_register(0);

    Some(fsa)
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;

    Ok(io::BufReader::new(file).lines())
}

#[test]
fn test_1() {
    build_fsa("sets_data/5.txt", "sets/5.fsa");
}
