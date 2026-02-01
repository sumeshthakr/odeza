//! # Odeza Core
//!
//! Core runtime library for the Odeza game engine.
//!
//! This crate provides the foundational systems for a mobile-first AAA game engine:
//! - **ECS**: Archetype-based Entity Component System with data-oriented layout
//! - **Job System**: Work-stealing thread pool with per-frame task graph
//! - **Memory**: Frame allocators, arenas, and pool allocators
//! - **Time**: Variable render step and fixed-step simulation support
//! - **Scene Graph**: Hierarchical transforms, parenting, and prefab support

pub mod ecs;
pub mod job;
pub mod memory;
pub mod time;
pub mod scene;
pub mod math;

pub use ecs::{Entity, World, Component};
pub use job::{JobSystem, Job, JobHandle};
pub use memory::{FrameAllocator, ArenaAllocator, PoolAllocator};
pub use time::{TimeManager, DeltaTime, FixedTimeStep};
pub use scene::{SceneGraph, Transform, Node};

/// Performance tier for mobile and handheld devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerformanceTier {
    /// Tier M: Phone baseline (30-60 fps, dynamic resolution, hybrid lighting)
    Mobile,
    /// Tier H: High-end phone/tablet (60 fps, optional RT, higher volumetric quality)
    HighEnd,
    /// Tier P: Handheld PC (60 fps at 720p-1080p, RT effects available)
    HandheldPC,
}

impl Default for PerformanceTier {
    fn default() -> Self {
        Self::Mobile
    }
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Target performance tier
    pub tier: PerformanceTier,
    /// Target frame rate
    pub target_fps: u32,
    /// Enable dynamic resolution scaling
    pub dynamic_resolution: bool,
    /// Enable temporal upscaling
    pub temporal_upscaling: bool,
    /// Fixed simulation timestep in seconds
    pub fixed_timestep: f64,
    /// Maximum frame time before slowdown
    pub max_frame_time: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            tier: PerformanceTier::Mobile,
            target_fps: 60,
            dynamic_resolution: true,
            temporal_upscaling: true,
            fixed_timestep: 1.0 / 60.0,
            max_frame_time: 1.0 / 30.0,
        }
    }
}

/// Core engine instance
pub struct Engine {
    config: EngineConfig,
    world: World,
    job_system: JobSystem,
    time_manager: TimeManager,
    scene_graph: SceneGraph,
}

impl Engine {
    /// Create a new engine instance with the given configuration
    pub fn new(config: EngineConfig) -> Self {
        let num_threads = rayon::current_num_threads();
        
        Self {
            config,
            world: World::new(),
            job_system: JobSystem::new(num_threads),
            time_manager: TimeManager::new(),
            scene_graph: SceneGraph::new(),
        }
    }

    /// Get the engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get mutable access to the ECS world
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Get the ECS world
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get the job system
    pub fn job_system(&self) -> &JobSystem {
        &self.job_system
    }

    /// Get the time manager
    pub fn time_manager(&self) -> &TimeManager {
        &self.time_manager
    }

    /// Get the scene graph
    pub fn scene_graph(&self) -> &SceneGraph {
        &self.scene_graph
    }

    /// Get mutable access to the scene graph
    pub fn scene_graph_mut(&mut self) -> &mut SceneGraph {
        &mut self.scene_graph
    }

    /// Update the engine for one frame
    pub fn update(&mut self, delta_time: f64) {
        self.time_manager.update(delta_time);
        
        // Run fixed timestep simulation
        let fixed_dt = self.config.fixed_timestep;
        while self.time_manager.should_run_fixed_update(fixed_dt) {
            self.fixed_update(fixed_dt);
            self.time_manager.consume_fixed_update(fixed_dt);
        }
    }

    /// Fixed timestep update for physics and deterministic simulation
    fn fixed_update(&mut self, _delta_time: f64) {
        // Physics and deterministic simulation runs here
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(EngineConfig::default());
        assert_eq!(engine.config().tier, PerformanceTier::Mobile);
        assert_eq!(engine.config().target_fps, 60);
    }

    #[test]
    fn test_performance_tiers() {
        assert_eq!(PerformanceTier::default(), PerformanceTier::Mobile);
    }
}
