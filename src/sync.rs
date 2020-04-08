use spin::Mutex;

pub struct Locked<T> {
    mutex: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(to_lock: T) -> Locked<T> {
        let mutex = Mutex::new(to_lock);
        Locked {
            mutex,
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.mutex.lock()
    }
}
