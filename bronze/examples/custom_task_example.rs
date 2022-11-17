use bronze::prelude::*;

fn main() {
    let mut s = SessionBuilder::default().build().unwrap();
    s.submit_new("1/2 * * * * *", SyncFn(|| println!("hello")))
        .unwrap();
    s.submit_new("1/2 * * * * *", MyCustomTask::default())
        .unwrap();
    // s.submit("", WrappedRunner(Box::new(MyCustomTask::new()))).unwrap();
}

#[derive(Default)]
pub struct MyCustomTask {}

impl Runnable for MyCustomTask {
    fn run_async(&self) -> Self::Handle {
        println!("My custom task");
        RuntimeJoinHandle::SyncJobHandle
    }
}
