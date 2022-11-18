use crate::prelude::{Executor, StorageType, Trigger};
use crate::service::Service;
use crate::store::Storage;

use crate::task::RunnableHolder;
use std::sync::{Arc, Mutex};

pub struct ScheduleManager<SG: Storage + 'static, TG: Trigger + 'static, E: Executor + 'static> {
    storage: StorageType<SG>,
    trigger: Arc<Mutex<TG>>,
    executor: Arc<Mutex<E>>,
}

impl<SG: Storage, TG: Trigger, E: Executor> ScheduleManager<SG, TG, E> {
    pub fn new(storage: SG, trigger: TG, executor: E) -> Self {
        // let storage: StorageType = MemoryStorage::new().into();
        ScheduleManager {
            storage: Arc::new(Mutex::new(storage)),
            executor: Arc::new(Mutex::new(executor)),
            trigger: Arc::new(Mutex::new(trigger)),
        }
    }

    pub fn add_runnable(&mut self, runnable: RunnableHolder) {
        self.storage.lock().unwrap().save_runnable(runnable)
    }

    fn start_loader(&mut self) {
        let storage = Arc::clone(&self.storage);
        let trigger = Arc::clone(&self.executor);
        self.trigger.lock().unwrap().trigger(storage, trigger);
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Service for ScheduleManager<SG, TG, E> {
    fn start(&mut self) {
        self.start_loader();
    }

    fn stop(&mut self) {
        self.trigger.lock().unwrap().stop();
    }
}
