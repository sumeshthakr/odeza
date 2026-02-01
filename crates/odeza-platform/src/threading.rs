//! Threading Primitives
//!
//! Cross-platform threading utilities and thread pool.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam::channel::{bounded, Receiver, Sender};
use parking_lot::{Condvar, Mutex};

/// Thread wrapper with platform-specific optimizations
pub struct Thread {
    handle: Option<JoinHandle<()>>,
    name: String,
}

impl Thread {
    /// Spawn a new thread with the given name and function
    pub fn spawn<F>(name: impl Into<String>, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let name = name.into();
        let thread_name = name.clone();
        
        let handle = thread::Builder::new()
            .name(thread_name)
            .spawn(f)
            .expect("Failed to spawn thread");

        Self {
            handle: Some(handle),
            name,
        }
    }

    /// Get the thread name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Join the thread, waiting for it to complete
    pub fn join(mut self) -> thread::Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.join()
        } else {
            Ok(())
        }
    }

    /// Check if the thread has finished
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().is_none_or(|h| h.is_finished())
    }
}

/// Task for the thread pool
type Task = Box<dyn FnOnce() + Send + 'static>;

/// Thread pool for parallel work
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Task>>,
    shutdown: Arc<AtomicBool>,
    task_count: Arc<AtomicUsize>,
}

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Receiver<Task>,
        shutdown: Arc<AtomicBool>,
        task_count: Arc<AtomicUsize>,
    ) -> Self {
        let thread = thread::Builder::new()
            .name(format!("pool-worker-{}", id))
            .spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok(task) => {
                            task();
                            task_count.fetch_sub(1, Ordering::Relaxed);
                        }
                        Err(crossbeam::channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam::channel::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .expect("Failed to spawn worker thread");

        Self {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    /// Create a new thread pool with the specified number of workers
    pub fn new(num_workers: usize) -> Self {
        let num_workers = num_workers.max(1);
        let (sender, receiver) = bounded(1024);
        let shutdown = Arc::new(AtomicBool::new(false));
        let task_count = Arc::new(AtomicUsize::new(0));

        let workers = (0..num_workers)
            .map(|id| Worker::new(id, receiver.clone(), shutdown.clone(), task_count.clone()))
            .collect();

        Self {
            workers,
            sender: Some(sender),
            shutdown,
            task_count,
        }
    }

    /// Create a thread pool with one worker per CPU core
    pub fn with_cores() -> Self {
        let num_cores = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        Self::new(num_cores)
    }

    /// Get the number of workers
    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    /// Submit a task to the pool
    pub fn submit<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            self.task_count.fetch_add(1, Ordering::Relaxed);
            sender.send(Box::new(task)).expect("Failed to send task");
        }
    }

    /// Get the number of pending tasks
    pub fn pending_tasks(&self) -> usize {
        self.task_count.load(Ordering::Relaxed)
    }

    /// Wait for all tasks to complete
    pub fn wait_all(&self) {
        while self.task_count.load(Ordering::Relaxed) > 0 {
            thread::yield_now();
        }
    }

    /// Check if the pool is idle
    pub fn is_idle(&self) -> bool {
        self.task_count.load(Ordering::Relaxed) == 0
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

/// Barrier for synchronizing multiple threads
pub struct Barrier {
    mutex: Mutex<BarrierState>,
    cvar: Condvar,
    count: usize,
}

struct BarrierState {
    waiting: usize,
    generation: usize,
}

impl Barrier {
    /// Create a new barrier for the given number of threads
    pub fn new(count: usize) -> Self {
        Self {
            mutex: Mutex::new(BarrierState {
                waiting: 0,
                generation: 0,
            }),
            cvar: Condvar::new(),
            count,
        }
    }

    /// Wait at the barrier until all threads have arrived
    pub fn wait(&self) {
        let mut state = self.mutex.lock();
        let generation = state.generation;
        
        state.waiting += 1;
        
        if state.waiting == self.count {
            state.waiting = 0;
            state.generation = state.generation.wrapping_add(1);
            self.cvar.notify_all();
        } else {
            while generation == state.generation {
                self.cvar.wait(&mut state);
            }
        }
    }
}

/// Spinlock for very short critical sections
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    /// Create a new unlocked spinlock
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    /// Acquire the lock
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Spin with a hint to the CPU
            std::hint::spin_loop();
        }
    }

    /// Try to acquire the lock without blocking
    pub fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Release the lock
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    /// Execute a closure with the lock held
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.lock();
        let result = f();
        self.unlock();
        result
    }
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped thread for automatic joining
pub struct ScopedThread<'a, T: Send + 'static> {
    handle: Option<JoinHandle<T>>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, T: Send + 'static> ScopedThread<'a, T> {
    /// Spawn a scoped thread
    pub fn spawn<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let handle = thread::spawn(f);

        Self {
            handle: Some(handle),
            _marker: std::marker::PhantomData,
        }
    }

    /// Join the thread and get the result
    pub fn join(mut self) -> thread::Result<T> {
        self.handle.take().unwrap().join()
    }
}

impl<T: Send + 'static> Drop for ScopedThread<'_, T> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_spawn() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let thread = Thread::spawn("test-thread", move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        });

        thread.join().unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..100 {
            let counter_clone = counter.clone();
            pool.submit(move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            });
        }

        pool.wait_all();
        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(3));
        let counter = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..3)
            .map(|_| {
                let barrier = barrier.clone();
                let counter = counter.clone();
                thread::spawn(move || {
                    counter.fetch_add(1, Ordering::Relaxed);
                    barrier.wait();
                    // All threads should have incremented before any pass this point
                    assert!(counter.load(Ordering::Relaxed) >= 3);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_spinlock() {
        let lock = SpinLock::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let lock = Arc::new(lock);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let lock = lock.clone();
                let counter = counter.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        lock.with_lock(|| {
                            let val = counter.load(Ordering::Relaxed);
                            counter.store(val + 1, Ordering::Relaxed);
                        });
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::Relaxed), 1000);
    }
}
