use alloc::collections::vec_deque::VecDeque;
use core::task::{
    Waker,
    RawWaker,
    RawWakerVTable,
};

use core::{
    task::{
        Context,
        Poll,
    },
};

use crate::cooperative::task::Task;

pub enum SimpleExecutorError {

}

pub struct SimpleExecutor {
    tasks: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        let tasks = VecDeque::new();
        SimpleExecutor {
            tasks,
        }
    }

    pub fn spawn(&mut self, task: Task) -> Result<(), SimpleExecutorError> {
        self.tasks.push_back(task);
        Ok(())
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.tasks.push_back(task),
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
