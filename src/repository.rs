use std::cell::RefCell;

use crate::model::Record;

pub trait Repository {
    fn add(&self, record: Record);

    fn get(&self) -> Vec<Record>;
}

pub struct FileRepository {
    
}

pub struct InMemoryRepository {
    records: RefCell<Vec<Record>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            records: RefCell::new(Vec::new()),
        }
    }
}

impl Repository for InMemoryRepository {
    fn add(&self, record: Record) {
        self.records.borrow_mut().push(record);
    }

    fn get(&self) -> Vec<Record> {
        self.records.borrow().to_vec()
    }
}
