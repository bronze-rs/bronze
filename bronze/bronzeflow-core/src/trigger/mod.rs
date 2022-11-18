#[cfg(feature = "async_tokio")]
pub mod tokio_trigger;

use crate::runtime::Runnable;
use crate::store::Storage;
use crate::task::RunnableHolder;
use bronzeflow_time::schedule_time::ScheduleTime;
use bronzeflow_utils::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread, time};

pub type StopSignal = Arc<AtomicBool>;

pub trait Trigger {
    fn trigger<SG, TC>(&mut self, storage: Arc<Mutex<SG>>, trigger_caller: TriggerCallerType<TC>)
    where
        SG: Storage + 'static,
        TC: TriggerCaller + 'static;

    fn stop(&mut self) {
        let is_stop = self.sig();
        if is_stop.load(Ordering::SeqCst) {
            return;
        }
        is_stop.store(true, Ordering::SeqCst);
        self.do_stop();
    }
    fn do_stop(&mut self);

    fn sig(&self) -> &StopSignal;
}

pub trait TriggerCaller: Send {
    fn trigger(&self, runnable: &mut impl Runnable, report_msg: bool);

    #[inline(always)]
    fn trigger_safe<F>(&self, mut runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        self.trigger(&mut runnable, report_msg)
    }

    #[inline(always)]
    fn trigger_holder(&self, runnable: RunnableHolder, report_msg: bool) {
        match runnable {
            RunnableHolder::Task(task) => self.trigger_safe(task.task, report_msg),
            RunnableHolder::Dag(dag) => dag.run_task(|task| {
                let runner = task.as_ref().lock().unwrap().task.clone();
                self.trigger_safe(runner, report_msg);
            }),
        }
    }
}

pub type TriggerCallerType<TC> = Arc<Mutex<TC>>;

pub struct ThreadTrigger {
    join_handler: Option<JoinHandle<()>>,
    is_stop: Arc<AtomicBool>,
}

impl Default for ThreadTrigger {
    fn default() -> Self {
        ThreadTrigger::new()
    }
}
impl ThreadTrigger {
    pub fn new() -> Self {
        ThreadTrigger {
            join_handler: None,
            is_stop: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Trigger for ThreadTrigger {
    fn trigger<SG, TC>(&mut self, storage: Arc<Mutex<SG>>, trigger_caller: TriggerCallerType<TC>)
    where
        SG: Storage + 'static,
        TC: TriggerCaller + 'static,
    {
        let is_stop = Arc::clone(&self.is_stop);
        let handler = thread::spawn(move || {
            // info!("Enter loop");
            loop {
                if is_stop.load(Ordering::SeqCst) {
                    info!("Stop!!!");
                    break;
                }
                // info!("Hello, world");
                let runs = { storage.lock().unwrap().load_runnable() };

                // info!("Get dags size: {}", dags.len());
                let now = ScheduleTime::from_now();
                for mut d in runs {
                    if let Some(ref mut tm) = d.time_holder() {
                        if tm
                            .lock()
                            .unwrap()
                            .schedule
                            .as_mut()
                            .unwrap()
                            .cmp_and_to_next(&now)
                        {
                            trigger_caller.lock().unwrap().trigger_holder(d, true);
                        }
                    }
                }
                thread::sleep(time::Duration::from_millis(500));
            }
        });
        self.join_handler = Some(handler);
    }

    fn do_stop(&mut self) {
        if let Some(handler) = self.join_handler.take() {
            handler.join().unwrap()
        }
    }

    fn sig(&self) -> &StopSignal {
        &self.is_stop
    }
}

impl Drop for ThreadTrigger {
    fn drop(&mut self) {
        self.stop();
    }
}
