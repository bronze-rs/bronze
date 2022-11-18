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

    pub fn task<T>(mut self, task: T) -> Self
    where
        T: TryIntoTask,
    {
        if let Some(node) = self.curr_node {
            self.roots.push(node);
        }
        let new_node = Some(Arc::new(Mutex::new(TaskNode::new(task.try_into_task()))));
        self.curr_node = new_node;
        self
    }

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

    pub fn p<F>(mut self, mut bf: F) -> Self
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

    pub fn c<F>(mut self, mut bf: F) -> Self
    where
        F: FnMut(Self) -> Self,
    {
        let children = bf(DAGBuilder::new()).build_vec();

        if let Some(ref mut node) = self.curr_node {
            for c in children {
                c.as_ref().lock().unwrap().children.push(node.clone());
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
        DAGBuilder::new().task(task)
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
            tmp_dag.p(|_| {
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
    #[test]
    fn test_create_dag() {
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
    fn test_single_parent() {
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
    fn test_multi_parents() {
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
