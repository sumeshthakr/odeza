//! # Odeza Physics
//!
//! Physics simulation for the Odeza engine.
//!
//! ## Features
//! - Rigid body dynamics
//! - Collision detection (broadphase + narrowphase)
//! - Raycasts, sweeps, and overlaps
//! - Character controller
//! - Constraints and joints

use glam::Vec3;

/// Collision layer mask
pub type LayerMask = u32;

/// Rigid body type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RigidBodyType {
    #[default]
    Dynamic,
    Kinematic,
    Static,
}

/// Rigid body component
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub layer: LayerMask,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            layer: 1,
        }
    }
}

/// Collider shapes
#[derive(Debug, Clone)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, half_height: f32 },
    Mesh { mesh_id: u64 },
}

/// Collider component
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec3,
    pub is_trigger: bool,
    pub friction: f32,
    pub restitution: f32,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Sphere { radius: 0.5 },
            offset: Vec3::ZERO,
            is_trigger: false,
            friction: 0.5,
            restitution: 0.0,
        }
    }
}

/// Raycast hit result
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub entity_index: u32,
}

/// Character controller
#[derive(Debug, Clone)]
pub struct CharacterController {
    pub height: f32,
    pub radius: f32,
    pub step_height: f32,
    pub slope_limit: f32,
    pub skin_width: f32,
    pub grounded: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            height: 1.8,
            radius: 0.3,
            step_height: 0.3,
            slope_limit: 45.0,
            skin_width: 0.02,
            grounded: false,
        }
    }
}

/// Physics world configuration
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub fixed_timestep: f32,
    pub max_substeps: u32,
    pub broadphase_margin: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            fixed_timestep: 1.0 / 60.0,
            max_substeps: 4,
            broadphase_margin: 0.1,
        }
    }
}

/// Physics world
pub struct PhysicsWorld {
    config: PhysicsConfig,
    bodies: Vec<RigidBody>,
    colliders: Vec<Collider>,
}

impl PhysicsWorld {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            bodies: Vec::new(),
            colliders: Vec::new(),
        }
    }

    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }

    pub fn step(&mut self, _delta_time: f32) {
        // Physics simulation step
    }

    pub fn raycast(&self, _origin: Vec3, _direction: Vec3, _max_distance: f32) -> Option<RaycastHit> {
        None // Placeholder
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(PhysicsConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_default() {
        let rb = RigidBody::default();
        assert_eq!(rb.body_type, RigidBodyType::Dynamic);
        assert_eq!(rb.mass, 1.0);
    }

    #[test]
    fn test_physics_world() {
        let world = PhysicsWorld::new(PhysicsConfig::default());
        assert_eq!(world.config().gravity.y, -9.81);
    }
}
