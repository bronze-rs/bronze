use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::Builder as StdThreadBuilder;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::prelude::{
    AsyncFn, BronzeRuntime, TokioRuntime, Trigger, TriggerCaller, TriggerCallerType,
};
use crate::store::Storage;
use crate::task::RunnableHolder;
use crate::trigger::StopSignal;
use bronzeflow_time::schedule_time::ScheduleTime;
use bronzeflow_utils::{info, BronzeError};

// type DAGSender = mpsc::Sender<DAGMessage>;
type DAGReceiver = mpsc::Receiver<DAGMessage>;

enum DAGMessage {
    PayLoad(RunnableHolder),
}

pub struct TokioTrigger {
    is_stop: Arc<AtomicBool>,
    runtime: Arc<TokioRuntime>,
}

impl TokioTrigger {
    pub fn new(runtime: Arc<TokioRuntime>) -> Self {
        TokioTrigger {
            is_stop: Arc::new(AtomicBool::new(false)),
            runtime,
        }
    }

    fn run_loop<T>(&self, mut handle: TriggerEventHandle<T>)
    where
        T: TriggerCaller,
    {
        let rt = self.runtime.runtime.clone();
        StdThreadBuilder::new()
            .name("trigger_handle".into())
            .spawn(move || {
                rt.block_on(async {
                    info!("Start trigger loop handle");
                    handle.run_loop().await;
                })
            })
            .expect("trigger_handle can't start.");
    }
}

impl Trigger for TokioTrigger {
    fn trigger<SG, TC>(&mut self, storage: Arc<Mutex<SG>>, trigger_caller: TriggerCallerType<TC>)
    where
        SG: Storage + 'static,
        TC: TriggerCaller + 'static,
    {
        let (tx, rx) = mpsc::channel(100);
        let is_stop = Arc::clone(&self.is_stop);

        let handle = TriggerEventHandle::new(rx, trigger_caller);
        self.run_loop(handle);

        self.runtime.run_safe(
            AsyncFn(move || {
                let is_stop = is_stop.clone();
                let storage = storage.clone();
                let dag_sender = tx.clone();
                async move {
                    println!("hello, world");
                    loop {
                        if is_stop.load(Ordering::SeqCst) {
                            info!("Stop!!!");
                            break;
                        }
                        // info!("Hello, world");
                        // let dags = { storage.lock().unwrap().load_dags() };
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
                                    // d.run();
                                    // trigger_caller.lock().unwrap().trigger_dag(d);
                                    dag_sender
                                        .send(DAGMessage::PayLoad(d))
                                        .await
                                        .map_err(|_| {
                                            BronzeError::msg(
                                                "Could not send dag to TriggerEventHandle",
                                            )
                                        })
                                        .ok();
                                }
                            }
                        }
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }),
            false,
        );
    }

    fn do_stop(&mut self) {}

    fn sig(&self) -> &StopSignal {
        &self.is_stop
    }
}

struct TriggerEventHandle<TC: TriggerCaller + 'static> {
    dag_receiver: DAGReceiver,
    trigger_caller: TriggerCallerType<TC>,
}

impl<TC: TriggerCaller + 'static> TriggerEventHandle<TC> {
    fn new(dag_receiver: DAGReceiver, trigger_caller: TriggerCallerType<TC>) -> Self {
        TriggerEventHandle {
            dag_receiver,
            trigger_caller,
        }
    }

    async fn run_loop(&mut self) {
        while let Some(event) = self.dag_receiver.recv().await {
            match event {
                DAGMessage::PayLoad(dag) => {
                    // self.trigger_caller.lock().unwrap().trigger_dag(dag, false)
                    self.trigger_caller
                        .lock()
                        .unwrap()
                        .trigger_holder(dag, true)
                    // self.trigger_caller.lock().unwrap().trigger(dag, false)
                },
            }
        }
    }
}
