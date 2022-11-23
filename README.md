## What is bronze?

A workflow scheduler in rust

## Get Started

TODO

## Examples

1. Create workflow by macro ```dag``` and submit to scheduler with cron expression

```rust
use bronzeflow::prelude::*;

fn main() {
    // Create a session to submit task/workflow to scheduler
    let mut s = SessionBuilder::default().build().unwrap();

    // Create a workflow
    let wf = dag!(
        "A" => || println!("Task A") => dag!(
            "A1" => || println!("Task A1"),
            "A2" => || println!("Task A2"),
            "C1" => dag!(
                "C1" => || println!("Task C1") => dag!(
                    "C11" => || println!("Task C11")
                )
            )
        )
    )
    .build()
    .unwrap();

    // Print the dependencies tree
    wf.print_tree();

    // Submit workflow to scheduler
    s.submit("1/2 * * * * *", wf).unwrap();

    // Sleep 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
```
## Development

### Build from source

TODO

## Todo list

### Core features

- [x] Add executor plugin system for dag and task.
- [x] Support async function and routine.
- [x] Use github CI/CD
- [ ] Add uniform message event loop to report task state
- [ ] All tasks in DAG support sequential execution according to dependencies(topological sort)
- [ ] Save and load task from dynamic link libraries(ffi)
- [ ] Optimization task design for better expansion
- [ ] Multi-language support.
- [ ] Add time trigger with native event loop(epoll, kqueue, etc.).
- [ ] Add not thread safe version for high performance.
- [ ] Add "exactly once" execute semantics.
- [ ] Add uniform common storage support which will used by other storage plugin, like database, zk etc.
- [ ] Add database storage.
