//! Virtual Texturing
//!
//! 4K texture support via virtualized streaming.

/// Virtual texture configuration
#[derive(Debug, Clone)]
pub struct VirtualTextureConfig {
    /// Page/tile size in pixels
    pub page_size: u32,
    /// Physical cache size in pages
    pub cache_size: u32,
    /// Maximum mip level
    pub max_mip: u32,
    /// Feedback buffer resolution divisor
    pub feedback_divisor: u32,
}

impl Default for VirtualTextureConfig {
    fn default() -> Self {
        Self {
            page_size: 128,
            cache_size: 1024,
            max_mip: 12,
            feedback_divisor: 8,
        }
    }
}

/// Texture streaming priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamingPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vt_config() {
        let config = VirtualTextureConfig::default();
        assert_eq!(config.page_size, 128);
    }
}
