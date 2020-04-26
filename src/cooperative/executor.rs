use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc};
use alloc::task::Wake;

use core::{
    task::{
        RawWakerVTable,
        RawWaker,
        Context,
        Waker,
        Poll,
    },
};

use crossbeam_queue::ArrayQueue;

use crate::cooperative::task::{Task, TaskId};
use crate::println;

const WAKE_Q_SIZE: usize = 100;

#[cfg(feature = "spawner")]
#[derive(Clone)]
pub struct Spawner {
    spawn_queue: Arc<spin::Mutex<VecDeque<Task>>>,
}

#[cfg(feature = "spawner")]
impl Spawner {
    fn new() -> Spawner {
        let spawn_queue = Arc::new(spin::Mutex::new(VecDeque::new()));
        Spawner {
            spawn_queue,
        }
    }

    pub fn spawn(&mut self, task: Task) -> Result<(), ExecutorError> {
        let task_id = task.id();
        self.spawn_queue.lock().push_back(task);
        Ok(())
    }

    fn tasks(&mut self) -> spin::MutexGuard<VecDeque<Task>> {
        self.spawn_queue.lock()
    }

    fn pop(&mut self) -> Option<Task> {
        self.spawn_queue.lock().pop_front()
    }

    fn len(&self) -> usize {
        self.spawn_queue.lock().len()
    }
}

pub enum ExecutorError {
}

pub struct Executor {
    #[cfg(feature = "spawner")]
    spawner: Spawner,

    task_queue: VecDeque<Task>,
    waiting_tasks: BTreeMap<TaskId, Task>,
    wake_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            #[cfg(feature = "spawner")]
            spawner: Spawner::new(),

            task_queue: VecDeque::new(),
            waiting_tasks: BTreeMap::new(),
            wake_queue: Arc::new(ArrayQueue::new(WAKE_Q_SIZE)),
            waker_cache: BTreeMap::new(),
        }
    }

    #[cfg(feature = "spawner")]
    pub fn spawn(&mut self, task: Task) -> Result<(), ExecutorError> {
        self.spawner.spawn(task)
    }

    #[cfg(not(feature = "spawner"))]
    pub fn spawn(&mut self, task: Task) -> Result<(), ExecutorError> {
        self.task_queue.push_back(task);
        Ok(())
    }

    #[cfg(feature = "spawner")]
    pub fn get_spawner(&self) -> Spawner {
        self.spawner.clone()
    }

    fn create_waker(&self, task: &Task) -> Waker {
        let task_waker = TaskWaker::new(task.id(), self.wake_queue.clone());
        Waker::from(Arc::new(task_waker))
    }

    fn run_ready(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            if !self.waker_cache.contains_key(&task.id()) {
                    self.waker_cache.insert(task.id(), self.create_waker(&task));
            }
            let waker = self.waker_cache.get(&task.id()).expect("should exist");
            let mut context = Context::from_waker(&waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.waker_cache.remove(&task.id());
                }
                Poll::Pending => {
                    if self.waiting_tasks.insert(task.id(), task).is_some() {
                        panic!("Attempted to insert task with non-unique ID");
                    }
                },
            }
        }
    }

    fn wake_tasks(&mut self) {
        while let Ok(task_id) = self.wake_queue.pop() {
            let task = self.waiting_tasks.remove(&task_id).expect("task with unknown ID woke up");
            self.task_queue.push_back(task);
        }
    }

    #[cfg(feature = "spawner")]
    fn spawn_tasks(&mut self) {
        while let Some(task) = self.spawner.pop() {
            self.task_queue.push_back(task);
        }
    }

    pub fn run(&mut self) {
        loop {
            #[cfg(feature = "spawner")]
            self.spawn_tasks();

            self.wake_tasks();
            self.run_ready();

            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        if !self.wake_queue.is_empty() {
            return;
        }

        crate::arch::interrupts::disable();
        if self.wake_queue.is_empty() {
            crate::arch::interrupts::enable_interrupt_halt();
        } else {
            crate::arch::interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    wake_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    pub fn new(task_id: TaskId, wake_queue: Arc<ArrayQueue<TaskId>>) -> TaskWaker {
        TaskWaker {
            task_id,
            wake_queue,
        }
    }

    fn wake_task(&self) {
        self.wake_queue.push(self.task_id).expect("wake_queue full");
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
