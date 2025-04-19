use crate::fsa::Acceptor;

pub fn build_fsa<'a>(strings: &[String]) -> Option<Acceptor<'a>> {
    //just for expect string
    //let mut previous_str = "";

    let mut fsa = Acceptor::new(30);

    for item in strings.iter() {
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
        fsa.add_suffix_to_fsa(last_state_index, suffix.as_bytes());

        let word_len = item.len();

        fsa.clear_last_str();

        fsa.last_str_len = word_len;
        fsa.last_str[..word_len].copy_from_slice(&item.as_bytes()[..word_len]);

        //previous_str = &item;
    }

    fsa.replace_or_register(0);

    Some(fsa)
}

#[test]
fn test_1() {
    let strings = vec!["ciao".to_string()];
    build_fsa(&strings);
}

#[test]
fn test_2() {
    let strings = vec!["ciao".to_string(), "gatto".into()];
    build_fsa(&strings);
}

#[test]
fn test_3() {
    let strings = vec![
        "ciao".to_string(),
        "gatto".into(),
        "galletta di riso".into(),
    ];
    let fsa = build_fsa(&strings).unwrap();
    //let res = fsa.delta_star_from_initial_state("ciao").unwrap();
    //assert!(fsa.is_final_state(res));
}

#[test]
fn test_4() {
    let strings = vec![
        "ciao".to_string(),
        "gatto".into(),
        "galletta di riso".into(),
        "galletta di zanzara".into(),
    ];
    let fsa = build_fsa(&strings).unwrap();
}

#[test]
fn test_5() {
    let strings = vec![
        "ciao".to_string(),
        "gatto".into(),
        "galletta di riso".into(),
        "zanzara".into(),
    ];
    let fsa = build_fsa(&strings).unwrap();
}
