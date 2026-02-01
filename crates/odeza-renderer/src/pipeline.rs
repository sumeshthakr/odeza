//! Render Pipeline
//!
//! GPU pipeline management.

/// Pipeline state
#[derive(Debug, Clone)]
pub struct PipelineState {
    /// Blend state
    pub blend: BlendState,
    /// Depth state
    pub depth: DepthState,
    /// Rasterizer state
    pub rasterizer: RasterizerState,
}

/// Blend state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendState {
    #[default]
    Opaque,
    AlphaBlend,
    Additive,
    Multiply,
}

/// Depth state
#[derive(Debug, Clone)]
pub struct DepthState {
    pub test: bool,
    pub write: bool,
    pub compare: CompareFunction,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            test: true,
            write: true,
            compare: CompareFunction::Less,
        }
    }
}

/// Compare function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareFunction {
    Never,
    #[default]
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Rasterizer state
#[derive(Debug, Clone)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub fill_mode: FillMode,
    pub front_face: FrontFace,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            fill_mode: FillMode::Solid,
            front_face: FrontFace::CounterClockwise,
        }
    }
}

/// Cull mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CullMode {
    None,
    Front,
    #[default]
    Back,
}

/// Fill mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillMode {
    #[default]
    Solid,
    Wireframe,
}

/// Front face winding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FrontFace {
    Clockwise,
    #[default]
    CounterClockwise,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            blend: BlendState::default(),
            depth: DepthState::default(),
            rasterizer: RasterizerState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_default() {
        let state = PipelineState::default();
        assert_eq!(state.blend, BlendState::Opaque);
        assert!(state.depth.test);
    }
}
