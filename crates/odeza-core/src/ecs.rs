//! Entity Component System (ECS)
//!
//! Archetype-based ECS with data-oriented layout for optimal cache performance.
//! Features:
//! - Stable entity IDs with generation counters
//! - Components stored in SoA (Structure of Arrays) layout where possible
//! - Systems scheduled via job graph
//! - Efficient component queries

use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicU32, Ordering};

use ahash::AHashMap;
use smallvec::SmallVec;

/// Marker trait for components
pub trait Component: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Component for T {}

/// Entity identifier with generation counter for stable IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    /// Entity index
    index: u32,
    /// Generation counter to detect stale references
    generation: u32,
}

impl Entity {
    /// Create a new entity with the given index and generation
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Get the entity index
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get the entity generation
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Create a null entity (invalid reference)
    pub fn null() -> Self {
        Self {
            index: u32::MAX,
            generation: 0,
        }
    }

    /// Check if this is a null entity
    pub fn is_null(&self) -> bool {
        self.index == u32::MAX
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::null()
    }
}

/// Internal entity metadata
#[derive(Debug)]
struct EntityMeta {
    generation: u32,
    alive: bool,
    archetype_id: Option<ArchetypeId>,
}

/// Archetype identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArchetypeId(u32);

/// Component storage trait
trait ComponentStorage: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, index: usize);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Typed component storage using SoA layout
struct TypedStorage<T: Component> {
    data: Vec<T>,
}

impl<T: Component> TypedStorage<T> {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, component: T) {
        self.data.push(component);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
}

impl<T: Component> ComponentStorage for TypedStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, index: usize) {
        if index < self.data.len() {
            self.data.swap_remove(index);
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

/// Archetype containing entities with the same component set
struct Archetype {
    id: ArchetypeId,
    /// Component type IDs in this archetype
    component_types: SmallVec<[TypeId; 8]>,
    /// Component storages indexed by TypeId
    storages: AHashMap<TypeId, Box<dyn ComponentStorage>>,
    /// Entity indices in this archetype
    entities: Vec<Entity>,
}

impl Archetype {
    fn new(id: ArchetypeId, component_types: SmallVec<[TypeId; 8]>) -> Self {
        Self {
            id,
            component_types,
            storages: AHashMap::new(),
            entities: Vec::new(),
        }
    }

    fn contains_type(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }
}

/// The ECS world containing all entities and components
pub struct World {
    /// Entity metadata indexed by entity index
    entities: Vec<EntityMeta>,
    /// Free entity indices for recycling
    free_indices: Vec<u32>,
    /// Next entity index to allocate
    next_index: AtomicU32,
    /// Archetypes
    archetypes: Vec<Archetype>,
    /// Map from component type set to archetype ID
    archetype_map: AHashMap<SmallVec<[TypeId; 8]>, ArchetypeId>,
    /// Next archetype ID
    next_archetype_id: u32,
    /// Entity to archetype index mapping
    entity_archetype_row: AHashMap<Entity, usize>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_indices: Vec::new(),
            next_index: AtomicU32::new(0),
            archetypes: Vec::new(),
            archetype_map: AHashMap::new(),
            next_archetype_id: 0,
            entity_archetype_row: AHashMap::new(),
        }
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let (index, generation) = if let Some(index) = self.free_indices.pop() {
            let meta = &mut self.entities[index as usize];
            meta.generation += 1;
            meta.alive = true;
            meta.archetype_id = None;
            (index, meta.generation)
        } else {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed);
            self.entities.push(EntityMeta {
                generation: 0,
                alive: true,
                archetype_id: None,
            });
            (index, 0)
        };

        Entity::new(index, generation)
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let meta = &mut self.entities[entity.index() as usize];
        meta.alive = false;
        
        // Remove from archetype if present
        if let Some(_archetype_id) = meta.archetype_id.take() {
            self.entity_archetype_row.remove(&entity);
        }

