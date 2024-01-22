use crate::proc;

use super::{Task, TaskId};
use crate::proc::ProcessId;
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::future::Future;
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(128)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: impl Future<Output = ()> + 'static) {
        let task = Task::new(task);
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("Task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("Task queue is full");
    }

    /// only return when init process is not alive
    pub fn run(&mut self, init: ProcessId) {
        loop {
            self.run_ready_tasks();
            if proc::still_alive(init) {
                self.sleep_if_idle();
            } else {
                break;
            }
        }
    }

    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new_waker(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();
        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue
            .push(self.task_id)
            .expect("Task queue is full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
