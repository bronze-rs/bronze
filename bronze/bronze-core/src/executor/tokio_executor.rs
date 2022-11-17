use crate::executor::Executor;
use crate::runtime::tokio_runtime::TokioRuntime;
use crate::runtime::{BronzeRuntime, Runnable};
use std::sync::Arc;

pub struct TokioExecutor {
    runtime: Arc<TokioRuntime>,
}

impl TokioExecutor {
    pub fn new(runtime: Arc<TokioRuntime>) -> Self {
        TokioExecutor { runtime }
    }
}

impl Executor for TokioExecutor {
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
}
