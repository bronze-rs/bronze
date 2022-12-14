// This is a part of bronze.

//! TokioRuntime, wrap tokio runtime
//!
//! TokioRuntime design to use tokio in a synchronous environment

use crate::runtime::event_loop::TaskEvent::TaskPending;
use crate::runtime::event_loop::{EventReceiver, EventSender, TaskEvent};
use crate::runtime::{BronzeRuntime, Runnable, RuntimeJoinHandle};
use bronzeflow_utils::{debug, info, BronzeError};
use std::sync::Arc;
use std::thread::Builder as StdThreadBuilder;
use tokio::runtime::{Builder, Runtime as TokioRawRuntime};
use tokio::sync::mpsc;

type MessageSender = mpsc::Sender<Message>;
type MessageReceiver = mpsc::Receiver<Message>;

#[derive(Debug)]
enum Message {
    TaskEnd(RuntimeJoinHandle<()>),
}

pub struct TokioRuntime {
    pub(crate) runtime: Arc<TokioRawRuntime>,
    message_tx: MessageSender,
}

impl Default for TokioRuntime {
    fn default() -> Self {
        TokioRuntime::new()
    }
}

impl TokioRuntime {
    pub fn new() -> Self {
        let runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()
            .unwrap();

        let (tx, rx) = mpsc::channel(100);
        let rt = TokioRuntime {
            runtime: Arc::new(runtime),
            message_tx: tx,
        };
        let event_handle = EventHandle::new(rx);
        rt.run_event_handle(event_handle);
        rt
    }
    fn run_event_handle(&self, mut event_handle: EventHandle) {
        let rt = self.runtime.clone();
        StdThreadBuilder::new()
            .name("event_handle".into())
            .spawn(move || {
                rt.block_on(async {
                    info!("Start event loop handle");
                    event_handle.run_loop().await;
                })
            })
            .expect("event_handle can't start.");
    }
}

impl BronzeRuntime for TokioRuntime {
    fn run(&self, _: impl Runnable, _: bool) {
        panic!("Not supported in `TokioRuntime`, please use `run_safe`")
    }

    fn run_with(&self, _: impl Runnable, _: bool, _: impl EventSender<TaskEvent> + 'static) {
        panic!("Not supported in `TokioRuntime`, please use `run_safe`")
    }

    #[inline(always)]
    fn run_safe<F>(&self, runnable: F, report_msg: bool)
    where
        F: Runnable + Send + Sync + 'static,
    {
        let handle = self.runtime.spawn({
            async move {
                runnable.run_async();
            }
        });
        if report_msg {
            self.message_tx
                .blocking_send(Message::TaskEnd(RuntimeJoinHandle::AsyncTokioJoinHandle(
                    handle,
                )))
                .unwrap();
        }
    }

    fn run_safe_with<F>(&self, runnable: F, _: bool, sender: impl EventSender<TaskEvent> + 'static)
    where
        F: Runnable + Send + Sync + 'static,
    {
        let cs = sender.clone_new();
        // TODO handle result
        sender.send(TaskEvent::TaskPending).unwrap();
        let handle = self.runtime.spawn({
            async move {
                // TODO handle result
                cs.send(TaskPending).unwrap();
                runnable.run_async();
            }
        });
        // TODO handle result
        sender
            .send(TaskEvent::TaskRunning(
                RuntimeJoinHandle::AsyncTokioJoinHandle(handle),
            ))
            .unwrap();
    }
}

#[derive(Clone)]
pub struct TokioEventSender<T> {
    sender: mpsc::Sender<T>,
}

impl<T> TokioEventSender<T> {
    pub fn new(sender: mpsc::Sender<T>) -> Self {
        TokioEventSender { sender }
    }
}

unsafe impl<T> Send for TokioEventSender<T> {}

impl<T> EventSender<T> for TokioEventSender<T> {
    type Sender<S> = ();

    fn send(&self, value: T) -> bronzeflow_utils::Result<()> {
        self.sender
            .blocking_send(value)
            .or(Err(BronzeError::msg("Send message failed")))
    }

    fn clone_new(&self) -> Self {
        TokioEventSender {
            sender: self.sender.clone(),
        }
    }
}

pub struct TokioEventReceiver<T> {
    recv: mpsc::Receiver<T>,
}

impl<T> TokioEventReceiver<T> {
    pub fn new(recv: mpsc::Receiver<T>) -> Self {
        TokioEventReceiver { recv }
    }
}

impl<T> EventReceiver<T> for TokioEventReceiver<T> {
    type Receiver<R> = ();

    fn recv(&mut self) -> Option<T> {
        self.recv.blocking_recv()
    }
}

struct EventHandle {
    message_receiver: MessageReceiver,
}

impl EventHandle {
    pub fn new(message_receiver: MessageReceiver) -> Self {
        EventHandle { message_receiver }
    }

    pub async fn run_loop(&mut self) {
        while let Some(event) = self.message_receiver.recv().await {
            // TODO add message dispatcher
            match event {
                Message::TaskEnd(RuntimeJoinHandle::AsyncTokioJoinHandle(jh)) => {
                    debug!("Receive tokio join handle message");
                    jh.await.map_err(BronzeError::msg).ok();
                },
                Message::TaskEnd(RuntimeJoinHandle::SyncJobHandle) => {},
                _ => {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::runtime::tokio_runtime::{EventHandle, TokioRuntime};
    use crate::runtime::{AsyncFn, BronzeRuntime};

    #[tokio::test]
    async fn test_event_loop() {
        let (tx, rx) = mpsc::channel(100);
        let mut event_loop = EventHandle::new(rx);
        StdThreadBuilder::new()
            .name("event_handle".into())
            .spawn(move || {
                futures::executor::block_on({
                    async move {
                        event_loop.run_loop().await;
                    }
                });
            })
            .expect("event loop can't start.");
        let f = tokio::spawn(async {});
        futures::executor::block_on(async {});

        tx.send(Message::TaskEnd(RuntimeJoinHandle::AsyncTokioJoinHandle(f)))
            .await
            .expect("Send message error");
        tx.send(Message::TaskEnd(RuntimeJoinHandle::FutureBlockJoinHandle(
            (),
        )))
        .await
        .expect("Send message error");
        tx.send(Message::TaskEnd(RuntimeJoinHandle::SyncJobHandle))
            .await
            .expect("Send message error");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }

    #[test]
    fn tokio_runtime_in_synchronous_env() {
        let rt = TokioRuntime::new();
        let f = || async { info!("I am async function in synchronous environment") };
        rt.run_safe(AsyncFn::from(f), false);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    #[tokio::test]
    async fn tokio_runtime_in_asynchronous_env() {
        let rt = TokioRuntime::new();
        let f = || async { info!("I am async function asynchronous environment") };
        rt.run_safe(AsyncFn::from(f), false);
        tokio::spawn(async { info!("I am common async function") })
            .await
            .expect("run common async function failed");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
