//! Scene Graph
//!
//! Hierarchical scene representation with:
//! - Transform parenting
//! - Prefab support
//! - Editor semantics
//! - Bridge to ECS for runtime performance

use std::collections::HashMap;

use glam::{Mat4, Quat, Vec3};
use smallvec::SmallVec;

use crate::ecs::Entity;

/// Transform component for entities
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Local position
    pub position: Vec3,
    /// Local rotation
    pub rotation: Quat,
    /// Local scale
    pub scale: Vec3,
}

impl Transform {
    /// Identity transform
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Create a new transform with the given position
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a new transform with position and rotation
    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Create a new transform from all components
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Get the local transformation matrix
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Get the forward direction (negative Z in local space)
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Get the right direction (positive X in local space)
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction (positive Y in local space)
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = (target - self.position).normalize();
        if direction.length_squared() > 0.0 {
            self.rotation = Quat::from_rotation_arc(-Vec3::Z, direction);
            // Adjust for up vector
            let right = direction.cross(up).normalize();
            let corrected_up = right.cross(direction).normalize();
            self.rotation = Quat::from_mat4(&Mat4::look_at_rh(Vec3::ZERO, direction, corrected_up)).inverse();
        }
    }

    /// Translate the transform
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    /// Rotate the transform by euler angles (radians)
    pub fn rotate_euler(&mut self, euler: Vec3) {
        self.rotation *= Quat::from_euler(glam::EulerRot::YXZ, euler.y, euler.x, euler.z);
    }

    /// Interpolate between two transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Scene graph node containing hierarchy information
#[derive(Debug, Clone)]
pub struct Node {
    /// Entity this node represents
    pub entity: Entity,
    /// Node name for identification
    pub name: String,
    /// Local transform
    pub local_transform: Transform,
    /// Cached world transform
    world_transform: Transform,
    /// Cached world matrix
    world_matrix: Mat4,
    /// Parent node entity
    pub parent: Option<Entity>,
    /// Child node entities
    pub children: SmallVec<[Entity; 8]>,
    /// Whether the transform is dirty and needs update
    dirty: bool,
    /// Whether this node is visible
    pub visible: bool,
    /// Whether this node is enabled
    pub enabled: bool,
}

impl Node {
    /// Create a new node
    pub fn new(entity: Entity, name: impl Into<String>) -> Self {
        Self {
            entity,
            name: name.into(),
            local_transform: Transform::IDENTITY,
            world_transform: Transform::IDENTITY,
            world_matrix: Mat4::IDENTITY,
            parent: None,
            children: SmallVec::new(),
            dirty: true,
            visible: true,
            enabled: true,
        }
    }

    /// Get the world transform
    pub fn world_transform(&self) -> &Transform {
        &self.world_transform
    }

    /// Get the world matrix
    pub fn world_matrix(&self) -> Mat4 {
        self.world_matrix
    }

    /// Mark the transform as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the transform is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Add a child entity
    pub fn add_child(&mut self, child: Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// Remove a child entity
    pub fn remove_child(&mut self, child: Entity) {
        self.children.retain(|c| *c != child);
    }
}

/// Scene graph managing the hierarchy of entities
pub struct SceneGraph {
    /// All nodes in the scene
    nodes: HashMap<Entity, Node>,
    /// Root entities (no parent)
    roots: Vec<Entity>,
}

impl SceneGraph {
    /// Create a new empty scene graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Add a new node to the scene
    pub fn add_node(&mut self, entity: Entity, name: impl Into<String>) -> &mut Node {
        let node = Node::new(entity, name);
        self.nodes.insert(entity, node);
        self.roots.push(entity);
        self.nodes.get_mut(&entity).unwrap()
    }

    /// Remove a node from the scene
    pub fn remove_node(&mut self, entity: Entity) -> Option<Node> {
        if let Some(node) = self.nodes.remove(&entity) {
            // Remove from parent's children
            if let Some(parent_entity) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_entity) {
                    parent.remove_child(entity);
                }
            }
            
