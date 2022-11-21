use bronzeflow::prelude::*;
use std::time::Duration;

fn main() {
    let mut s = SessionBuilder::default().build().unwrap();
    s.submit_new("1/2 * * * * *", SyncFn(|| println!("hello")))
        .unwrap();
    s.submit_new("1/2 * * * * *", MyCustomTask::default())
        .unwrap();
    // s.submit("", WrappedRunner(Box::new(MyCustomTask::new()))).unwrap();
    std::thread::sleep(Duration::from_secs(3));
}

#[derive(Default)]
pub struct MyCustomTask {}

impl Runnable for MyCustomTask {
    fn run_async(&self) -> Self::Handle {
        println!("My custom task");
        RuntimeJoinHandle::SyncJobHandle
    }
}
