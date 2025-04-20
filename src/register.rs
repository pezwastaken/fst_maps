use crate::automaton::{AutomatonState, WithAddress};

//must always be a power of 2
pub const REGISTER_CAPACITY: u64 = 256;
pub const REGISTER_COUNT_MASK: u64 = REGISTER_CAPACITY - 1;
pub const BUCKET_SIZE: usize = 8;

#[derive(Debug)]
pub struct Bucket<T> {
    //elements: [Option<usize>; BUCKET_SIZE],
    elements: [Option<T>; BUCKET_SIZE],
    count: usize,
}

///NOTE
///This doesn't contain any lookup/search method because the actual state object is needed to check for equality.
///It will be implemented by the transducer itself.
impl<T> Bucket<T>
where
    T: AutomatonState + WithAddress + PartialEq,
{
    fn new() -> Self {
        let tmp: [Option<T>; BUCKET_SIZE] = array_init::array_init(|_| None);

        Self {
            elements: tmp,
            count: 0,
        }
    }

    fn insert(&mut self, elem: T) {
        let i = self.count % BUCKET_SIZE;
        self.elements[i] = Some(elem);
        self.count += 1;
    }

    pub fn get_elements(&self) -> &[Option<T>] {
        &self.elements
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }

    ///Check whether this bucket contains an equivalent state.
    ///If so return its address
    fn contains_equivalent(&self, state: &T) -> Option<usize> {
        self.elements
            .iter()
            .filter_map(|x| x.as_ref())
            .find(|elem| {
                return state.eq(*elem);
            })
            .map(|state| state.get_address())
    }
}

///map of frozen states
#[derive(Debug)]
pub struct Register<T> {
    buckets: [Bucket<T>; REGISTER_CAPACITY as usize],
    written_bytes: usize,
}

impl<T> Register<T>
where
    T: AutomatonState + WithAddress + PartialEq,
{
    pub fn new() -> Self {
        //const EMPTY_BUCKET: Bucket = Bucket {
        //    elements: [const { None }; BUCKET_SIZE],
        //    count: 0,
        //};

        let tmp = array_init::array_init(|_| Bucket::new());

        Self {
            buckets: tmp,
            written_bytes: 0,
        }
    }

    //TODO get raddress of mod
    #[inline]
    pub fn add_state(&mut self, state: T) {
        //println!("adding state to register");
        //dbg!(&state);
        let state_hash = (self.hash(&state) % REGISTER_CAPACITY) as usize;
        let bucket = &mut self.buckets[state_hash];
        bucket.insert(state);
    }

    //#[inline]
    //pub fn get_bucket(&self, state: &FstState) -> Option<&Bucket> {
    //    let state_hash = self.hash(state) % REGISTER_CAPACITY;
    //    let res = &self.buckets[state_hash as usize];
    //    return Some(res);
    //}
    //
    #[inline]
    pub fn get_equivalent(&self, state: &T) -> Option<usize> {
        let bucket_address = self.hash(state) % REGISTER_CAPACITY;
        let bucket = &self.buckets[bucket_address as usize];
        bucket.contains_equivalent(state)
    }

    #[inline]
    fn hash(&self, state: &T) -> u64 {
        const FNV_PRIME: u64 = 1099511628211;

        let mut h: u64 = 14695981039346656037;

        h = (h ^ state.is_final() as u64).wrapping_mul(FNV_PRIME);
        for t in state.get_transitions().iter() {
            h = (h ^ (t.0 as u64)).wrapping_mul(FNV_PRIME);
            h = (h ^ (t.1 as u64)).wrapping_mul(FNV_PRIME);
        }

        //working with an fst
        if let Some(outputs) = state.get_outputs() {
            for o in outputs.iter() {
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
        }

        return h;
    }
}
//
//#[test]
//fn should_get_bucket() {
//    let mut register = Register::new();
//
//    //println!("size: {}", size_of::<FstState>());
//    //println!("size: {}", size_of::<Bucket>());
//
//    let mut state_1 = FstState::new();
//
//    state_1.add_transition(100, 1);
//    state_1.set_last_output("ciao".as_bytes());
//    state_1.add_transition(102, 2);
//    state_1.set_last_output("topo".as_bytes());
//    state_1.add_transition(103, 3);
//    state_1.set_last_output("pizza".as_bytes());
//
//    let mut state_2 = FstState::new();
//
//    state_2.add_transition(100, 1);
//    state_2.set_last_output("ciao".as_bytes());
//    state_2.add_transition(102, 2);
//    state_2.set_last_output("topo".as_bytes());
//    state_2.add_transition(103, 3);
//    state_2.set_last_output("pizza".as_bytes());
//
//    assert_eq!(state_1, state_2);
//
//    register.add_state(state_1);
//
//    assert!(register.get_equivalent(&state_2).is_some());
//}
//
//#[test]
//fn should_not_find_eq() {
//    let mut register = Register::new();
//
//    let mut state_1 = FstState::new();
//
//    state_1.add_transition(100, 1);
//    state_1.set_last_output("ciao".as_bytes());
//    state_1.add_transition(102, 2);
//    state_1.set_last_output("topo".as_bytes());
//    state_1.add_transition(103, 3);
//    state_1.set_last_output("pizza".as_bytes());
//
//    let mut state_2 = FstState::new();
//
//    state_2.add_transition(100, 1);
//    state_2.set_last_output("ciao".as_bytes());
//    state_2.add_transition(102, 2);
//    state_2.set_last_output("topo".as_bytes());
//    state_2.add_transition(103, 4);
//    state_2.set_last_output("pizza".as_bytes());
//
//    assert_ne!(state_1, state_2);
//
//    register.add_state(state_1);
//    register.add_state(state_2);
//
//    let mut not_eq = FstState::new();
//
//    not_eq.add_transition(100, 1);
//    not_eq.set_last_output("ciao".as_bytes());
//    not_eq.add_transition(102, 2);
//    not_eq.set_last_output("topo".as_bytes());
//    not_eq.add_transition(103, 4);
//    not_eq.set_last_output("pizzaz".as_bytes());
//
//    assert!(register.get_equivalent(&not_eq).is_none());
//}
//
//#[test]
//fn should_not_find_eq_2() {
//    let mut register = Register::new();
//
//    let mut state_1 = FstState::new();
//    state_1.is_final = true;
//
//    let mut state_2 = FstState::new();
//
//    state_2.add_transition(99, 1);
//    state_2.set_last_output(b"ciao");
//    state_2.add_transition(112, 5);
//    state_2.set_last_output(b"ratto");
//
//    assert_ne!(state_1, state_2);
//    register.add_state(state_1);
//
//    assert!(register.get_equivalent(&state_2).is_none());
//}
