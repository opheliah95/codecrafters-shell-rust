pub struct Counter {
    pub value: u8
}

impl Counter {
    fn increment (&mut self) {
        self.value += 1
    }
}