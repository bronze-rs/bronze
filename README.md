# bronze

Scheduler in rust

## TODOLIST

### Core features

- [x] Add executor plugin system for dag and task.
- [x] Support async function and routine.
- [ ] Save and load task from dynamic link libraries(ffi)
- [ ] Optimization task design for better expansion
- [ ] Multi-language support.
- [ ] Add time trigger with native event loop(epoll, kqueue, etc.).
- [ ] Add not thread safe version for high performance.
- [ ] Add "exactly once" execute semantics.
- [ ] Add uniform common storage support which will used by other storage plugin, like database, zk etc.
- [ ] Add database storage.

### unit test

- [x] ```bronze-time/schedule_time.rs``` need full unit test
- [x] ```bronze-core/runtime/mod.rs```
