use std::cell::{Cell, RefCell};


#[derive(Clone)]
pub struct HistFileHelper {
    pub current_index: Cell<usize>,
    pub total_length: Cell<usize>,
    pub hist: RefCell<Vec<String>>
}

impl HistFileHelper {
    pub fn new() -> HistFileHelper {
        HistFileHelper {
            current_index: Cell::new(0),
            total_length: Cell::new(0),
            hist: RefCell::new(Vec::new())
        }
    }
}
