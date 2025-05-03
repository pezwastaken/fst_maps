use std::{
    fs::File,
    io::{self, Write},
};

#[derive(Debug)]
pub enum Char {
    Eps,
    Any,
    Exact(u8),
}

#[derive(Debug)]
struct NfaState {
    is_final: bool,
    //transitions could be split by type
    //TODO? should be faster
    transitions: Vec<(Char, usize)>,
}

impl NfaState {
    fn new(f: bool) -> Self {
        Self {
            is_final: f,
            transitions: vec![],
        }
    }

    ///TODO should be used by merge_with
    pub fn set_last_transition_child(&mut self, to_state: usize) -> bool {
        assert!(!self.transitions.is_empty());
        if let Some(transition) = self.transitions.last_mut() {
            transition.1 = to_state;
            return true;
        }
        false
    }
}

impl NfaState {
    fn eps_closure(&self) -> Vec<usize> {
        //todo!("this should go in the automaton struct since it needs access to other states");

        let eps_transitions = self
            .transitions
            .iter()
            .filter(|transition| match transition.0 {
                Char::Eps => true,
                _ => false,
            })
            .map(|t| t.1)
            .collect();

        eps_transitions
    }

    #[inline]
    fn delta(&self, _: Char) -> Option<Vec<usize>> {
        todo!(
            "delta is fine here if eps transitions are not considered. Just any and exact should be considered when looking for transitions on char `on_char`"
        );
        //let transition = self
        //    .transitions
        //    .iter()
        //    .filter(|(on_char, _)| {
        //
        //        match *on_char
        //
        //    })
        //    .find(|(transition_char, _)| *transition_char == c);
        //
        ////return Some(transition?.1);
        //return transition.map(|(_, next_state)| *next_state);
    }

    fn has_children(&self) -> bool {
        !(self.transitions.is_empty())
    }

    fn last_child(&self) -> Option<usize> {
        match self.transitions.last() {
            Some((_, child_state)) => Some(*child_state),
            None => None,
        }
    }

    fn add_transition(&mut self, y: Char, new_state_address: usize) {
        self.transitions.push((y, new_state_address));
        assert!(self.transitions.len() > 0);
    }

    fn clear_state(&mut self) {
        self.is_final = false;
        self.transitions.clear();
    }

    pub fn is_final(&self) -> bool {
        self.is_final
    }

    pub fn get_transitions(&self) -> &[(Char, usize)] {
        &self.transitions
    }
}

pub struct LevNFA {
    states: Vec<NfaState>,
    distance: u8,
}

impl LevNFA {
    pub fn new(k: u8) -> Self {
        Self {
            states: vec![],
            distance: k,
        }
    }

    fn eps_closure(&self, state: usize) -> Vec<usize> {
        let mut reachable_states: Vec<usize> = vec![state];

        let mut i: usize = 0;

        loop {
            if i == reachable_states.len() {
                break;
            }

            dbg!(i);
            //
            let state = reachable_states[i].clone();
            reachable_states.extend_from_slice(&self.states[state].eps_closure());
            dbg!(&reachable_states);

            i += 1;
        }

        reachable_states
    }

    ///Returns the set of initial states.
    ///Includes q0 and all states reachable by computing the eps-closure of q0.
    fn initial_states(&self) -> Vec<usize> {
        self.eps_closure(0)
    }

    fn build_row(&mut self, query_string: &[u8]) {
        assert!(!query_string.is_empty());

        let string_len = query_string.len();

        let first_state_index = self.states.len();

        for i in 0..=string_len {
            let is_final = i == string_len;

            let new_state = NfaState::new(is_final);
            self.states.push(new_state);
        }

        let new_states = &mut self.states[first_state_index..];

        //link the states
        for (i, (state, c)) in new_states.iter_mut().zip(query_string).enumerate() {
            state.add_transition(Char::Exact(*c), i + first_state_index + 1);
        }
    }

    ///Links the previous row of the automaton to the current one.
    ///`current_row_start` is the first index of the current row in the vector
    ///of states.
    ///
    ///The previous row will have indices
    ///     current_row_start - states_per_row .. current_row_start
    fn link_prev_row(&mut self, prev_row_start: usize, states_per_row: usize) {
        assert_eq!(prev_row_start % states_per_row, 0);
        assert_eq!(self.states.len(), prev_row_start + 2 * states_per_row);

        let prev_row = &mut self.states[prev_row_start..prev_row_start + states_per_row];

        for (i, state) in prev_row.iter_mut().enumerate() {
            //transition that represents the insertion of any character
            let up_state = i + states_per_row + prev_row_start;
            state.add_transition(Char::Any, up_state);

            //add diagonal transitions
            //
            //Only possible if `i` is not at the end of a row.
            //Being at the end of the row means:
            //     i + 1 % states_per_row == 0
            //
            if (i + 1 + prev_row_start) % states_per_row != 0 {
                let diag_state = i + states_per_row + 1 + prev_row_start;

                state.add_transition(Char::Any, diag_state);

                state.add_transition(Char::Eps, diag_state);
            }
        }
    }

    pub fn build(&mut self, query_string: &[u8]) {
        let states_per_row = query_string.len() + 1;
        self.build_row(query_string);
        let mut prev_row_start: usize = 0;

        //fill other rows.
        //row 1 will be the one in charge of accepting strings with
        //lev. distance = 1.
        for _ in 1..=self.distance {
            self.build_row(query_string);

            self.link_prev_row(prev_row_start, states_per_row);

            prev_row_start += states_per_row;
        }
    }
}

pub fn dump_graph(nfa: &LevNFA, file_path: &str) -> io::Result<()> {
    println!("states in dump graph");
    //dbg!(&fst.temp_states[0]);
    //dbg!(&fst.states);

    let mut file = File::create(file_path)?;

    file.write(b"digraph DFA {")?;
    file.write(b"    {0} [shape=circle, color=blue, label=\"start\"];")?;

    //mark the accepting states
    for (i, state) in nfa.states.iter().enumerate() {
        if state.is_final {
            let s = format!("    {} [shape=doublecircle, color=green];", i);
            file.write(s.as_bytes())?;
        }
    }

    //add transitions
    for (i, state) in nfa.states.iter().enumerate() {
        let state_transitions = &state.transitions;

        for transition in state_transitions.iter() {
            let (on_char, to_state) = transition;

            let label: String;
            match *on_char {
                Char::Exact(c) => label = format!("{}", c as char),
                Char::Any => label = String::from("*"),
                Char::Eps => label = String::from("ε"),
            }

            let s = format!("    {} -> {} [label=\"{}\"];", i, *to_state, label);
            file.write(s.as_bytes())?;
        }
    }

    file.write(b"}")?;

    Ok(())
}

#[test]
fn build_lev_nfa() {
    let mut nfa = LevNFA::new(3);
    nfa.build(b"mela");
}

#[test]
fn initial_states() {
    let mut nfa = LevNFA::new(2);
    nfa.build(b"quinoa");

    let _ = dump_graph(&nfa, "lev.dot");

    let initial_states = nfa.initial_states();
    dbg!(&initial_states);
    assert!(!initial_states.is_empty());
    assert_eq!(initial_states[0], 0);
    assert_eq!(initial_states[1], 8);
    assert_eq!(initial_states[2], 16);
}
