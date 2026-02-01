//! # Odeza Animation
//!
//! Animation system for the Odeza engine.
//!
//! ## Features
//! - Animation clips and compression
//! - Blend trees and state machines
//! - IK (2-bone, foot placement)
//! - Retargeting system
//! - GPU skinning
//! - Animation LOD

use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Animation clip
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub sample_rate: f32,
    pub tracks: Vec<AnimationTrack>,
    pub looping: bool,
}

impl Default for AnimationClip {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: 0.0,
            sample_rate: 30.0,
            tracks: Vec::new(),
            looping: true,
        }
    }
}

/// Animation track for a single bone
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub bone_name: String,
    pub positions: Vec<(f32, Vec3)>,
    pub rotations: Vec<(f32, Quat)>,
    pub scales: Vec<(f32, Vec3)>,
}

/// Skeleton definition
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<Bone>,
    pub root_bone: usize,
}

/// Bone in a skeleton
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub local_position: Vec3,
    pub local_rotation: Quat,
    pub local_scale: Vec3,
}

/// Animation state in a state machine
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub name: String,
    pub clip_id: u64,
    pub speed: f32,
    pub transitions: Vec<StateTransition>,
}

/// Transition between animation states
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub target_state: String,
    pub duration: f32,
    pub condition: TransitionCondition,
}

/// Condition for state transitions
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    Always,
    OnComplete,
    Parameter { name: String, op: CompareOp, value: f32 },
    Trigger { name: String },
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Animation state machine
#[derive(Debug, Clone)]
pub struct AnimatorController {
    pub states: Vec<AnimationState>,
    pub default_state: String,
    pub parameters: HashMap<String, AnimatorParameter>,
}

/// Animator parameter types
#[derive(Debug, Clone)]
pub enum AnimatorParameter {
    Float(f32),
    Int(i32),
    Bool(bool),
    Trigger(bool),
}

/// Two-bone IK solver
pub struct TwoBoneIK {
    pub target: Vec3,
    pub pole: Vec3,
    pub weight: f32,
}

impl TwoBoneIK {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            pole: Vec3::Y,
            weight: 1.0,
        }
    }

    pub fn solve(&self, _root: Vec3, _mid: Vec3, _end: Vec3) -> (Quat, Quat) {
        // Placeholder - would implement actual IK solving
        (Quat::IDENTITY, Quat::IDENTITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_clip() {
        let clip = AnimationClip::default();
        assert!(clip.looping);
        assert_eq!(clip.sample_rate, 30.0);
    }

    #[test]
    fn test_two_bone_ik() {
        let ik = TwoBoneIK::new(Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(ik.weight, 1.0);
    }
}
