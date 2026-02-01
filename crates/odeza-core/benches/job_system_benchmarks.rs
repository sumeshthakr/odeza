//! Job System Benchmarks
//!
//! Performance benchmarks for the job system

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use odeza_core::job::{Job, JobPriority, JobSystem, TaskGraphBuilder};

struct IncrementJob {
    counter: Arc<AtomicU32>,
}

impl Job for IncrementJob {
    fn execute(&mut self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

    fn name(&self) -> &str {
        "increment"
    }
}

struct FibonacciJob {
    n: u32,
    result: u64,
}

impl FibonacciJob {
    fn fib(n: u32) -> u64 {
        if n <= 1 {
            n as u64
        } else {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let c = a + b;
                a = b;
                b = c;
            }
            b
        }
    }
}

impl Job for FibonacciJob {
    fn execute(&mut self) {
        self.result = Self::fib(self.n);
    }

    fn name(&self) -> &str {
        "fibonacci"
    }
}

fn bench_job_submit_wait(c: &mut Criterion) {
    let mut group = c.benchmark_group("job_submit_wait");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let job_system = JobSystem::new(4);
            let counter = Arc::new(AtomicU32::new(0));
            
            b.iter(|| {
                counter.store(0, Ordering::Relaxed);
                
                let handles: Vec<_> = (0..count)
                    .map(|_| {
                        job_system.submit(
                            IncrementJob { counter: counter.clone() },
                            JobPriority::Normal,
                        )
                    })
                    .collect();
                
                for handle in &handles {
                    job_system.wait_for(handle);
                }
                
                black_box(counter.load(Ordering::Relaxed))
            });
        });
    }
    
    group.finish();
}

fn bench_closure_job(c: &mut Criterion) {
    let mut group = c.benchmark_group("closure_job");
    
    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let job_system = JobSystem::new(4);
            let counter = Arc::new(AtomicU32::new(0));
            
            b.iter(|| {
                counter.store(0, Ordering::Relaxed);
                
                let handles: Vec<_> = (0..count)
                    .map(|_| {
                        let c = counter.clone();
                        job_system.submit_fn("increment", JobPriority::Normal, move || {
                            c.fetch_add(1, Ordering::Relaxed);
                        })
                    })
                    .collect();
                
                for handle in &handles {
                    job_system.wait_for(handle);
                }
                
                black_box(counter.load(Ordering::Relaxed))
            });
        });
    }
    
    group.finish();
}

fn bench_task_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_graph");
    
    group.bench_function("linear_chain_10", |b| {
        let job_system = JobSystem::new(4);
        let counter = Arc::new(AtomicU32::new(0));
        
        b.iter(|| {
            counter.store(0, Ordering::Relaxed);
            
            let mut graph = TaskGraphBuilder::new();
            let mut prev_task = None;
            
            for _ in 0..10 {
                let task = graph.add_task(
                    IncrementJob { counter: counter.clone() },
                    JobPriority::Normal,
                );
                if let Some(prev) = prev_task {
                    graph.add_dependency(task, prev);
                }
                prev_task = Some(task);
            }
            
            let handles = graph.execute(&job_system);
            for handle in &handles {
                job_system.wait_for(handle);
            }
            
            black_box(counter.load(Ordering::Relaxed))
        });
    });
    
    group.bench_function("parallel_10", |b| {
        let job_system = JobSystem::new(4);
        let counter = Arc::new(AtomicU32::new(0));
        
        b.iter(|| {
            counter.store(0, Ordering::Relaxed);
            
            let mut graph = TaskGraphBuilder::new();
            
            for _ in 0..10 {
                graph.add_task(
                    IncrementJob { counter: counter.clone() },
                    JobPriority::Normal,
                );
            }
            
            let handles = graph.execute(&job_system);
            for handle in &handles {
                job_system.wait_for(handle);
            }
            
            black_box(counter.load(Ordering::Relaxed))
        });
    });
    
    group.finish();
}

fn bench_compute_jobs(c: &mut Criterion) {
    let mut group = c.benchmark_group("compute_jobs");
    
    group.bench_function("fibonacci_30", |b| {
        let job_system = JobSystem::new(4);
        
        b.iter(|| {
            let handle = job_system.submit(
                FibonacciJob { n: 30, result: 0 },
                JobPriority::High,
            );
            job_system.wait_for(&handle);
            black_box(handle.is_complete())
        });
    });
    
    group.finish();
}

fn bench_priority_scheduling(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_scheduling");
    
    group.bench_function("mixed_priorities_100", |b| {
        let job_system = JobSystem::new(4);
        let counter = Arc::new(AtomicU32::new(0));
        
        b.iter(|| {
            counter.store(0, Ordering::Relaxed);
            
            let priorities = [
                JobPriority::Low,
                JobPriority::Normal,
                JobPriority::High,
                JobPriority::Critical,
            ];
            
            let handles: Vec<_> = (0..100)
                .map(|i| {
                    let priority = priorities[i % priorities.len()];
                    job_system.submit(
                        IncrementJob { counter: counter.clone() },
                        priority,
                    )
                })
                .collect();
            
            for handle in &handles {
                job_system.wait_for(handle);
            }
            
            black_box(counter.load(Ordering::Relaxed))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_job_submit_wait,
    bench_closure_job,
    bench_task_graph,
    bench_compute_jobs,
    bench_priority_scheduling,
);

criterion_main!(benches);
