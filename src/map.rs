use crate::automaton::AutomatonState;
use crate::automaton_decoder::AutomatonDecoder;
use crate::fst_state::FstState;

pub struct Map {
    decoder: AutomatonDecoder,
}

impl Map {
    pub fn new(fst_file: &str) -> Self {
        match AutomatonDecoder::new(fst_file) {
            Ok(tmp) => Self { decoder: tmp },
            Err(e) => panic!("error while creating fst_decoder {e:?}"),
        }
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.delta_hat(key)
    }

    fn delta_hat(&mut self, input_tape: &[u8]) -> Option<Vec<u8>> {
        let Some(mut current_state) = self.decoder.read_first_state::<FstState>() else {
            panic!("couldn't decode first state given mmapped file");
        };

        let mut output: Vec<u8> = Vec::new();
        //dbg!(&current_state);

        for c in input_tape {
            let (next_state_addr, transition_output) = current_state.delta_with_output(*c)?;

            if current_state.is_final() {
                output.clear();
            }

            //append edge value to the output that is being built
            match transition_output {
                Some(value) => output.extend_from_slice(value),
                None => {}
            }

            current_state = self.decoder.read_state_at_addr(next_state_addr)?;
            //dbg!(&current_state);
        }

        if current_state.is_final() {
            return Some(output);
        }
        None
    }
}

#[allow(unused)]
#[cfg(test)]
mod map_from_file {
    use super::Map;
    //
    //#[test]
    //fn get_1() {
    //    let mut map = Map::new("test.fst");
    //
    //    let res = map.get("ca".as_bytes());
    //
    //    assert!(res.is_some());
    //
    //    assert_eq!(res.as_ref().unwrap(), "ciao".as_bytes());
    //    dbg!(res.unwrap());
    //}
    //
    //#[test]
    //fn get_2() {
    //    let mut map = Map::new("test.fst");
    //
    //    let res = map.get("cane".as_bytes());
    //
    //    assert!(res.is_some());
    //
    //    assert_eq!(res.as_ref().unwrap(), "pizza".as_bytes());
    //    dbg!(res.unwrap());
    //}
    //
    //#[test]
    //fn get_3() {
    //    let mut map = Map::new("test.fst");
    //
    //    let res = map.get("gatto".as_bytes());
    //
    //    assert!(res.is_some());
    //
    //    assert_eq!(res.as_ref().unwrap(), "mammifero".as_bytes());
    //    dbg!(res.unwrap());
    //}
}
