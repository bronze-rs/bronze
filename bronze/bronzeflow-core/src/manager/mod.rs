use crate::prelude::{Executor, StorageType, Trigger};
use crate::service::Service;
use crate::store::Storage;

use crate::task::RunnableHolder;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct ScheduleManager<SG: Storage + 'static, TG: Trigger + 'static, E: Executor + 'static> {
    storage: StorageType<SG>,
    trigger: Arc<Mutex<TG>>,
    executor: Arc<Mutex<E>>,
    dag_id: AtomicU64,
    task_id: AtomicU64,
}

impl<SG: Storage, TG: Trigger, E: Executor> ScheduleManager<SG, TG, E> {
    pub fn new(storage: SG, trigger: TG, executor: E) -> Self {
        // let storage: StorageType = MemoryStorage::new().into();
        ScheduleManager {
            storage: Arc::new(Mutex::new(storage)),
            executor: Arc::new(Mutex::new(executor)),
            trigger: Arc::new(Mutex::new(trigger)),
            dag_id: AtomicU64::new(0),
            task_id: AtomicU64::new(0),
        }
    }

    pub fn add_runnable(&mut self, mut runnable: RunnableHolder) {
        match runnable {
            RunnableHolder::Dag(ref mut dag) => {
                // set dag id
                if let Some(ref mut meta) = dag.meta {
                    meta.lock()
                        .unwrap()
                        .set_id(self.dag_id.fetch_add(1, Ordering::Relaxed));
                }
                // set task id for all tasks in this dag
                dag.for_all_task(|task| {
                    if let Some(m) = task.as_ref().lock().unwrap().meta.as_mut() {
                        m.set_id(self.task_id.fetch_add(1, Ordering::Relaxed));
                    }
                })
            },
            RunnableHolder::Task(ref mut t) => {
                // set task id
                if let Some(ref mut meta) = t.meta {
                    meta.lock()
                        .unwrap()
                        .set_id(self.task_id.fetch_add(1, Ordering::Relaxed));
                }
            },
        }
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
