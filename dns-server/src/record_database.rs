use crate::message::{Name, Record};

pub struct RecordDatabase {
    records: Vec<Box<dyn Record>>,
}

impl RecordDatabase {
    pub fn new(records: Vec<Box<dyn Record>>) -> RecordDatabase {
        RecordDatabase { records }
    }
    pub fn get_record(&self, name: &Name) -> Option<&Box<dyn Record>> {
        self.records.iter().find(|record| record.get_name() == name)
    }
}