            // Remove from roots if it was a root
            self.roots.retain(|&e| e != entity);
            
            // Orphan children (make them roots)
            for child in &node.children {
                if let Some(child_node) = self.nodes.get_mut(child) {
                    child_node.parent = None;
                    self.roots.push(*child);
                }
            }
            
            Some(node)
        } else {
            None
        }
    }

    /// Get a node by entity
    pub fn get_node(&self, entity: Entity) -> Option<&Node> {
        self.nodes.get(&entity)
    }

    /// Get a mutable node by entity
    pub fn get_node_mut(&mut self, entity: Entity) -> Option<&mut Node> {
        self.nodes.get_mut(&entity)
    }

    /// Set the parent of a node
    pub fn set_parent(&mut self, child: Entity, parent: Option<Entity>) {
        // Remove from old parent
        if let Some(child_node) = self.nodes.get(&child) {
            if let Some(old_parent) = child_node.parent {
                if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
                    old_parent_node.remove_child(child);
                }
            }
        }
        
        // Add to new parent
        if let Some(parent_entity) = parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_entity) {
                parent_node.add_child(child);
            }
            
            // Remove from roots
            self.roots.retain(|&e| e != child);
        } else {
            // Make it a root if no parent
            if !self.roots.contains(&child) {
                self.roots.push(child);
            }
        }
        
        // Update child's parent reference
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = parent;
            child_node.mark_dirty();
        }
    }

    /// Get root entities
    pub fn roots(&self) -> &[Entity] {
        &self.roots
    }

    /// Update world transforms for all dirty nodes
    pub fn update_transforms(&mut self) {
        // Start from roots and propagate down
        let roots: Vec<_> = self.roots.clone();
        for root in roots {
            self.update_transform_recursive(root, Mat4::IDENTITY);
        }
    }

    fn update_transform_recursive(&mut self, entity: Entity, parent_world: Mat4) {
        let (world_matrix, children) = {
            let node = match self.nodes.get_mut(&entity) {
                Some(n) => n,
                None => return,
            };
            
            let world_matrix = parent_world * node.local_transform.local_matrix();
            node.world_matrix = world_matrix;
            
            // Decompose world matrix to get world transform
            let (scale, rotation, translation) = world_matrix.to_scale_rotation_translation();
            node.world_transform = Transform::new(translation, rotation, scale);
            node.dirty = false;
            
            (world_matrix, node.children.clone())
        };
        
        // Recursively update children
        for child in children {
            self.update_transform_recursive(child, world_matrix);
        }
    }

    /// Find a node by name
    pub fn find_by_name(&self, name: &str) -> Option<Entity> {
        self.nodes.values()
            .find(|node| node.name == name)
            .map(|node| node.entity)
    }

    /// Get all descendants of an entity
    pub fn get_descendants(&self, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        self.collect_descendants(entity, &mut descendants);
        descendants
    }

    fn collect_descendants(&self, entity: Entity, result: &mut Vec<Entity>) {
        if let Some(node) = self.nodes.get(&entity) {
            for &child in &node.children {
                result.push(child);
                self.collect_descendants(child, result);
            }
        }
    }

    /// Get the number of nodes in the scene
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the scene is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Clear all nodes from the scene
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.roots.clear();
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Prefab definition for instantiating hierarchies
#[derive(Debug, Clone)]
pub struct Prefab {
    /// Prefab name
    pub name: String,
    /// Root node template
    pub root: PrefabNode,
}

/// Node within a prefab
#[derive(Debug, Clone)]
pub struct PrefabNode {
    /// Node name
    pub name: String,
    /// Local transform
    pub transform: Transform,
    /// Child nodes
    pub children: Vec<PrefabNode>,
    // Components would be added here
}

