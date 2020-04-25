use core::{
    sync::atomic::{AtomicU64, Ordering},
    task::{
        Context,
        Poll,
    },
    future::Future,
    pin::Pin
};
use alloc::boxed::Box;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new<T: 'static + Future<Output=()>>(raw_future: T) -> Task {
        let future = Box::pin(raw_future);
        let id = TaskId::new();
        Task {
            id,
            future,
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        let future_ref = self.future.as_mut();
        future_ref.poll(context)
    }
}
