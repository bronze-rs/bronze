use crate::executor::DefaultExecutor;
use crate::manager::ScheduleManager;
use crate::prelude::{Executor, ThreadTrigger, Trigger, DAG};
use crate::service::Service;
use crate::store::{MemoryStorage, Storage};
use crate::task::RunnableHolder;
use bronzeflow_time::prelude::ScheduleExpr;
use bronzeflow_utils::{debug, BronzeError, Result};
use std::fmt::Debug;

pub trait Session: Service {
    fn submit<D, S>(&mut self, s: S, try_into_dag: D) -> Result<()>
    where
        D: Into<DAG>,
        S: TryInto<ScheduleExpr>,
        BronzeError: From<S::Error>,
    {
        let schedule_expr = s.try_into()?;
        let mut dag = try_into_dag.into();
        dag.set_schedule(schedule_expr);
        dag.prepare();

        let runnable = if dag.cal_task_nums() == 1 {
            debug!("Dag has one task, transform it to single task");
            let task = dag.to_single_task()?;
            RunnableHolder::Task(task)
        } else {
            RunnableHolder::Dag(dag)
        };
        self.submit_runnable(runnable)
    }

    fn submit_runnable(&mut self, runnable: RunnableHolder) -> Result<()>;

    fn build_session(&mut self) -> Result<()>;
}

pub struct LocalSession<SG: Storage + 'static, TG: Trigger + 'static, E: Executor + 'static> {
    manager: Option<ScheduleManager<SG, TG, E>>,
    trigger: Option<TG>,
    storage: Option<SG>,
    executor: Option<E>,
}

