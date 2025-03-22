mod fsa;

#[cfg(test)]
mod tests {
    use crate::fsa::{Automata, build_fsa};

    #[test]
    fn add_word() {
        let mut automata = Automata::new(100);

        let string = String::from("ciao bau");

        automata.add_suffix_to_fsa(0, string.as_bytes());

        let number_of_states = automata.get_n_of_states();

        assert_eq!(number_of_states, string.len() + 1);
    }

    #[test]
    fn common_prefix_on_empty_fsa() {
        let automata = Automata::new(100);

        let string = String::from("ciao bau");

        let i = automata.common_prefix(&string);
        assert_eq!(i, 0);
    }

    #[test]
    fn common_prefix_0() {
        let mut automata = Automata::new(100);

        let string = String::from("ciao bau");
        automata.add_suffix_to_fsa(0, string.as_bytes());

        let i = automata.common_prefix("topo");
        assert_eq!(i, 0);

        //let number_of_states = automata.get_n_of_states();

        //assert_eq!(number_of_states, string.len() + 1);
    }

    #[test]
    fn common_prefix() {
        let mut automata = Automata::new(100);

        build_fsa(&mut automata, &["ciao bau"]);

        let i = automata.common_prefix("ciao");
        assert_eq!(i, 4);
    }

    #[test]
    fn process_string() {
        let mut automata = Automata::new(100);

        let string = String::from("ciao bau");

        automata.add_suffix_to_fsa(0, string.as_bytes());

        let number_of_states = automata.get_n_of_states();

        assert_eq!(number_of_states, string.len() + 1);

        let res_state = automata
            .delta_star_from_initial_state(&string)
            .expect("transitioned to err state");
        assert_eq!(automata.is_final_state(res_state), true);
    }

    #[test]
    fn double_add_suffix() {
        let mut automata = Automata::new(100);

        //add first string
        let string = String::from("ci");
        let first_string_len = string.len();
        automata.add_suffix_to_fsa(0, string.as_bytes());
        let number_of_states = automata.get_n_of_states();

        let res_state = automata
            .delta_star_from_initial_state(&string)
            .expect("moved to error state");

        assert_eq!(number_of_states, string.len() + 1);
        assert_eq!(automata.is_final_state(res_state), true);

        //second string
        let string = String::from("topolon");

        automata.add_suffix_to_fsa(0, string.as_bytes());
        let number_of_states = automata.get_n_of_states();

        assert_eq!(number_of_states, first_string_len + string.len() + 1);

        let res_state = automata
            .delta_star_from_initial_state(&string)
            .expect("moved to error state!");
        assert_eq!(automata.is_final_state(res_state), true);
    }

    #[test]
    fn delta_star_on_empty() {
        let automata = Automata::new(100);

        assert!(automata.delta_star(0, &String::from("ciao")).is_none());
    }

    #[test]
    fn build_automata() {
        let mut automata = Automata::new(100);

        assert!(automata.delta_star(0, &String::from("ciao")).is_none());
        let string_collection = ["ciao", "miao", "x"];
        crate::fsa::build_fsa(&mut automata, &string_collection);

        assert!(automata.delta_star(0, &String::from("ciao")).is_some());
        assert!(automata.delta_star(0, &String::from("miao")).is_some());
        assert!(automata.delta_star(0, &String::from("x")).is_some());

        assert!(automata.delta_star(0, &String::from("xy")).is_none());
        assert!(automata.delta_star(0, &String::from("caio")).is_none());
        assert_eq!(
            automata.is_final_state(
                automata
                    .delta_star(0, &String::from("cia"))
                    .expect("should have reached a non accepting state instead of an error state!")
            ),
            false
        );
        assert_eq!(
            automata.is_final_state(
                automata
                    .delta_star(0, &String::from("mi"))
                    .expect("should have reached a non accepting state instead of an error state!")
            ),
            false
        );
    }

    //#[test]
    //fn build_automata_2() {
    //    let mut automata = Automata::new(100);
    //
    //    assert!(automata.delta_star(0, &String::from("ciao")).is_none());
    //    let string_collection = ["ciao", "ciaone"];
    //    crate::fsa::build_fsa(&mut automata, &string_collection);
    //
    //    let s1 = automata
    //        .delta_star(0, &String::from("ciao"))
    //        .expect("should be valid state");
    //
    //    let s2 = automata
    //        .delta_star(0, &String::from("ciaone"))
    //        .expect("should be valid state");
    //
    //    assert!(s2 > s1);
    //
    //    assert!(automata.is_final_state(s1));
    //    assert!(automata.is_final_state(s2));
    //}
}
