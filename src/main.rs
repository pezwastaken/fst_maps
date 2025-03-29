mod fsa;
mod fst_state;
use crate::fsa::Automata;
use std::fs::File;
use std::io;
use std::io::BufRead;

fn main() {
    let str1 = "ciao";
    let str2 = "ciaocds";

    let res = str1
        .as_bytes()
        .iter()
        .zip(str2.as_bytes())
        .all(|(x, y)| *x == *y);
    println!("{res}");

    //let mut automata = Automata::new(200);
    //
    //let mut string_collection: Vec<String> = Vec::new();
    //if let Ok(lines) = read_lines("words_small.txt") {
    //    for (_, line) in lines.filter(|x| x.is_ok()).enumerate() {
    //        string_collection.push(line.expect("couldn't read line"));
    //    }
    //}
    //
    //crate::fsa::build_fsa(&mut automata, &string_collection);
    //let _ = crate::fsa::dump_graph(&automata, "fsa.dot");
}

fn read_lines(filename: &str) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;

    Ok(io::BufReader::new(file).lines())
}
