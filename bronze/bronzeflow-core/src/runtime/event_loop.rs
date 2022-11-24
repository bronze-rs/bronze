use crate::runtime::RuntimeJoinHandle;
use bronzeflow_utils::{BronzeError, Result};

pub trait EventSender<T>: Send {
    type Sender<S>;

    fn send(&self, value: T) -> Result<()>;

    fn clone_new(&self) -> Self;
}

pub trait EventReceiver<T> {
    type Receiver<R>;

    fn recv(&mut self) -> Option<T>;
}

#[derive(Clone)]
pub struct StdEventSender<T> {
    sender: std::sync::mpsc::Sender<T>,
}

unsafe impl<T> Send for StdEventSender<T> {}

pub struct StdEventReceiver<T> {
    recv: std::sync::mpsc::Receiver<T>,
}

impl<T> EventSender<T> for StdEventSender<T> {
    type Sender<S> = ();

    fn send(&self, value: T) -> Result<()> {
        self.sender
            .send(value)
            .or(Err(BronzeError::msg("Send message failed")))
    }

    fn clone_new(&self) -> Self {
        StdEventSender {
            sender: self.sender.clone(),
        }
    }
}

impl<T> EventReceiver<T> for StdEventReceiver<T> {
    type Receiver<R> = ();

    fn recv(&mut self) -> Option<T> {
        self.recv.recv().ok()
    }
}

pub enum TaskEvent {
    TaskPending,

    TaskRunning(RuntimeJoinHandle<()>),

    TaskFinished,
}
