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

    pub fn solve(&self, root: Vec3, mid: Vec3, end: Vec3) -> (Quat, Quat) {
        let weight = self.weight.clamp(0.0, 1.0);

        let upper = mid - root;
        let lower = end - mid;
        let target = self.target - root;

        let upper_len = upper.length();
        let lower_len = lower.length();
        let target_len = target.length().max(1e-4).min(upper_len + lower_len);

        let current_dir = (end - root).normalize_or_zero();
        let target_dir = target.normalize_or_zero();

        // Root rotation to align current direction to the target direction
        let root_rot_full = Quat::from_rotation_arc(current_dir, target_dir);

        // Compute elbow angle using the law of cosines
        let cos_angle = ((upper_len * upper_len + lower_len * lower_len - target_len * target_len)
            / (2.0 * upper_len * lower_len))
            .clamp(-1.0, 1.0);
        let elbow_angle = std::f32::consts::PI - cos_angle.acos();

        // Axis for elbow rotation based on pole vector
        let axis = target_dir.cross(self.pole).try_normalize().unwrap_or(Vec3::Z);
        // If the target is already aligned with the current chain direction, avoid rotating
        let mid_rot_full = if elbow_angle.abs() < 1e-5 {
            Quat::IDENTITY
        } else {
            Quat::from_axis_angle(axis, elbow_angle)
        };

        // Apply weighting
        let root_rot = Quat::IDENTITY.slerp(root_rot_full, weight);
        let mid_rot = Quat::IDENTITY.slerp(mid_rot_full, weight);

        (root_rot, mid_rot)
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

    #[test]
    fn test_two_bone_ik_identity_when_aligned() {
        let ik = TwoBoneIK::new(Vec3::new(0.0, 2.0, 0.0));
        let (root_rot, mid_rot) = ik.solve(Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 2.0, 0.0));
        assert!(root_rot.length() - 1.0 < 1e-5);
        assert!(mid_rot.length() - 1.0 < 1e-5);
        assert!(root_rot.angle_between(Quat::IDENTITY) < 1e-3);
        assert!(mid_rot.angle_between(Quat::IDENTITY) < 1e-3);
    }

    #[test]
    fn test_two_bone_ik_rotates_toward_target() {
        let ik = TwoBoneIK::new(Vec3::new(1.0, 1.0, 0.0));
        let (root_rot, _) = ik.solve(Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 2.0, 0.0));
        let rotated_dir = root_rot * Vec3::Y;
        let target_dir = Vec3::new(1.0, 1.0, 0.0).normalize();
        assert!(rotated_dir.normalize().dot(target_dir) > 0.9);
    }
}
