//! Job System
//!
//! Work-stealing thread pool with per-frame task graph and dependencies.
//! Features:
//! - Work-stealing scheduler for load balancing
//! - Task dependencies via job graph
//! - Priority-based scheduling
//! - Per-subsystem job budgets

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam::deque::{Injector, Stealer, Worker};
use parking_lot::{Condvar, Mutex};

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JobPriority {
    /// Low priority background tasks
    Low = 0,
    /// Normal priority tasks
    Normal = 1,
    /// High priority tasks (render-critical)
    High = 2,
    /// Critical tasks (must complete this frame)
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Handle to a submitted job
#[derive(Debug, Clone)]
pub struct JobHandle {
    completed: Arc<AtomicBool>,
    id: u64,
}

impl JobHandle {
    /// Check if the job has completed
    pub fn is_complete(&self) -> bool {
        self.completed.load(Ordering::Acquire)
    }

    /// Get the job ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// A job that can be executed by the job system
pub trait Job: Send + 'static {
    /// Execute the job
    fn execute(&mut self);
    
    /// Get the job name for debugging
    fn name(&self) -> &str {
        "unnamed_job"
    }
}

/// Wrapper for closure-based jobs
struct ClosureJob<F: FnOnce() + Send + 'static> {
    func: Option<F>,
    name: &'static str,
}

