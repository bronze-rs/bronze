#[cfg(feature = "async_tokio")]
pub mod tokio_executor;

use crate::prelude::{BronzeRuntime, TriggerCaller};
use crate::runtime::{Runnable, ThreadRuntime};
use bronzeflow_utils::debug;

pub trait Executor: TriggerCaller {
    fn submit(&self, runnable: &mut impl Runnable, report_msg: bool);

    fn submit_safe<F>(&self, mut runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        self.submit(&mut runnable, report_msg)
    }

    #[inline(always)]
    fn support_async(&self) -> bool {
        false
    }

    #[inline(always)]
    fn check_async_support(&self, runnable: &impl Runnable) {
        if runnable.is_async() && !self.support_async() {
            panic!("Current executor not supported async runnable");
        }
    }
}

impl<T: Executor> TriggerCaller for T {
    #[inline(always)]
    fn trigger(&self, runnable: &mut impl Runnable, report_msg: bool) {
        self.check_async_support(runnable);
        debug!(
            "Begin run {}, {:?}",
            runnable.run_type_name(),
            runnable.run_type_id()
        );
        self.submit(runnable, report_msg);
    }

    #[inline(always)]
    fn trigger_safe<F>(&self, runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        debug!(
            "Begin run {}, {:?}",
            runnable.run_type_name(),
            runnable.run_type_id()
        );
        self.check_async_support(&runnable);
        self.submit_safe(runnable, report_msg);
    }
}

#[derive(Default)]
pub struct DefaultExecutor {}

impl DefaultExecutor {
    pub fn new() -> Self {
        DefaultExecutor {}
    }
}

impl Executor for DefaultExecutor {
    #[inline(always)]
    fn submit(&self, runnable: &mut impl Runnable, _: bool) {
        runnable.run();
    }
}

#[derive(Default)]
pub struct ThreadExecutor {
    runtime: ThreadRuntime,
}

impl Executor for ThreadExecutor {
    fn submit(&self, _: &mut impl Runnable, _: bool) {
        // self.runtime.run(runnable, report_msg);
        panic!("Not supported submit in `ThreadExecutor`")
    }

    #[inline(always)]
    fn submit_safe<F>(&self, runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        self.runtime.run_safe(runnable, report_msg);
    }
}
