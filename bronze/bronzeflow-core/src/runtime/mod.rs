// This is a part of bronze.

//! Bronze's runtime, which is the abstraction of the basic scheduling task, and wraps the synchronous and asynchronous runtime environment
//!
//! You could implement your custom runtime
//!
//! **There are two abstraction:**
//!
//! 1. ```Runnable```: basic trait to be implemented that are scheduled to run, it maybe a function, a closure or a struct, etc.
//! 2. ```BronzeRuntime```: the trait for runtime environment。
//!
// TODO Add more examples to use the runnable and runtime

#[cfg(feature = "async_tokio")]
pub mod tokio_runtime;

use std::any::{type_name, TypeId};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;

#[allow(unused_imports)]
#[cfg(feature = "async")]
use futures::executor as executor_executor;

use bronzeflow_time::schedule_time::ScheduleTimeHolder;
#[cfg(feature = "async_tokio")]
use tokio;
#[cfg(feature = "async_tokio")]
use tokio::spawn as tokio_spawn;

#[derive(Debug)]
pub enum RuntimeJoinHandle<T> {
    SyncJobHandle,

    #[cfg(feature = "async_tokio")]
    AsyncTokioJoinHandle(tokio::task::JoinHandle<T>),

    #[cfg(feature = "async")]
    FutureBlockJoinHandle(T),

    _Unreachable(std::convert::Infallible, std::marker::PhantomData<T>),
}

pub(crate) trait NotFnRunnable {}

#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct AsyncFn<F: Fn() -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static>(
    pub F,
);

#[derive(Debug, Clone)]
pub struct SyncFn<F: Fn() + Send + 'static + Clone>(pub F);

#[cfg(feature = "async")]
impl<F: Fn() -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static> From<F>
    for AsyncFn<F, U>
{
    fn from(value: F) -> Self {
        AsyncFn(value)
    }
}

// #[cfg(feature = "async")]
// impl<F: Fn() -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static> !NotFnRunnable for F{}

impl<F: Fn() + Send + 'static + Clone> From<F> for SyncFn<F> {
    fn from(value: F) -> Self {
        SyncFn(value)
    }
}

// impl<F: Fn() + Send + 'static + Clone> !NotFnRunnable for F{}

// pub trait RunnableType {
//     fn get_type_name(&self) -> &str {
//         type_name::<Self>()
//     }
// }

pub trait Runnable: 'static {
    type Handle = RuntimeJoinHandle<()>;

    // TODO remove name
    fn name(&self) -> String {
        "test name in runnable".to_string()
    }

    // TODO remove name
    fn set_name(&mut self, _: &str) {}

    #[inline(always)]
    fn run(&self) -> Self::Handle {
        self.run_async()
    }

    fn run_async(&self) -> Self::Handle;

    #[inline(always)]
    fn is_async(&self) -> bool {
        false
    }

    fn metadata(&self) -> Option<RunnableMetadata> {
        None
    }

    #[inline(always)]
    fn run_type_name(&self) -> String {
        type_name::<Self>().to_string()
    }

    #[inline(always)]
    fn run_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    //
    // #[inline(always)]
    // fn to_safe_wrapper(self) -> SafeWrappedRunner {
    //     SafeWrappedRunner(Arc::new(Mutex::new(
    //         WrappedRunner(Box::new(self))
    //     )))
    // }
}

// TODO delete this
pub trait BuildFromRunnable {
    type Type;
    fn build_from(
        runnable: impl Runnable<Handle = RuntimeJoinHandle<()>> + Send + 'static,
    ) -> Self::Type;
}

pub fn run_async<F, U>(runnable: &F) -> RuntimeJoinHandle<()>
where
    F: Fn() -> U + Send + Clone + 'static,
    U: std::future::Future + Send + 'static,
{
    #[allow(unused_variables)]
    let f = runnable();
    cfg_if::cfg_if! {
        if #[cfg(feature = "async_tokio")] {
            let handle = tokio_spawn({
                async {
                    f.await;
                }
            });
            RuntimeJoinHandle::AsyncTokioJoinHandle(handle)
        } else if #[cfg(feature = "async")] {
            // if not tokio, use `block_on`
            // just for dev
            let _output = executor_executor::block_on({
                async {
                    f.await
                }
            });
            // TODO return data of real type
            RuntimeJoinHandle::FutureBlockJoinHandle(())
        } else {
            panic!("Not support run async");
        }
    }
}

#[cfg(feature = "async")]
impl<F: Fn() -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static> Runnable
    for AsyncFn<F, U>
{
    type Handle = RuntimeJoinHandle<()>;

    #[inline(always)]
    fn run_async(&self) -> Self::Handle {
        run_async(&self.0)
    }

    #[inline(always)]
    fn is_async(&self) -> bool {
        true
    }
}
// Can`t do this, see: https://stackoverflow.com/questions/73782573/why-do-blanket-implementations-for-two-different-traits-conflict
// #[cfg(feature = "async")]
// impl<F: Fn(i32) -> U + Send + Clone + 'static, U: std::future::Future + Send + 'static> Runnable for F {
//     fn run_async(&self) -> Self::Handle {
//         run_async(&self)
//     }
// }

