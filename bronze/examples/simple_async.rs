use bronzeflow::prelude::*;
use bronzeflow_core::store::MemoryStorage;
use std::sync::Arc;

#[cfg(feature = "async_tokio")]
fn main() {
    let rt = Arc::new(TokioRuntime::new());
    let mut s = SessionBuilder::local()
        .trigger(TokioTrigger::new(Arc::clone(&rt)))
        .executor(TokioExecutor::new(rt))
        .storage(MemoryStorage::new())
        .build()
        .unwrap();
    s.submit(
        "1/1 * * * * *",
        AsyncFn(|| async {
            println!("I am asynchronous task");
        }),
    )
    .unwrap();
}
