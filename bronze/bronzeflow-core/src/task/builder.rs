use crate::runtime::RunnableMetadata;
use crate::task::dag::{DepTaskNode, TaskNode, DAG};
use crate::task::TryIntoTask;
use bronzeflow_utils::Result;
use std::borrow::BorrowMut;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct DAGBuilder {
    curr_node: Option<DepTaskNode>,
    roots: Vec<DepTaskNode>,
}

impl DAGBuilder {
    pub fn new() -> DAGBuilder {
        DAGBuilder {
            curr_node: None,
            roots: vec![],
        }
    }

    /// Add a task to DAG as dag node
    ///
    pub fn task<M, T>(mut self, meta: M, task: T) -> Self
    where
        M: Into<RunnableMetadata>,
        T: TryIntoTask,
    {
        if let Some(node) = self.curr_node {
            self.roots.push(node);
        }
        let new_node = Some(Arc::new(Mutex::new(TaskNode::with_meta(
            meta,
            task.try_into_task(),
        ))));
        self.curr_node = new_node;
        self
    }

    /// Set name for current task
    pub fn set_name(mut self, name: &str) -> Self {
        let node = self.curr_node.take();
        if let Some(node) = node {
            node.as_ref()
                .lock()
                .unwrap()
                .borrow_mut()
                .meta
                .as_mut()
                .map(|r| r.set_name(name.to_string()));
            self.curr_node = Some(node);
        }
        self
    }

    /// Add a parent task to current task
    pub fn parent<F>(mut self, mut bf: F) -> Self
    where
        F: FnMut(Self) -> Self,
    {
        let parents = bf(DAGBuilder::new()).build_vec();

        if let Some(ref mut node) = self.curr_node {
            for p in parents {
                p.as_ref().lock().unwrap().children.push(node.clone());
                node.as_ref().lock().unwrap().parents.push(p.clone());
            }
        }
        self
    }

    /// Add a child task to current task
    pub fn child<F>(mut self, mut bf: F) -> Self
    where
        F: FnMut(Self) -> Self,
    {
        let children = bf(DAGBuilder::new()).build_vec();

        if let Some(ref mut node) = self.curr_node {
            for c in children {
                c.as_ref().lock().unwrap().parents.push(node.clone());
                node.as_ref().lock().unwrap().children.push(c.clone());
            }
        }
        self
    }

    pub fn merge(mut self, mut other: DAGBuilder) -> Self {
        if self.curr_node.is_none() {
            self.curr_node = other.curr_node.take();
        }
        self.roots.append(&mut other.build_vec());
        self
    }

    fn build_vec(mut self) -> Vec<DepTaskNode> {
        if let Some(node) = self.curr_node {
            self.roots.push(node);
        }
        self.roots
    }

    pub fn build(self) -> Result<DAG> {
        let vec = self.build_vec();
        let mut roots = vec![];
        let mut f = |node: DepTaskNode| {
            roots.push(node);
        };
        DAG::handle_top_node(&vec, &mut f);
        Ok(DAG::new(roots))
    }
}

impl<F: TryIntoTask> From<F> for DAGBuilder {
    fn from(task: F) -> Self {
        DAGBuilder::new().task("", task)
    }
}

#[macro_export]
macro_rules! dag {
    ($dag_builder:stmt) => {{
        {
            $dag_builder
        }
    }};

    ($task_name:expr => $task:expr => $parent:expr) => {
        {
            let tmp_dag = dag! {$task_name => $task};
            tmp_dag.parent(|_| {
                dag!($parent)
            })
        }
    };

    ( $( $task_name:expr => $task:expr ), + $(,)?) => {
        {
            let mut builder = $crate::task::builder::DAGBuilder::new();
            $(
                let task = dag!{$task};
                let tmp_builder = $crate::task::builder::DAGBuilder::from(task).set_name($task_name);
                builder = builder.merge(tmp_builder);
            )+
            builder
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{AsyncFn, SyncFn};

    #[test]
    fn create_dag() {
        let _ = DAGBuilder::new()
            .task("T1", || {
                println!("I am a task");
            })
            .build()
            .unwrap();
        let _ = DAGBuilder::new()
            .task(
                "T2",
                AsyncFn(|| async {
                    println!("I am a asynchronous task");
                }),
            )
            .build()
            .unwrap();

        let _ = DAGBuilder::from(SyncFn(|| {
            println!("Task");
        }));
    }

    #[test]
    fn create_dag_with_parent_and_child() {
        let d = DAGBuilder::new()
            .task("Task C", || println!("Task C"))
            .parent(|bd: DAGBuilder| {
                bd.task("Task D", || println!("Task D"))
                    .task("Task D1", || println!("Task D1"))
                    .parent(|bd2| {
                        bd2.task("Task E1", || {})
                            .task("Task E2", || {})
                            .task("Task E3", || {})
                    })
            })
            .child(|bd: DAGBuilder| {
                bd.task("Task C1", || println!("Task C1"))
                    .task("Task C2", || println!("Task C2"))
                    .child(|bd2| bd2.task("Task B1", || {}).task("Task B2", || {}))
            })
            .build()
            .unwrap();
        d.print_tree();
    }

    #[test]
    fn test_macro_create_dag() {
        let d1 = dag!("first" => || {println!("Task first")})
            .build()
            .unwrap();
        let d2 = dag!(
            "hello" => || println!("Task A"),
            "name" => || println!("Task A"),
            "fn tas" => ||{println!("Im function task")}
        )
        .build()
        .unwrap();
        d1.print_tree();
        d2.print_tree();
    }

    #[test]
    fn test_macro_single_parent() {
        let d = dag!(
            "P1A" => ||println!("P1-A") => dag!(
                "P1A1" => ||println!("P1-A1")
            )
        )
        .build()
        .unwrap();
        d.print_tree();
    }

    #[test]
    fn test_macro_multi_parents() {
        let d = dag!(
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
        d.print_tree();
    }
}
