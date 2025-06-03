use fsa_builder::build_fsa;
use fuzzy_set::FuzzySet;
use set::Set;

mod automaton;
mod automaton_decoder;
#[path = "levenshtein/bit_vector.rs"]
mod bit_vector;
mod cap_allocator;
#[path = "inner/compressed_states.rs"]
mod compressed_states;
#[path = "levenshtein/dula.rs"]
mod dula;
mod error;
mod fsa;
mod fsa_builder;
mod mappings;
// mod fst;
// mod fst_builder;
// mod fst_state;
mod fuzzy_set;
#[path = "levenshtein/levenshtein_nfa.rs"]
mod levenshtein_nfa;
// mod map;
#[path = "levenshtein/nula.rs"]
mod nula;
#[path = "inner/raw_fsa_state.rs"]
mod raw_fsa_state;
// #[path = "inner/raw_fst_state.rs"]
// mod raw_fst_state;
mod raw_writer;
mod register;
mod set;
mod utils;

fn main() {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();

    // println!("size of each transition: {}", size_of::<(u8, usize)>());

    // let _ = build_fsa("sets_data/words_alpha.txt", "sets/words_alpha.fsa");
    // let _ = build_fsa("sets_data/words_1.txt", "sets/words_1.fsa");
    // let mut fuzzy_set = FuzzySet::new("sets/bag.fsa", "dog");

    let _ = build_fsa("sets_data/words_alpha.txt", "sets/words_alpha.fsa").unwrap();

    // let mut set = Set::new("sets/words_alpha.fsa");
    // ensure_all_values(&mut set, "sets_data/words_alpha.txt");

    // assert!(!set.contains(b"topo0"));

    // let mut fuzzy_set = FuzzySet::new("sets/words_alpha.fsa", "dog");
    //
    // fuzzy_set.fuzzy_search();
}
