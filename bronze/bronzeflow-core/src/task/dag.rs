use crate::prelude::{RuntimeJoinHandle, SyncFn};
use crate::runtime::{
    BuildFromRunnable, Runnable, RunnableMetadata, RunnableMetadataBuilder, SafeMetadata,
};
use crate::task::{TaskInfo, TryIntoTask, WrappedTask};
use bronzeflow_time::prelude::ScheduleExpr;
use bronzeflow_time::schedule_time::ScheduleTimeHolder;
use bronzeflow_utils::{BronzeError, Result};
use std::sync::{Arc, Mutex};

pub type DepTaskNode = Arc<Mutex<TaskNode>>;
pub type TimeHoldType = Arc<Mutex<ScheduleTimeHolder>>;

// #[derive(Debug)]
pub struct TaskNode {
    pub(crate) task: TaskInfo,
    pub(crate) meta: Option<RunnableMetadata>,
    pub(crate) parents: Vec<DepTaskNode>,
    pub(crate) children: Vec<DepTaskNode>,
}

#[derive(Clone)]
pub struct DAG {
    // name: Option<String>,
    root_tasks: Vec<DepTaskNode>,
    schedule: Option<ScheduleExpr>,
    pub(crate) meta: Option<SafeMetadata>,
}

impl<T: TryIntoTask> From<T> for DAG {
    fn from(value: T) -> Self {
        let new_node = Arc::new(Mutex::new(TaskNode::new(value.try_into_task())));
        DAG::new(vec![new_node])
    }
}

impl BuildFromRunnable for DAG {
    type Type = DAG;
    fn build_from(runnable: impl Runnable<Handle = RuntimeJoinHandle<()>> + Send + 'static) -> DAG {
        let task = TaskInfo::build_from(runnable);
        let new_node = Arc::new(Mutex::new(TaskNode::new(task)));
        DAG::new(vec![new_node])
    }
}

impl TaskNode {
    pub fn new(task: TaskInfo) -> Self {
        TaskNode {
            task,
            meta: Some(RunnableMetadata::default()),
            parents: vec![],
            children: vec![],
        }
    }

    pub fn run(&mut self) {
        self.task.0.as_ref().lock().unwrap().0.run();
    }

    pub fn with_meta<T>(meta: T, task: TaskInfo) -> Self
    where
        T: Into<RunnableMetadata>,
    {
        TaskNode {
            task,
            meta: Some(meta.into()),
            parents: vec![],
            children: vec![],
        }
    }
}

unsafe impl Send for TaskNode {}

impl DAG {
    pub fn new(root_tasks: Vec<DepTaskNode>) -> Self {
        DAG {
            root_tasks,
            schedule: None,
            // name: None,
            meta: None,
        }
    }

    pub fn set_schedule(&mut self, schedule: ScheduleExpr) {
        self.schedule = Some(schedule);
    }

    pub fn handle_top_node<F>(nodes: &Vec<DepTaskNode>, f: &mut F)
    where
        F: FnMut(DepTaskNode),
    {
        for node in nodes {
            let parents = &node.as_ref().lock().unwrap().parents;
            if !parents.is_empty() {
                DAG::handle_top_node(parents, f);
            } else {
                f(node.clone());
            }
        }
    }

    pub fn prepare(&mut self) {
        let mut time_holder = ScheduleTimeHolder::new(self.schedule.take().unwrap());
        time_holder.init();
        self.meta = Some(Arc::new(Mutex::new(
            RunnableMetadataBuilder::default()
                .id(None)
                .name(None)
                .maximum_run_times(None)
                .maximum_parallelism(None)
                .schedule(Some(time_holder))
                .build()
                .unwrap(),
        )));
    }

    pub fn run(&mut self) {
        self.for_all_task(|t| {
            t.as_ref().lock().unwrap().run();
        })
    }

    pub fn for_all_task<F>(&self, f: F)
    where
        F: Fn(DepTaskNode),
    {
        for t in &self.root_tasks {
            DAG::handle(false, t.clone(), |task| {
                // task.as_ref().lock().unwrap().run();
                f(task)
            })
        }
    }

    pub fn print_tree(&self) {
        let mut s = vec![];
        let f = |node: DepTaskNode, level| {
            for _ in 0..level {
                s.push(String::from("  "));
            }
            s.push(
                node.as_ref()
                    .lock()
                    .unwrap()
                    .meta
                    .as_ref()
                    .unwrap()
                    .name
                    .as_ref()
                    .map_or_else(|| "".to_string(), |r| r.to_string()),
            );
            s.push("\n".to_string());
        };
        self.handle_with_level(f);
        let tree_str = s.join("");
        println!("{}", tree_str);
    }

    // TODO
    pub fn print_in_one_tree(&self) {
        let mut vnode = TaskNode::new(TryIntoTask::try_into_task(SyncFn(|| println!("root"))));
        for node in &self.root_tasks {
            vnode.children.push(node.clone());
        }
    }

    pub fn handle<F>(parent: bool, node: DepTaskNode, mut f: F)
    where
        F: FnMut(DepTaskNode),
    {
        f(node.clone());
        let task = node.as_ref().lock().unwrap();
        let list = if parent {
            &task.parents
        } else {
            &task.children
        };
        for l in list {
            f(l.clone());
        }
    }

    pub fn handle_with_level<F>(&self, mut f: F)
    where
        F: FnMut(DepTaskNode, usize),
    {
        for node in &self.root_tasks {
            DAG::handle_one_with_level(node.clone(), 0, &mut f);
        }
    }

    pub(crate) fn handle_one_with_level<F>(node: DepTaskNode, level: usize, f: &mut F)
    where
        F: FnMut(DepTaskNode, usize),
    {
        f(node.clone(), level);
        let task = node.as_ref().lock().unwrap();
        let level = level + 1;
        for t in &task.children {
            DAG::handle_one_with_level(t.clone(), level, f)
        }
    }

    pub fn cal_task_nums(&self) -> usize {
        // TODO use better method
        let mut v = vec![];
        self.handle_with_level(|t, _| v.push(t));
        v.len()
    }

    pub fn to_single_task(mut self) -> Result<WrappedTask> {
        let t =
            self.root_tasks.into_iter().next().ok_or_else(|| {
                BronzeError::msg("DAG is empty, could not transform to single task")
            })?;
        let meta = self.meta.take();
        let lock = Arc::try_unwrap(t)
            .ok()
            .ok_or_else(|| BronzeError::msg("Could take ownership from task point in DAG"))?;
        let task_node = lock
            .into_inner()
            .ok()
            .ok_or_else(|| BronzeError::msg("Could take ownership from locked task in DAG"))?;
        Ok(WrappedTask::new(task_node.task, meta))
    }
}
