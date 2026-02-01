//! Frame Graph (Render Graph)
//!
//! Declarative rendering pipeline with automatic resource management.

use ahash::AHashMap;
use smallvec::SmallVec;

/// Unique identifier for a render resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(u32);

/// Unique identifier for a render pass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(u32);

/// Render resource descriptor
#[derive(Debug, Clone)]
pub struct RenderResource {
    /// Resource ID
    pub id: ResourceId,
    /// Resource name (for debugging)
    pub name: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Size (for textures)
    pub size: Option<(u32, u32)>,
    /// Format (for textures)
    pub format: Option<TextureFormat>,
    /// Whether this resource is transient (can be aliased)
    pub transient: bool,
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// 2D texture
    Texture2D,
    /// Depth texture
    DepthTexture,
    /// Buffer
    Buffer,
    /// Render target
    RenderTarget,
}

/// Texture formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    R8,
    R16F,
    R32F,
    RG8,
    RG16F,
    Depth24,
    Depth32F,
    Depth24Stencil8,
}

/// Resource access mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAccess {
    Read,
    Write,
    ReadWrite,
}

/// Render pass definition
#[derive(Debug, Clone)]
pub struct RenderPass {
    /// Pass ID
    pub id: PassId,
    /// Pass name
    pub name: String,
    /// Resources read by this pass
    pub reads: SmallVec<[ResourceId; 8]>,
    /// Resources written by this pass
    pub writes: SmallVec<[ResourceId; 4]>,
    /// Whether this pass uses async compute
    pub async_compute: bool,
    /// Pass queue type
    pub queue: QueueType,
}

/// Queue types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Graphics,
    Compute,
    Transfer,
}

impl RenderPass {
    /// Create a new render pass
    pub fn new(id: PassId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            reads: SmallVec::new(),
            writes: SmallVec::new(),
            async_compute: false,
            queue: QueueType::Graphics,
        }
    }

    /// Add a read dependency
    pub fn read(mut self, resource: ResourceId) -> Self {
        self.reads.push(resource);
        self
    }

    /// Add a write output
    pub fn write(mut self, resource: ResourceId) -> Self {
        self.writes.push(resource);
        self
    }

    /// Mark as async compute
    pub fn async_compute(mut self) -> Self {
        self.async_compute = true;
        self.queue = QueueType::Compute;
        self
    }

    /// Set the queue type
    pub fn on_queue(mut self, queue: QueueType) -> Self {
        self.queue = queue;
        self
    }
}

/// Resource lifetime tracking
#[derive(Debug, Clone)]
struct ResourceLifetime {
    first_use: PassId,
    last_use: PassId,
}

/// Frame graph for render pass scheduling
pub struct FrameGraph {
    /// All resources
    resources: AHashMap<ResourceId, RenderResource>,
    /// All passes
    passes: Vec<RenderPass>,
    /// Pass execution order (after compilation)
    execution_order: Vec<PassId>,
    /// Resource lifetimes
    lifetimes: AHashMap<ResourceId, ResourceLifetime>,
    /// Resource aliasing (transient resources sharing memory)
    aliases: AHashMap<ResourceId, ResourceId>,
    /// Next resource ID
    next_resource_id: u32,
    /// Next pass ID
    next_pass_id: u32,
    /// Whether the graph has been compiled
    compiled: bool,
}

impl FrameGraph {
    /// Create a new frame graph
    pub fn new() -> Self {
        Self {
            resources: AHashMap::new(),
            passes: Vec::new(),
            execution_order: Vec::new(),
            lifetimes: AHashMap::new(),
            aliases: AHashMap::new(),
            next_resource_id: 0,
            next_pass_id: 0,
            compiled: false,
        }
    }

    /// Create a new transient resource
    pub fn create_transient(&mut self, name: impl Into<String>, resource_type: ResourceType) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let resource = RenderResource {
            id,
            name: name.into(),
            resource_type,
            size: None,
            format: None,
            transient: true,
        };

