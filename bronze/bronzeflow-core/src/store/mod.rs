// This is a part of bronze.

//! Storage, storage the runnable object
//!
//!  Storage is a common layer for storing scheduling tasks. Now, just support the ```MemoryStorage```

use crate::task::RunnableHolder;
use std::sync::{Arc, Mutex};

pub trait Storage: Send {
    fn save_runnable(&mut self, runnable: RunnableHolder);

    fn load_runnable(&self) -> Vec<RunnableHolder>;
}

pub type StorageType<SG> = Arc<Mutex<SG>>;

#[derive(Default)]
pub struct MemoryStorage {
    runs: Vec<RunnableHolder>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage { runs: vec![] }
    }
}

impl Storage for MemoryStorage {
    fn save_runnable(&mut self, runnable: RunnableHolder) {
        self.runs.push(runnable)
    }

    fn load_runnable(&self) -> Vec<RunnableHolder> {
        self.runs.iter().map(RunnableHolder::clone).collect()
    }
}