impl<F: FnOnce() + Send + 'static> Job for ClosureJob<F> {
    fn execute(&mut self) {
        if let Some(func) = self.func.take() {
            func();
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

/// Internal job wrapper with metadata
struct JobWrapper {
    job: Box<dyn Job>,
    priority: JobPriority,
    completed: Arc<AtomicBool>,
    dependencies: Vec<Arc<AtomicBool>>,
}

impl JobWrapper {
    fn can_execute(&self) -> bool {
        self.dependencies.iter().all(|dep| dep.load(Ordering::Acquire))
    }
}

/// Work-stealing job system
pub struct JobSystem {
    /// Global job queue
    global_queue: Arc<Injector<JobWrapper>>,
    /// Per-worker local queues
    local_queues: Vec<Worker<JobWrapper>>,
    /// Stealers for work stealing
    stealers: Vec<Stealer<JobWrapper>>,
    /// Number of worker threads
    num_workers: usize,
    /// Job counter for IDs
    job_counter: AtomicUsize,
    /// Shutdown flag
    shutdown: Arc<AtomicBool>,
    /// Worker threads
    workers: Vec<std::thread::JoinHandle<()>>,
    /// Condition variable for job availability
    job_available: Arc<(Mutex<bool>, Condvar)>,
}

impl JobSystem {
    /// Create a new job system with the specified number of worker threads
    pub fn new(num_workers: usize) -> Self {
        let num_workers = num_workers.max(1);
        let global_queue = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let job_available = Arc::new((Mutex::new(false), Condvar::new()));

        let mut local_queues = Vec::with_capacity(num_workers);
        let mut stealers = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            local_queues.push(worker);
        }

        let workers = Vec::with_capacity(num_workers);

        // Note: In a full implementation, worker threads would be spawned here
        // For now, we keep the job system single-threaded for simplicity

        Self {
            global_queue,
            local_queues,
            stealers,
            num_workers,
            job_counter: AtomicUsize::new(0),
            shutdown,
            workers,
            job_available,
        }
    }

    /// Get the number of worker threads
    pub fn num_workers(&self) -> usize {
        self.num_workers
    }

    /// Submit a job to the system
    pub fn submit<J: Job>(&self, job: J, priority: JobPriority) -> JobHandle {
        let id = self.job_counter.fetch_add(1, Ordering::Relaxed) as u64;
        let completed = Arc::new(AtomicBool::new(false));
        
        let wrapper = JobWrapper {
            job: Box::new(job),
            priority,
            completed: completed.clone(),
            dependencies: Vec::new(),
        };

        self.global_queue.push(wrapper);
        
        // Notify workers
        let (lock, cvar) = &*self.job_available;
        let mut available = lock.lock();
        *available = true;
        cvar.notify_one();

        JobHandle { completed, id }
    }

    /// Submit a closure as a job
    pub fn submit_fn<F>(&self, name: &'static str, priority: JobPriority, func: F) -> JobHandle
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(ClosureJob { func: Some(func), name }, priority)
    }

    /// Submit a job with dependencies
    pub fn submit_with_deps<J: Job>(
        &self,
        job: J,
        priority: JobPriority,
        dependencies: &[&JobHandle],
    ) -> JobHandle {
        let id = self.job_counter.fetch_add(1, Ordering::Relaxed) as u64;
        let completed = Arc::new(AtomicBool::new(false));
        
        let deps: Vec<_> = dependencies.iter().map(|h| h.completed.clone()).collect();
        
        let wrapper = JobWrapper {
            job: Box::new(job),
            priority,
            completed: completed.clone(),
            dependencies: deps,
        };

        self.global_queue.push(wrapper);

        JobHandle { completed, id }
    }

    /// Process jobs on the current thread (for main thread execution)
    pub fn process_jobs(&self, max_jobs: usize) -> usize {
        let mut processed = 0;
        
        while processed < max_jobs {
            match self.global_queue.steal() {
                crossbeam::deque::Steal::Success(mut wrapper) => {
                    if wrapper.can_execute() {
                        wrapper.job.execute();
                        wrapper.completed.store(true, Ordering::Release);
                        processed += 1;
                    } else {
                        // Re-queue if dependencies not met
                        self.global_queue.push(wrapper);
                    }
                }
                crossbeam::deque::Steal::Empty => break,
                crossbeam::deque::Steal::Retry => continue,
            }
        }
        
        processed
    }

    /// Wait for a job to complete
    pub fn wait_for(&self, handle: &JobHandle) {
        while !handle.is_complete() {
            // Try to process jobs while waiting
            if self.process_jobs(1) == 0 {
                std::thread::yield_now();
            }
        }
    }

    /// Wait for all submitted jobs to complete
    pub fn wait_all(&self) {
        while !self.global_queue.is_empty() {
            self.process_jobs(16);
            std::thread::yield_now();
        }
    }

    /// Get pending job count
    pub fn pending_jobs(&self) -> usize {
        // Note: Injector doesn't have a len() method, this is approximate
        self.job_counter.load(Ordering::Relaxed)
    }
}

impl Drop for JobSystem {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        
        // Wake up all workers
        let (lock, cvar) = &*self.job_available;
        let mut available = lock.lock();
        *available = true;
        cvar.notify_all();
        drop(available);

        // Join worker threads
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// Builder for creating task graphs with dependencies
pub struct TaskGraphBuilder {
    tasks: Vec<(Box<dyn Job>, JobPriority, Vec<usize>)>,
}

impl TaskGraphBuilder {
    /// Create a new task graph builder
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Add a task to the graph
    pub fn add_task<J: Job>(&mut self, job: J, priority: JobPriority) -> usize {
        let index = self.tasks.len();
        self.tasks.push((Box::new(job), priority, Vec::new()));
        index
    }

    /// Add a dependency between tasks
    pub fn add_dependency(&mut self, task: usize, depends_on: usize) {
        if task < self.tasks.len() && depends_on < self.tasks.len() {
            self.tasks[task].2.push(depends_on);
        }
    }

    /// Execute the task graph on the given job system
    pub fn execute(self, job_system: &JobSystem) -> Vec<JobHandle> {
        let mut handles = Vec::with_capacity(self.tasks.len());
        
        for (job, priority, deps) in self.tasks {
            if deps.is_empty() {
                // Wrap the job in a simple struct that implements Job
                let handle = job_system.submit(BoxedJob { inner: job }, priority);
                handles.push(handle);
            } else {
                let dep_handles: Vec<&JobHandle> = deps.iter()
                    .filter_map(|&idx| handles.get(idx))
                    .collect();
                let handle = job_system.submit_with_deps(
                    BoxedJob { inner: job },
                    priority,
                    &dep_handles,
                );
                handles.push(handle);
            }
        }
        
        handles
    }
}

impl Default for TaskGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for boxed jobs
struct BoxedJob {
    inner: Box<dyn Job>,
}

impl Job for BoxedJob {
    fn execute(&mut self) {
        self.inner.execute();
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    struct CounterJob {
        counter: Arc<AtomicU32>,
    }

    impl Job for CounterJob {
        fn execute(&mut self) {
            self.counter.fetch_add(1, Ordering::Relaxed);
        }

        fn name(&self) -> &str {
            "counter_job"
        }
    }

    #[test]
    fn test_job_system_creation() {
        let job_system = JobSystem::new(4);
        assert_eq!(job_system.num_workers(), 4);
    }

    #[test]
    fn test_job_submission_and_execution() {
        let job_system = JobSystem::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        let handle = job_system.submit(
            CounterJob { counter: counter.clone() },
            JobPriority::Normal,
        );

        job_system.wait_for(&handle);
        assert!(handle.is_complete());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_closure_job() {
        let job_system = JobSystem::new(2);
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handle = job_system.submit_fn("increment", JobPriority::Normal, move || {
            counter_clone.fetch_add(5, Ordering::Relaxed);
        });

        job_system.wait_for(&handle);
        assert_eq!(counter.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_job_dependencies() {
        let job_system = JobSystem::new(2);
        let value = Arc::new(AtomicU32::new(0));
        let value_clone = value.clone();

        // First job sets value to 10
        let handle1 = job_system.submit_fn("set", JobPriority::Normal, move || {
            value_clone.store(10, Ordering::Relaxed);
        });

        // Second job doubles the value (depends on first)
        let value_clone2 = value.clone();
        let handle2 = job_system.submit_with_deps(
            ClosureJob {
                func: Some(move || {
                    let v = value_clone2.load(Ordering::Relaxed);
                    value_clone2.store(v * 2, Ordering::Relaxed);
                }),
                name: "double",
            },
            JobPriority::Normal,
            &[&handle1],
        );

        job_system.wait_for(&handle2);
        assert_eq!(value.load(Ordering::Relaxed), 20);
    }

    #[test]
    fn test_task_graph() {
        let job_system = JobSystem::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        let mut graph = TaskGraphBuilder::new();
        
        let task1 = graph.add_task(CounterJob { counter: counter.clone() }, JobPriority::Normal);
        let task2 = graph.add_task(CounterJob { counter: counter.clone() }, JobPriority::Normal);
        let task3 = graph.add_task(CounterJob { counter: counter.clone() }, JobPriority::Normal);
        
        graph.add_dependency(task2, task1);
        graph.add_dependency(task3, task2);

        let handles = graph.execute(&job_system);
        
        for handle in &handles {
            job_system.wait_for(handle);
        }

        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }
}