        self.free_indices.push(entity.index());
        true
    }

    /// Check if an entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.index() as usize)
            .is_some_and(|meta| meta.alive && meta.generation == entity.generation())
    }

    /// Get the number of alive entities
    pub fn entity_count(&self) -> usize {
        self.entities.iter().filter(|m| m.alive).count()
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let type_id = TypeId::of::<T>();
        
        // Get or create archetype with this component
        let archetype_id = self.get_or_create_archetype_for_component::<T>(entity);
        
        // Get the archetype
        let archetype = &mut self.archetypes[archetype_id.0 as usize];
        
        // Ensure storage exists for this component type
        if !archetype.storages.contains_key(&type_id) {
            archetype.storages.insert(type_id, Box::new(TypedStorage::<T>::new()));
        }

        // Add component to storage
        let storage = archetype.storages.get_mut(&type_id).unwrap();
        let typed_storage = storage.as_any_mut().downcast_mut::<TypedStorage<T>>().unwrap();
        typed_storage.push(component);

        // Track entity in archetype
        let row = archetype.entities.len();
        archetype.entities.push(entity);
        self.entity_archetype_row.insert(entity, row);

        // Update entity metadata
        self.entities[entity.index() as usize].archetype_id = Some(archetype_id);

        true
    }

    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.is_alive(entity) {
            return None;
        }

        let archetype_id = self.entities[entity.index() as usize].archetype_id?;
        let row = *self.entity_archetype_row.get(&entity)?;
        let archetype = &self.archetypes[archetype_id.0 as usize];
        
        let type_id = TypeId::of::<T>();
        let storage = archetype.storages.get(&type_id)?;
        let typed_storage = storage.as_any().downcast_ref::<TypedStorage<T>>()?;
        
        typed_storage.get(row)
    }

    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.is_alive(entity) {
            return None;
        }

        let archetype_id = self.entities[entity.index() as usize].archetype_id?;
        let row = *self.entity_archetype_row.get(&entity)?;
        let archetype = &mut self.archetypes[archetype_id.0 as usize];
        
        let type_id = TypeId::of::<T>();
        let storage = archetype.storages.get_mut(&type_id)?;
        let typed_storage = storage.as_any_mut().downcast_mut::<TypedStorage<T>>()?;
        
        typed_storage.get_mut(row)
    }

    /// Check if an entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let Some(archetype_id) = self.entities[entity.index() as usize].archetype_id else {
            return false;
        };

        let archetype = &self.archetypes[archetype_id.0 as usize];
        archetype.contains_type(TypeId::of::<T>())
    }

    fn get_or_create_archetype_for_component<T: Component>(&mut self, _entity: Entity) -> ArchetypeId {
        let type_id = TypeId::of::<T>();
        let types: SmallVec<[TypeId; 8]> = smallvec::smallvec![type_id];

        if let Some(&id) = self.archetype_map.get(&types) {
            return id;
        }

        let id = ArchetypeId(self.next_archetype_id);
        self.next_archetype_id += 1;

        let archetype = Archetype::new(id, types.clone());
        self.archetypes.push(archetype);
        self.archetype_map.insert(types, id);

        id
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
        z: f32,
    }

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();
        let entity = world.spawn();
        
        assert!(!entity.is_null());
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_entity_despawn() {
        let mut world = World::new();
        let entity = world.spawn();
        
        assert!(world.despawn(entity));
        assert!(!world.is_alive(entity));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_entity_recycling() {
        let mut world = World::new();
        let entity1 = world.spawn();
        world.despawn(entity1);
        let entity2 = world.spawn();
        
        // Same index, different generation
        assert_eq!(entity1.index(), entity2.index());
        assert_ne!(entity1.generation(), entity2.generation());
        
        // Old entity reference is no longer valid
        assert!(!world.is_alive(entity1));
        assert!(world.is_alive(entity2));
    }

    #[test]
    fn test_component_add_get() {
        let mut world = World::new();
        let entity = world.spawn();
        
        let pos = Position { x: 1.0, y: 2.0, z: 3.0 };
        assert!(world.add_component(entity, pos.clone()));
        
        let retrieved = world.get_component::<Position>(entity);
        assert_eq!(retrieved, Some(&pos));
    }

    #[test]
    fn test_component_mutation() {
        let mut world = World::new();
        let entity = world.spawn();
        
        world.add_component(entity, Position { x: 0.0, y: 0.0, z: 0.0 });
        
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            pos.x = 10.0;
        }
        
        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
    }

    #[test]
    fn test_has_component() {
        let mut world = World::new();
        let entity = world.spawn();
        
        assert!(!world.has_component::<Position>(entity));
        
        world.add_component(entity, Position { x: 0.0, y: 0.0, z: 0.0 });
        
        assert!(world.has_component::<Position>(entity));
        assert!(!world.has_component::<Velocity>(entity));
    }
}