impl Prefab {
    /// Create a new prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            root: PrefabNode {
                name: String::from("Root"),
                transform: Transform::IDENTITY,
                children: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let t = Transform::IDENTITY;
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_matrix() {
        let t = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        let matrix = t.local_matrix();
        let translation = matrix.w_axis.truncate();
        assert!((translation - Vec3::new(1.0, 2.0, 3.0)).length() < 0.001);
    }

    #[test]
    fn test_transform_directions() {
        let t = Transform::IDENTITY;
        assert!((t.forward() - Vec3::NEG_Z).length() < 0.001);
        assert!((t.right() - Vec3::X).length() < 0.001);
        assert!((t.up() - Vec3::Y).length() < 0.001);
    }

    #[test]
    fn test_transform_lerp() {
        let t1 = Transform::from_position(Vec3::ZERO);
        let t2 = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
        
        let mid = t1.lerp(&t2, 0.5);
        assert!((mid.position.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_scene_graph_add_node() {
        let mut sg = SceneGraph::new();
        let entity = Entity::new(0, 0);
        
        sg.add_node(entity, "TestNode");
        
        assert_eq!(sg.node_count(), 1);
        assert!(!sg.is_empty());
        
        let node = sg.get_node(entity).unwrap();
        assert_eq!(node.name, "TestNode");
    }

    #[test]
    fn test_scene_graph_parenting() {
        let mut sg = SceneGraph::new();
        let parent = Entity::new(0, 0);
        let child = Entity::new(1, 0);
        
        sg.add_node(parent, "Parent");
        sg.add_node(child, "Child");
        
        sg.set_parent(child, Some(parent));
        
        let parent_node = sg.get_node(parent).unwrap();
        assert!(parent_node.children.contains(&child));
        
        let child_node = sg.get_node(child).unwrap();
        assert_eq!(child_node.parent, Some(parent));
        
        // Child should no longer be a root
        assert!(!sg.roots().contains(&child));
    }

    #[test]
    fn test_scene_graph_remove_node() {
        let mut sg = SceneGraph::new();
        let entity = Entity::new(0, 0);
        
        sg.add_node(entity, "TestNode");
        sg.remove_node(entity);
        
        assert!(sg.is_empty());
        assert!(sg.get_node(entity).is_none());
    }

    #[test]
    fn test_scene_graph_update_transforms() {
        let mut sg = SceneGraph::new();
        let parent = Entity::new(0, 0);
        let child = Entity::new(1, 0);
        
        {
            let parent_node = sg.add_node(parent, "Parent");
            parent_node.local_transform.position = Vec3::new(10.0, 0.0, 0.0);
        }
        
        {
            let child_node = sg.add_node(child, "Child");
            child_node.local_transform.position = Vec3::new(5.0, 0.0, 0.0);
        }
        
        sg.set_parent(child, Some(parent));
        sg.update_transforms();
        
        let child_node = sg.get_node(child).unwrap();
        // Child world position should be parent + local = 15
        assert!((child_node.world_transform().position.x - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_find_by_name() {
        let mut sg = SceneGraph::new();
        let entity = Entity::new(42, 0);
        
        sg.add_node(entity, "UniqueNode");
        
        let found = sg.find_by_name("UniqueNode");
        assert_eq!(found, Some(entity));
        
        let not_found = sg.find_by_name("NonExistent");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_get_descendants() {
        let mut sg = SceneGraph::new();
        let root = Entity::new(0, 0);
        let child1 = Entity::new(1, 0);
        let child2 = Entity::new(2, 0);
        let grandchild = Entity::new(3, 0);
        
        sg.add_node(root, "Root");
        sg.add_node(child1, "Child1");
        sg.add_node(child2, "Child2");
        sg.add_node(grandchild, "Grandchild");
        
        sg.set_parent(child1, Some(root));
        sg.set_parent(child2, Some(root));
        sg.set_parent(grandchild, Some(child1));
        
        let descendants = sg.get_descendants(root);
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
        assert!(descendants.contains(&grandchild));
    }
}
