pub mod builder;
pub mod dag;

#[cfg(feature = "async")]
use crate::prelude::{Runnable, RuntimeJoinHandle};
use std::sync::{Arc, Mutex};

use crate::runtime::{BuildFromRunnable, SafeMetadata, SafeWrappedRunner, WrappedRunner};

pub type TaskInfo = SafeWrappedRunner;

impl BuildFromRunnable for TaskInfo {
    type Type = TaskInfo;
    fn build_from(
        runnable: impl Runnable<Handle = RuntimeJoinHandle<()>> + Send + 'static,
    ) -> TaskInfo {
        SafeWrappedRunner(Arc::new(Mutex::new(WrappedRunner(Box::new(runnable)))))
    }
}

#[derive(Clone)]
pub struct WrappedTask {
    pub(crate) task: TaskInfo,
    pub(crate) meta: Option<SafeMetadata>,
}

#[derive(Clone)]
pub enum RunnableHolder {
    Task(WrappedTask),
    Dag(dag::DAG),
}

impl RunnableHolder {
    pub fn time_holder(&mut self) -> Option<SafeMetadata> {
        match self {
            RunnableHolder::Task(t) => t.meta.as_ref().map(Arc::clone),
            RunnableHolder::Dag(d) => d.meta.as_ref().map(Arc::clone),
        }
    }
}

impl WrappedTask {
    pub fn new(task: TaskInfo, meta: Option<SafeMetadata>) -> Self {
        WrappedTask { task, meta }
    }
}

pub trait TryIntoTask {
    type TaskDetail;

    fn try_into_task(self) -> TaskInfo;
}
//
// impl<F: Fn() + Send + 'static + Clone> TryIntoTask for SyncFn<F> {
//     type TaskDetail = WrappedRunner;
//
//     fn try_into_task(self) -> TaskInfo {
//         SafeWrappedRunner(Arc::new(Mutex::new(WrappedRunner(Box::new(SyncFn(
//             self.0,
//         ))))))
//     }
// }
//
// #[cfg(feature = "async")]
// impl<F: Fn() -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static> TryIntoTask
//     for AsyncFn<F, U>
// {
//     type TaskDetail = WrappedRunner;
//
//     fn try_into_task(self) -> TaskInfo {
//         SafeWrappedRunner(Arc::new(Mutex::new(WrappedRunner(Box::new(AsyncFn(
//             self.0,
//         ))))))
//     }
// }
//
// impl<F: Fn() + Send + 'static + Clone> TryIntoTask for F {
//     type TaskDetail = WrappedRunner;
//
//     fn try_into_task(self) -> TaskInfo {
//         TryIntoTask::try_into_task(SyncFn(self))
//     }
// }

// impl<F: Fn() + Send + 'static + Clone> ! NotFnRunnable for F {}

impl<F> TryIntoTask for F
where
    F: Runnable<Handle = RuntimeJoinHandle<()>> + Send,
{
    type TaskDetail = WrappedRunner;

    fn try_into_task(self) -> TaskInfo {
        SafeWrappedRunner(Arc::new(Mutex::new(WrappedRunner(Box::new(self)))))
    }
}