        self.resources.insert(id, resource);
        self.compiled = false;
        id
    }

    /// Create a new texture resource
    pub fn create_texture(
        &mut self,
        name: impl Into<String>,
        width: u32,
        height: u32,
        format: TextureFormat,
        transient: bool,
    ) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let resource = RenderResource {
            id,
            name: name.into(),
            resource_type: ResourceType::Texture2D,
            size: Some((width, height)),
            format: Some(format),
            transient,
        };

        self.resources.insert(id, resource);
        self.compiled = false;
        id
    }

    /// Add a render pass
    pub fn add_pass(&mut self, name: impl Into<String>) -> PassBuilder<'_> {
        let id = PassId(self.next_pass_id);
        self.next_pass_id += 1;
        self.compiled = false;

        PassBuilder {
            graph: self,
            pass: RenderPass::new(id, name),
        }
    }

    /// Compile the frame graph
    pub fn compile(&mut self) {
        if self.compiled {
            return;
        }

        // Calculate resource lifetimes
        self.calculate_lifetimes();

        // Topological sort for execution order
        self.topological_sort();

        // Calculate resource aliasing
        self.calculate_aliasing();

        self.compiled = true;
    }

    fn calculate_lifetimes(&mut self) {
        self.lifetimes.clear();

        for pass in &self.passes {
            // Update first/last use for all resources
            for &resource_id in pass.reads.iter().chain(pass.writes.iter()) {
                self.lifetimes
                    .entry(resource_id)
                    .and_modify(|lifetime| {
                        lifetime.last_use = pass.id;
                    })
                    .or_insert(ResourceLifetime {
                        first_use: pass.id,
                        last_use: pass.id,
                    });
            }
        }
    }

    fn topological_sort(&mut self) {
        // Simple implementation - in production would use proper graph algorithms
        self.execution_order = self.passes.iter().map(|p| p.id).collect();
    }

    fn calculate_aliasing(&mut self) {
        // Simple implementation - would calculate memory aliasing for transient resources
        self.aliases.clear();
    }

    /// Get the execution order
    pub fn execution_order(&self) -> &[PassId] {
        &self.execution_order
    }

    /// Get a pass by ID
    pub fn get_pass(&self, id: PassId) -> Option<&RenderPass> {
        self.passes.iter().find(|p| p.id == id)
    }

    /// Get a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Option<&RenderResource> {
        self.resources.get(&id)
    }

    /// Clear the frame graph for reuse
    pub fn clear(&mut self) {
        self.resources.clear();
        self.passes.clear();
        self.execution_order.clear();
        self.lifetimes.clear();
        self.aliases.clear();
        self.next_resource_id = 0;
        self.next_pass_id = 0;
        self.compiled = false;
    }

    /// Get the number of passes
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Get the number of resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing render passes
pub struct PassBuilder<'a> {
    graph: &'a mut FrameGraph,
    pass: RenderPass,
}

impl<'a> PassBuilder<'a> {
    /// Add a read dependency
    pub fn read(mut self, resource: ResourceId) -> Self {
        self.pass.reads.push(resource);
        self
    }

    /// Add a write output
    pub fn write(mut self, resource: ResourceId) -> Self {
        self.pass.writes.push(resource);
        self
    }

    /// Mark as async compute
    pub fn async_compute(mut self) -> Self {
        self.pass.async_compute = true;
        self.pass.queue = QueueType::Compute;
        self
    }

    /// Set the queue type
    pub fn on_queue(mut self, queue: QueueType) -> Self {
        self.pass.queue = queue;
        self
    }

    /// Build and add the pass to the graph
    pub fn build(self) -> PassId {
        let id = self.pass.id;
        self.graph.passes.push(self.pass);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_graph_creation() {
        let graph = FrameGraph::new();
        assert_eq!(graph.pass_count(), 0);
        assert_eq!(graph.resource_count(), 0);
    }

    #[test]
    fn test_resource_creation() {
        let mut graph = FrameGraph::new();
        
        let color = graph.create_texture("Color", 1920, 1080, TextureFormat::RGBA16F, true);
        let depth = graph.create_texture("Depth", 1920, 1080, TextureFormat::Depth32F, true);
        
        assert_eq!(graph.resource_count(), 2);
        
        let color_res = graph.get_resource(color).unwrap();
        assert_eq!(color_res.name, "Color");
        assert_eq!(color_res.size, Some((1920, 1080)));
    }

    #[test]
    fn test_pass_creation() {
        let mut graph = FrameGraph::new();
        
        let color = graph.create_texture("Color", 1920, 1080, TextureFormat::RGBA16F, true);
        let depth = graph.create_texture("Depth", 1920, 1080, TextureFormat::Depth32F, true);
        
        let pass_id = graph.add_pass("GBuffer")
            .write(color)
            .write(depth)
            .build();
        
        assert_eq!(graph.pass_count(), 1);
        
        let pass = graph.get_pass(pass_id).unwrap();
        assert_eq!(pass.name, "GBuffer");
        assert_eq!(pass.writes.len(), 2);
    }

    #[test]
    fn test_frame_graph_compilation() {
        let mut graph = FrameGraph::new();
        
        let gbuffer = graph.create_transient("GBuffer", ResourceType::RenderTarget);
        let lighting = graph.create_transient("Lighting", ResourceType::RenderTarget);
        
        graph.add_pass("GBuffer Pass")
            .write(gbuffer)
            .build();
        
        graph.add_pass("Lighting Pass")
            .read(gbuffer)
            .write(lighting)
            .build();
        
        graph.compile();
        
        assert_eq!(graph.execution_order().len(), 2);
    }

    #[test]
    fn test_async_compute_pass() {
        let mut graph = FrameGraph::new();
        
        let data = graph.create_transient("Data", ResourceType::Buffer);
        
        let pass_id = graph.add_pass("Compute Pass")
            .write(data)
            .async_compute()
            .build();
        
        let pass = graph.get_pass(pass_id).unwrap();
        assert!(pass.async_compute);
        assert_eq!(pass.queue, QueueType::Compute);
    }
}
