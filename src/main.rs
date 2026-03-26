use fsa_builder::build_fsa;
use fuzzy_set::FuzzySet;

mod automaton;
mod automaton_decoder;
#[path = "levenshtein/bit_vector.rs"]
mod bit_vector;
mod cap_allocator;
#[path = "levenshtein/dula.rs"]
mod dula;
mod error;
mod fsa;
mod fsa_builder;
mod fst;
mod fst_builder;
mod fst_state;
mod fuzzy_set;
#[path = "levenshtein/levenshtein_nfa.rs"]
mod levenshtein_nfa;
mod map;
#[path = "levenshtein/nula.rs"]
mod nula;
#[path = "inner/raw_fsa_state.rs"]
mod raw_fsa_state;
#[path = "inner/raw_fst_state.rs"]
mod raw_fst_state;
mod raw_writer;
mod register;
mod set;
mod utils;

fn main() {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    //

    println!("size of each FsaState: {}", size_of::<fsa::FsaState>());
    // println!("size of each transition: {}", size_of::<(u8, usize)>());

    let _ = build_fsa("sets_data/words_alpha.txt", "sets/words_alpha.fsa");
    // let _ = build_fsa("sets_data/bag.txt", "sets/bag.fsa");

    let mut fuzzy_set = FuzzySet::new("sets/words_alpha.fsa", "dog");

    fuzzy_set.fuzzy_search();
}