impl<F: Fn() + Send + 'static + Clone> Runnable for SyncFn<F> {
    type Handle = RuntimeJoinHandle<()>;

    #[inline(always)]
    fn run_async(&self) -> Self::Handle {
        self.0();
        RuntimeJoinHandle::SyncJobHandle
    }
}

impl<F: Fn() + Send + 'static + Clone> Runnable for F {
    fn run_async(&self) -> Self::Handle {
        self();
        RuntimeJoinHandle::SyncJobHandle
    }
}

pub type RunnerType = Box<dyn Runnable<Handle = RuntimeJoinHandle<()>> + 'static + Send>;

pub struct WrappedRunner(pub RunnerType);

#[derive(Clone)]
pub struct SafeWrappedRunner(pub(crate) Arc<Mutex<WrappedRunner>>);

impl Runnable for WrappedRunner {
    type Handle = RuntimeJoinHandle<()>;

    #[inline(always)]
    fn run_async(&self) -> Self::Handle {
        self.0.run_async()
    }

    #[inline(always)]
    fn run_type_name(&self) -> String {
        self.0.run_type_name()
    }

    #[inline(always)]
    fn run_type_id(&self) -> TypeId {
        self.0.run_type_id()
    }
}

impl Runnable for SafeWrappedRunner {
    type Handle = RuntimeJoinHandle<()>;

    #[inline(always)]
    fn run_async(&self) -> Self::Handle {
        self.0.lock().unwrap().0.run_async()
    }

    #[inline(always)]
    fn run_type_name(&self) -> String {
        // self.0.as_ref().lock().unwrap().type_name()
        self.0.lock().unwrap().0.run_type_name()
    }
    #[inline(always)]
    fn run_type_id(&self) -> TypeId {
        self.0.lock().unwrap().0.run_type_id()
    }
}

pub trait BronzeRuntime {
    fn run(&self, runnable: impl Runnable, report_msg: bool);

    fn run_safe<F>(&self, runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        self.run(runnable, report_msg)
    }
}

#[derive(Default)]
pub struct ThreadRuntime {}

impl BronzeRuntime for ThreadRuntime {
    fn run(&self, _: impl Runnable, _: bool) {
        panic!("Not supported in `ThreadRuntime`, please use `run_safe`")
    }

