use core::{
    task::{
        Context,
        Poll,
    },
    future::Future,
    pin::Pin
};
use alloc::boxed::Box;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new<T: 'static + Future<Output=()>>(raw_future: T) -> Task {
        let future = Box::pin(raw_future);
        Task {
            future,
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        let future_ref = self.future.as_mut();
        future_ref.poll(context)
    }
}
