use crate::executor::Executor;
use crate::runtime::tokio_runtime::{TokioEventReceiver, TokioEventSender, TokioRuntime};
use crate::runtime::{BronzeRuntime, Runnable};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct TokioExecutor {
    runtime: Arc<TokioRuntime>,
}

impl TokioExecutor {
    pub fn new(runtime: Arc<TokioRuntime>) -> Self {
        TokioExecutor { runtime }
    }
}

impl Executor for TokioExecutor {
    type Sender<T> = TokioEventSender<T>;
    type Receiver<T> = TokioEventReceiver<T>;

    #[inline(always)]
    fn submit(&self, _: &mut impl Runnable, _: bool) {
        // runnable.run();
        // self.runtime.run_safe(runnable, report_msg);
        panic!("not supported submit in `TokioExecutor`")
    }

    #[inline(always)]
    fn submit_safe<F>(&self, runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        self.runtime.run_safe(runnable, report_msg);
    }

    #[inline(always)]
    fn support_async(&self) -> bool {
        true
    }

    fn create_channel<T>(buffer: usize) -> (TokioEventSender<T>, TokioEventReceiver<T>) {
        let (tx, rx) = mpsc::channel(buffer);
        let es = TokioEventSender::<T>::new(tx);
        let ee = TokioEventReceiver::<T>::new(rx);
        (es, ee)
    }
}
