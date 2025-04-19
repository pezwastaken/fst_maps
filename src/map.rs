use crate::automaton::AutomatonState;
use crate::fst_decoder::FstDecoder;

pub struct Map {
    decoder: FstDecoder,
}

impl Map {
    pub fn new(fst_file: &str) -> Self {
        match FstDecoder::new(fst_file) {
            Ok(tmp) => Self { decoder: tmp },
            Err(e) => panic!("error while creating fst_decoder {e:?}"),
        }
    }

    pub fn get(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.delta_star(key)
    }

    fn delta_star(&mut self, input_tape: &[u8]) -> Option<Vec<u8>> {
        let Some(mut current_state) = self.decoder.read_first_state() else {
            panic!("couldn't decode first state given mmapped file");
        };

        let mut output: Vec<u8> = Vec::new();

        for c in input_tape {
            let (next_state_addr, transition_output) = current_state.delta_with_output(*c)?;

            if current_state.is_final {
                output.clear();
            }

            //append edge value to the output that is being built
            match transition_output.get_value_as_deref() {
                Some(value) => output.extend_from_slice(value),
                None => {}
            }

            current_state = self.decoder.read_state_at_addr(next_state_addr as u64)?
        }

        if current_state.is_final {
            return Some(output);
        }
        None
    }
}

#[allow(unused)]
#[cfg(test)]
mod map_from_file {
    use super::Map;

    #[test]
    fn get_1() {
        let mut map = Map::new("test.fst");

        let res = map.get("ca".as_bytes());

        assert!(res.is_some());

        assert_eq!(res.as_ref().unwrap(), "ciao".as_bytes());
        dbg!(res.unwrap());
    }

    #[test]
    fn get_2() {
        let mut map = Map::new("test.fst");

        let res = map.get("cane".as_bytes());

        assert!(res.is_some());

        assert_eq!(res.as_ref().unwrap(), "pizza".as_bytes());
        dbg!(res.unwrap());
    }

    #[test]
    fn get_3() {
        let mut map = Map::new("test.fst");

        let res = map.get("gatto".as_bytes());

        assert!(res.is_some());

        assert_eq!(res.as_ref().unwrap(), "mammifero".as_bytes());
        dbg!(res.unwrap());
    }
}
