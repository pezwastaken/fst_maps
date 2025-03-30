//must always be a power of 2
pub const REGISTER_CAPACITY: usize = 512;
pub const REGISTER_COUNT_MASK: usize = REGISTER_CAPACITY - 1;

//array of frozen states (their identifiers)
#[derive(Debug)]
pub struct Register {
    pub elements: [usize; REGISTER_CAPACITY],
    pub count: usize,
}

impl Register {
    //TODO get rid of mod
    #[inline]
    pub fn add_state(&mut self, state_id: usize) {
        let i = self.count % REGISTER_CAPACITY;
        self.elements[i] = state_id;
        self.count += 1;
    }
}