    #[inline(always)]
    fn run_safe<F>(&self, runnable: F, _: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        let handle = thread::spawn(move || {
            runnable.run();
        });
        handle.join().unwrap();
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct RunnableMetadata {
    #[allow(dead_code)]
    pub(crate) id: Option<u64>,
    #[allow(dead_code)]
    pub(crate) name: Option<String>,
    #[allow(dead_code)]
    pub(crate) maximum_run_times: Option<u64>,
    #[allow(dead_code)]
    pub(crate) maximum_parallelism: Option<u32>,
    #[allow(dead_code)]
    pub(crate) schedule: Option<ScheduleTimeHolder>,
}

impl Default for RunnableMetadata {
    fn default() -> Self {
        RunnableMetadataBuilder::default()
            .id(None)
            .name(None)
            .maximum_run_times(None)
            .maximum_parallelism(None)
            .schedule(None)
            .build()
            .unwrap()
    }
}

impl RunnableMetadata {
    pub fn set_id(&mut self, id: u64) -> &mut Self {
        self.id = Some(id);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn set_maximum_run_times(&mut self, maximum_run_times: u64) -> &mut Self {
        self.maximum_run_times = Some(maximum_run_times);
        self
    }

    pub fn set_maximum_parallelism(&mut self, maximum_parallelism: u32) -> &mut Self {
        self.maximum_parallelism = Some(maximum_parallelism);
        self
    }

    pub fn set_schedule(&mut self, schedule: ScheduleTimeHolder) -> &mut Self {
        self.schedule = Some(schedule);
        self
    }
}

impl From<&str> for RunnableMetadata {
    fn from(value: &str) -> Self {
        let mut m = RunnableMetadata::default();
        m.set_name(value.to_string());
        m
    }
}

pub type SafeMetadata = Arc<Mutex<RunnableMetadata>>;

#[cfg(test)]
mod tests {
    use super::*;
    use bronzeflow_utils::{debug, info};

    #[cfg(feature = "async")]
    #[test]
    fn create_async_fn() {
        let _ = AsyncFn(|| async { info!("I am an async function") });
        let _ = AsyncFn(|| async { 0 });
    }

    #[cfg(feature = "async")]
    #[test]
    #[allow(unused_assignments)]
    fn create_async_fn_mut() {
        let t = String::from("test");
        let _ = AsyncFn(move || {
            let mut nt = t.clone();
            println!("{}", nt);
            async move {
                nt = String::from("new string");
                println!("Nt: {:?}", nt);
                nt.push_str("hx");
                let _ = nt;
            }
        });
    }

    #[cfg(feature = "async")]
    #[test]
    #[allow(unused_assignments)]
    fn async_fn_from_closure() {
        let _ = AsyncFn::from(|| async {
            info!("new function");
        });
        let t = String::new();
        let _ = AsyncFn::from(move || {
            let mut t = t.clone();
            async move {
                t = String::new();
                info!("{}", t);
            }
        });
    }

    #[cfg(feature = "async_tokio")]
    #[tokio::test]
    async fn async_fn_with_tokio() {
        let f = AsyncFn(|| async { info!("I am an async function") });
        check_run_result(f.run(), true, true);
        check_run_result(f.run_async(), true, true);
    }

    #[cfg(feature = "async")]
    #[cfg(not(feature = "async_tokio"))]
    #[test]
    fn async_fn_without_tokio() {
        let f = AsyncFn(|| async { info!("I am an async function") });
        check_run_result(f.run(), true, false);
        check_run_result(f.run_async(), true, false);
    }

    #[test]
    fn create_sync_fn() {
        let _ = SyncFn(|| info!("I am a sync function"));
        let _ = SyncFn(|| {
            info!("This function could not return data");
        });
        let _ = SyncFn::from(|| info!("I am a sync function"));
    }

    #[test]
    fn sync_fn_run() {
        let f = SyncFn(|| info!("I am a sync function"));
        check_run_result(f.run(), false, false);
        check_run_result(f.run_async(), false, false);
    }

    fn check_run_result<T>(handle: RuntimeJoinHandle<T>, is_async: bool, is_tokio: bool) {
        match handle {
            RuntimeJoinHandle::SyncJobHandle if !is_async => (),
            #[cfg(feature = "async_tokio")]
            RuntimeJoinHandle::AsyncTokioJoinHandle(_) if is_async && is_tokio => (),

            #[cfg(feature = "async")]
            RuntimeJoinHandle::FutureBlockJoinHandle(_) if is_async && !is_tokio => (),
            _ => panic!("Run sync function failed"),
        }
    }

    fn test_basic_runnable<T, F>(
        runnable: impl Runnable<Handle = T>,
        is_async: bool,
        is_tokio: bool,
        validator: F,
    ) where
        F: Fn(T, bool, bool),
    {
        assert_eq!(is_async, runnable.is_async());
        validator(runnable.run(), is_async, is_tokio);
        validator(runnable.run_async(), is_async, is_tokio);
    }

    #[test]
    fn run_runnable() {
        let sync_fn = SyncFn(|| info!("I am a sync function"));
        test_basic_runnable(sync_fn, false, false, check_run_result);
    }

    #[cfg(feature = "async")]
    #[cfg(not(feature = "async_tokio"))]
    #[test]
    fn run_runnable_without_tokio() {
        let f = AsyncFn(|| async { info!("I am an async function to run without tokio") });
        test_basic_runnable(f, true, false, check_run_result);
    }

    #[cfg(feature = "async_tokio")]
    #[tokio::test]
    async fn run_runnable_with_tokio() {
        let f = AsyncFn(|| async { info!("I am an async function run with tokio") });
        test_basic_runnable(f, true, true, check_run_result);
    }

    #[test]
    #[should_panic]
    fn bronze_runtime_run_panic() {
        let sync_fn = SyncFn(|| info!("I am a sync function"));
        let rt = ThreadRuntime::default();
        rt.run(sync_fn, false);
    }

    #[test]
    fn bronze_runtime_run_safe() {
        let sync_fn = SyncFn(|| info!("I am a sync function"));
        let rt = ThreadRuntime::default();
        rt.run_safe(sync_fn, false);
    }

    #[test]
    fn custom_runnable() {
        struct CustomRunnable {}
        impl CustomRunnable {
            pub fn new() -> Self {
                CustomRunnable {}
            }
        }
        impl Runnable for CustomRunnable {
            fn run_async(&self) -> Self::Handle {
                RuntimeJoinHandle::SyncJobHandle
            }
        }
        test_basic_runnable(CustomRunnable::new(), false, false, check_run_result);

        let s = SafeWrappedRunner(Arc::new(Mutex::new(WrappedRunner(Box::new(
            CustomRunnable::new(),
        )))));
        test_basic_runnable(s, false, false, check_run_result);
    }

    #[test]
    fn type_name_type_id() {
        let r1 = SyncFn(|| println!("runnable"));
        let name = r1.run_type_name();
        let id = r1.run_type_id();
        debug!("type name: {}, type id: {:?}", name, id);
    }
}