impl<SG: Storage, TG: Trigger, E: Executor> LocalSession<SG, TG, E> {
    pub fn new(storage: Option<SG>, trigger: Option<TG>, executor: Option<E>) -> Self {
        LocalSession {
            manager: None,
            trigger,
            storage,
            executor,
        }
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Service for LocalSession<SG, TG, E> {
    fn start(&mut self) {
        self.manager.as_mut().unwrap().start();
    }

    fn stop(&mut self) {
        self.manager.as_mut().unwrap().stop();
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Drop for LocalSession<SG, TG, E> {
    fn drop(&mut self) {
        if let Some(ref mut m) = self.manager {
            m.stop();
        }
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Session for LocalSession<SG, TG, E> {
    fn submit_runnable(&mut self, runnable: RunnableHolder) -> Result<()> {
        self.manager.as_mut().unwrap().add_runnable(runnable);
        Ok(())
    }

    fn build_session(&mut self) -> Result<()> {
        let storage = self
            .storage
            .take()
            .ok_or_else(|| BronzeError::msg("Please set storage first"))?;
        let trigger = self
            .trigger
            .take()
            .ok_or_else(|| BronzeError::msg("Please set trigger first"))?;
        let executor = self
            .executor
            .take()
            .ok_or_else(|| BronzeError::msg("Please set executor first"))?;
        self.manager = Some(ScheduleManager::new(storage, trigger, executor));
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionBuilder<
    SG: Storage = MemoryStorage,
    TG: Trigger = ThreadTrigger,
    E: Executor = DefaultExecutor,
    S: Session = LocalSession<SG, TG, E>,
> {
    /// Scheduler backend address
    uri: Option<String>,

    /// Schedule trigger
    trigger: Option<TG>,

    /// The storage of DAG
    storage: Option<SG>,

    executor: Option<E>,

    /// Schedule session
    #[allow(dead_code)]
    session: Option<S>,
}

impl SessionBuilder {
    pub fn set_uri(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_string());
        self
    }
}

impl<SG: Storage, TG: Trigger, E: Executor, S: Session> SessionBuilder<SG, TG, E, S> {
    pub fn trigger(mut self, trigger: TG) -> Self {
        self.trigger = Some(trigger);
        self
    }

    pub fn storage(mut self, storage: SG) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn executor(mut self, executor: E) -> Self {
        self.executor = Some(executor);
        self
    }

    pub fn get(&mut self) -> Result<(SG, TG, E)> {
        let storage = self
            .storage
            .take()
            .ok_or_else(|| BronzeError::msg("Please set storage first"))?;
        let trigger = self
            .trigger
            .take()
            .ok_or_else(|| BronzeError::msg("Please set trigger first"))?;
        let executor = self
            .executor
            .take()
            .ok_or_else(|| BronzeError::msg("Please set executor first"))?;
        Ok((storage, trigger, executor))
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> SessionBuilder<SG, TG, E, LocalSession<SG, TG, E>> {
    pub fn build(mut self) -> Result<LocalSession<SG, TG, E>> {
        let (storage, trigger, executor) = self.get()?;
        let mut session = LocalSession::new(Some(storage), Some(trigger), Some(executor));
        let _ = &session.build_session()?;
        session.start();
        Ok(session)
    }
}

pub trait LocalSessionFactory {
    fn local() -> Self;
}

pub trait DefaultSessionFactory {
    fn default() -> Self;
}

impl DefaultSessionFactory
    for SessionBuilder<
        MemoryStorage,
        ThreadTrigger,
        DefaultExecutor,
        LocalSession<MemoryStorage, ThreadTrigger, DefaultExecutor>,
    >
{
    fn default() -> Self {
        let storage = Some(MemoryStorage::new());
        let trigger = Some(ThreadTrigger::new());
        let executor = Some(DefaultExecutor::default());
        SessionBuilder {
            uri: None,
            storage,
            trigger,
            executor,
            session: Some(LocalSession::new(
                Some(MemoryStorage::new()),
                Some(ThreadTrigger::new()),
                Some(DefaultExecutor::default()),
            )),
        }
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> LocalSessionFactory
    for SessionBuilder<SG, TG, E, LocalSession<SG, TG, E>>
{
    fn local() -> Self {
        SessionBuilder {
            uri: None,
            storage: None,
            trigger: None,
            executor: None,
            session: None,
        }
    }
}

pub trait RemoteSessionFactory {
    fn remote() -> Self;
}

pub struct RemoteSession<SG: Storage + 'static, TG: Trigger + 'static, E: Executor + 'static> {
    manager: Option<ScheduleManager<SG, TG, E>>,
    #[allow(dead_code)]
    trigger: Option<TG>,
    #[allow(dead_code)]
    storage: Option<SG>,
    #[allow(dead_code)]
    executor: Option<E>,
}

impl<SG: Storage, TG: Trigger, E: Executor> RemoteSession<SG, TG, E> {
    pub fn new(storage: Option<SG>, trigger: Option<TG>, executor: Option<E>) -> Self {
        RemoteSession {
            manager: None,
            trigger,
            storage,
            executor,
        }
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Service for RemoteSession<SG, TG, E> {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Drop for RemoteSession<SG, TG, E> {
    fn drop(&mut self) {
        if let Some(ref mut m) = self.manager {
            m.stop();
        }
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> Session for RemoteSession<SG, TG, E> {
    fn submit_runnable(&mut self, _: RunnableHolder) -> Result<()> {
        todo!()
    }

    fn build_session(&mut self) -> Result<()> {
        // let storage = self
        //     .storage
        //     .take()
        //     .ok_or(BronzeError::msg("Please set storage first"))?;
        // let trigger = self
        //     .trigger
        //     .take()
        //     .ok_or(BronzeError::msg("Please set trigger first"))?;
        // let executor = self
        //     .executor
        //     .take()
        //     .ok_or(BronzeError::msg("Please set executor first"))?;
        Ok(())
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> SessionBuilder<SG, TG, E, RemoteSession<SG, TG, E>> {
    pub fn build(mut self) -> Result<RemoteSession<SG, TG, E>> {
        let (storage, trigger, executor) = self.get()?;
        let mut session = RemoteSession::new(Some(storage), Some(trigger), Some(executor));
        session.build_session()?;
        Ok(session)
    }
}

impl<SG: Storage, TG: Trigger, E: Executor> RemoteSessionFactory
    for SessionBuilder<SG, TG, E, RemoteSession<SG, TG, E>>
{
    fn remote() -> Self {
        SessionBuilder {
            uri: None,
            storage: None,
            trigger: None,
            executor: None,
            session: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag;
    use crate::prelude::*;
    use std::{thread, time};

    fn get_dag() -> DAG {
        dag!(
            "P1A" => ||println!("P1-A") => dag!(
                "P1A1" => ||println!("P1-A1")
            )
        )
        .build()
        .unwrap()
    }

    fn test_local_session(
        d: DAG,
        trigger: impl Trigger + 'static,
        storage: impl Storage + 'static,
        executor: impl Executor + 'static,
    ) {
        let mut s = SessionBuilder::local()
            .trigger(trigger)
            .storage(storage)
            .executor(executor)
            .build()
            .unwrap();
        s.submit("1/10 * * * * *", d).unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }

    #[test]
    fn create_default_local_session() {
        let mut s = SessionBuilder::default().build().unwrap();
        let d = get_dag();
        s.submit("1/10 * * * * *", d).unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }

    #[test]
    fn create_local_session() {
        // let tokio_rt = Arc::new(TokioRuntime::new());
        let mut s = SessionBuilder::local()
            .trigger(ThreadTrigger::new())
            .storage(MemoryStorage::new())
            .executor(DefaultExecutor::default())
            .build()
            .unwrap();
        let d = get_dag();
        s.submit("1/10 * * * * *", d).unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }

    #[cfg(feature = "async_tokio")]
    #[tokio::test]
    async fn create_local_session_with_tokio_runtime() {
        use std::sync::Arc;
        let tokio_rt = Arc::new(TokioRuntime::new());
        let mut s = SessionBuilder::local()
            .trigger(TokioTrigger::new(Arc::clone(&tokio_rt)))
            .storage(MemoryStorage::new())
            .executor(TokioExecutor::new(tokio_rt))
            .build()
            .unwrap();

        let d = get_dag();
        let f = || async {
            println!("Im async function");
        };
        s.submit("1/1 * * * * *", d).unwrap();
        s.submit("1/2 * * * * *", AsyncFn(f)).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
    }

    #[test]
    fn create_and_run() {
        let d = get_dag();
        test_local_session(
            d,
            ThreadTrigger::new(),
            MemoryStorage::new(),
            DefaultExecutor::default(),
        );
    }

    #[test]
    fn submit_sync_fn() {
        let mut s = SessionBuilder::default().build().unwrap();
        s.submit("1/1 * * * * *", || println!("I am sync function"))
            .unwrap();
        thread::sleep(time::Duration::from_secs(2));
    }

    #[cfg(feature = "async_tokio")]
    #[tokio::test]
    async fn run_async_unction() {
        use std::sync::Arc;
        let tokio_rt = Arc::new(TokioRuntime::new());
        let mut s = SessionBuilder::local()
            .trigger(TokioTrigger::new(Arc::clone(&tokio_rt)))
            .storage(MemoryStorage::new())
            .executor(TokioExecutor::new(tokio_rt))
            .build()
            .unwrap();
        s.submit(
            "1/1 * * * * *",
            AsyncFn(|| async { println!("I am asynchronous task") }),
        )
        .unwrap();
    }
}
