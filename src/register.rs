use crate::{automaton::AutomatonState, fst_state::FstState};

//must always be a power of 2
pub const REGISTER_CAPACITY: u64 = 1024;
pub const REGISTER_COUNT_MASK: u64 = REGISTER_CAPACITY - 1;
pub const BUCKET_SIZE: usize = 16;

#[derive(Debug)]
pub struct Bucket {
    elements: [Option<usize>; BUCKET_SIZE],
    count: usize,
}

///NOTE
///This doesn't contain any lookup/search method because the actual state object is needed to check for equality.
///It will be implemented by the transducer itself.
impl Bucket {
    fn new() -> Self {
        Self {
            elements: [None; BUCKET_SIZE],
            count: 0,
        }
    }

    fn insert(&mut self, elem: usize) {
        let i = self.count % BUCKET_SIZE;
        self.elements[i] = Some(elem);
        self.count += 1;
    }

    pub fn get_elements(&self) -> &[Option<usize>] {
        &self.elements
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }
}

///map of frozen states (their identifiers)
#[derive(Debug)]
pub struct Register {
    //elements: [Option<usize>; REGISTER_CAPACITY as usize],
    buckets: [Bucket; REGISTER_CAPACITY as usize],
}

impl Register {
    pub fn new() -> Self {
        const EMPTY_BUCKET: Bucket = Bucket {
            elements: [None; BUCKET_SIZE],
            count: 0,
        };

        Self {
            buckets: [EMPTY_BUCKET; REGISTER_CAPACITY as usize],
        }
    }

    //TODO get rid of mod
    #[inline]
    pub fn add_state(&mut self, state: &FstState, state_index: usize) {
        let state_hash = (self.hash(&state) % REGISTER_CAPACITY) as usize;
        let bucket = &mut self.buckets[state_hash];
        bucket.insert(state_index);
    }

    #[inline]
    pub fn get_bucket(&self, state: &FstState) -> Option<&Bucket> {
        let state_hash = self.hash(state) % REGISTER_CAPACITY;
        let res = &self.buckets[state_hash as usize];
        return Some(res);
    }

    #[inline]
    fn hash(&self, state: &FstState) -> u64 {
        const FNV_PRIME: u64 = 1099511628211;

        let mut h: u64 = 14695981039346656037;

        h = (h ^ state.is_final as u64).wrapping_mul(FNV_PRIME);
        for t in state.transitions.iter() {
            h = (h ^ (t.0 as u64)).wrapping_mul(FNV_PRIME);
            h = (h ^ (t.1 as u64)).wrapping_mul(FNV_PRIME);
        }

        for o in state.outputs.iter() {
            let value = o.get_value_as_deref();
            match value {
                Some(seq) => {
                    for byte in seq {
                        h = (h ^ (*byte as u64)).wrapping_mul(FNV_PRIME);
                    }
                }
                None => {
                    h = (h ^ 0).wrapping_mul(FNV_PRIME);
                }
            }
        }
        return h;
    }
}

#[test]
fn should_get_bucket() {
    let mut register = Register::new();

    let mut state_1 = FstState::new();

    state_1.add_transition(100, 1);
    state_1.set_last_output("ciao".as_bytes());
    state_1.add_transition(102, 2);
    state_1.set_last_output("topo".as_bytes());
    state_1.add_transition(103, 3);
    state_1.set_last_output("pizza".as_bytes());

    let mut state_2 = FstState::new();

    state_2.add_transition(100, 1);
    state_2.set_last_output("ciao".as_bytes());
    state_2.add_transition(102, 2);
    state_2.set_last_output("topo".as_bytes());
    state_2.add_transition(103, 3);
    state_2.set_last_output("pizza".as_bytes());

    assert_eq!(state_1, state_2);

    register.add_state(&state_1, 1);
    register.add_state(&state_2, 2);
    assert!(register.get_bucket(&state_1).is_some());
    assert!(register.get_bucket(&state_2).is_some());
}

#[test]
fn should_not_find_eq() {
    let mut register = Register::new();

    let mut state_1 = FstState::new();

    state_1.add_transition(100, 1);
    state_1.set_last_output("ciao".as_bytes());
    state_1.add_transition(102, 2);
    state_1.set_last_output("topo".as_bytes());
    state_1.add_transition(103, 3);
    state_1.set_last_output("pizza".as_bytes());

    let mut state_2 = FstState::new();

    state_2.add_transition(100, 1);
    state_2.set_last_output("ciao".as_bytes());
    state_2.add_transition(102, 2);
    state_2.set_last_output("topo".as_bytes());
    state_2.add_transition(103, 4);
    state_2.set_last_output("pizza".as_bytes());

    assert_ne!(state_1, state_2);

    register.add_state(&state_1, 1);
    assert!(register.get_bucket(&state_1).unwrap().count == 1);

    assert!(register.get_bucket(&state_2).unwrap().is_empty());
}

#[test]
fn should_not_find_eq_2() {
    let mut register = Register::new();

    let mut state_1 = FstState::new();
    state_1.add_transition(103, 88);

    let mut state_2 = FstState::new();

    state_2.add_transition(111, 89);

    assert_ne!(state_1, state_2);
    register.add_state(&state_1, 1);

    assert!(register.get_bucket(&state_2).unwrap().is_empty());
}
