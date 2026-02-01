//! ECS Benchmarks
//!
//! Performance benchmarks for the Entity Component System

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use odeza_core::ecs::{Entity, World};

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct Health {
    current: f32,
    max: f32,
}

fn bench_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");
    
    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut world = World::new();
                for _ in 0..count {
                    black_box(world.spawn());
                }
                world
            });
        });
    }
    
    group.finish();
}

fn bench_entity_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_despawn");
    
    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let mut world = World::new();
                    let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        world.despawn(entity);
                    }
                    world
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

fn bench_component_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_add");
    
    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    let mut world = World::new();
                    let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        world.add_component(entity, Position { x: 0.0, y: 0.0, z: 0.0 });
                    }
                    world
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

fn bench_component_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_get");
    
    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut world = World::new();
            let entities: Vec<_> = (0..count)
                .map(|_| {
                    let e = world.spawn();
                    world.add_component(e, Position { x: 1.0, y: 2.0, z: 3.0 });
                    e
                })
                .collect();
            
            b.iter(|| {
                let mut sum = 0.0f32;
                for &entity in &entities {
                    if let Some(pos) = world.get_component::<Position>(entity) {
                        sum += pos.x + pos.y + pos.z;
                    }
                }
                black_box(sum)
            });
        });
    }
    
    group.finish();
}

fn bench_entity_alive_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_alive_check");
    
    let mut world = World::new();
    let entities: Vec<_> = (0..10000).map(|_| world.spawn()).collect();
    
    // Despawn every other entity
    for (i, &entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            world.despawn(entity);
        }
    }
    
    group.bench_function("check_10000", |b| {
        b.iter(|| {
            let mut alive_count = 0;
            for &entity in &entities {
                if world.is_alive(entity) {
                    alive_count += 1;
                }
            }
            black_box(alive_count)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_entity_spawn,
    bench_entity_despawn,
    bench_component_add,
    bench_component_get,
    bench_entity_alive_check,
);

criterion_main!(benches);
