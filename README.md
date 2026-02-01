# Odeza Engine

A **Rust-first** mobile and handheld PC game engine with modern ray tracing support, 4K asset streaming, and an Unreal-like editor workflow.

## Overview

Odeza is a next-generation game engine designed from the ground up for:
- **Mobile devices** (Android arm64, iOS arm64)
- **Handheld PCs** (Windows x64, Linux x64)
- **Modern rendering** with hybrid ray tracing
- **4K asset support** via virtualized texture streaming
- **Professional tooling** comparable to Unreal Engine

## Features

### Core Runtime
- **ECS**: Archetype-based Entity Component System with data-oriented layout
- **Job System**: Work-stealing thread pool with per-frame task graph
- **Memory Management**: Frame allocators, arenas, and pool allocators
- **Scene Graph**: Hierarchical transforms with prefab support

### Rendering
- **Frame Graph**: Declarative render pass scheduling with automatic resource management
- **Forward+**: Clustered forward rendering for mobile efficiency
- **PBR Materials**: Physically-based rendering with material graph editor
- **Ray Tracing**: Tiered RT effects (reflections, shadows, AO, GI)
- **Volumetrics**: Froxel-based volumetric lighting and fog
- **Virtual Texturing**: 4K texture streaming with strict memory budgets
- **Temporal Upscaling**: TAA/TAAU with dynamic resolution

### Performance Tiers

| Tier | Target | FPS | Features |
|------|--------|-----|----------|
| **M** (Mobile) | Phone baseline | 30-60 | Dynamic resolution, hybrid lighting |
| **H** (High-End) | High-end phone/tablet | 60 | Optional RT reflections/shadows |
| **P** (PC) | Handheld PC | 60 | Full RT effects, higher volumetric quality |

### Platform Support
- **Android** (arm64) - Vulkan backend
- **iOS** (arm64) - Metal backend  
- **Windows** (x64) - Vulkan/DX12 backend
- **Linux** (x64) - Vulkan backend

## Project Structure

```
odeza/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── odeza-core/         # Core runtime (ECS, job system, memory)
│   ├── odeza-platform/     # Platform abstraction layer
│   ├── odeza-renderer/     # Hybrid AAA renderer
│   ├── odeza-assets/       # Asset pipeline and database
│   ├── odeza-physics/      # Physics simulation
│   ├── odeza-audio/        # Audio system
│   ├── odeza-animation/    # Animation system
│   ├── odeza-editor/       # Editor application
│   └── odeza-cli/          # Command-line tools
└── README.md
```

## Getting Started

### Prerequisites

- **Rust** 1.75 or later
- **Cargo** (included with Rust)
- Platform-specific SDKs for target platforms

### Installation

```bash
# Clone the repository
git clone https://github.com/sumeshthakr/odeza.git
cd odeza

# Build all crates
cargo build

# Run tests
cargo test

# Build in release mode
cargo build --release
```

### CLI Usage

```bash
# Create a new project
odeza new my_game

# Build for Android
odeza build -p android -c shipping

# Cook assets
odeza cook -p android

# Package for distribution
odeza package -p android -o ./dist

# Open the editor
odeza editor
```

## Architecture

### Core Crates

| Crate | Description |
|-------|-------------|
| `odeza-core` | ECS, job system, memory management, time, math |
| `odeza-platform` | Window, input, filesystem, threading, audio backends |
| `odeza-renderer` | Frame graph, materials, lighting, RT, volumetrics |
| `odeza-assets` | Asset database, cooking, streaming |
| `odeza-physics` | Rigid bodies, collision, character controller |
| `odeza-audio` | Spatial audio, mixer graph, streaming |
| `odeza-animation` | Clips, state machines, IK, GPU skinning |
| `odeza-editor` | Viewport, outliner, inspector, profiler |
| `odeza-cli` | Build, run, cook, package, deploy commands |

### Performance Engineering

- **Scalability Manager**: Dynamic resolution, per-feature quality tiers
- **Hard Caps**: Max lights per cluster, volumetric lights, RT rays
- **Profiling**: CPU/GPU timers, memory tracking, IO monitoring
- **Content Validation**: Asset lint rules with red/yellow/green warnings

## Development Phases

1. **Phase 1 - Foundations** ✅
   - Platform HAL, ECS, scene graph, job system
   - Minimal Forward+ renderer with PBR
   - Editor skeleton with viewport and outliner
   - Asset database with incremental cooking

2. **Phase 2 - Modern Look** (In Progress)
   - Temporal upscaling + dynamic resolution
   - World streaming with async IO
   - Virtual texturing prototype
   - Animation system + GPU skinning

3. **Phase 3 - Hybrid RT + Volumetrics**
   - RT reflections with denoising
   - Volumetric fog with temporal reprojection
   - Scalability governor

4. **Phase 4 - Production Hardening**
   - Full platform packaging
   - Physics completeness
   - VFX system
   - Crash reporting and CI gates

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Coding Standards

- Use `#[deny(unsafe_code)]` where possible
- No allocations in hot loops
- Profile macros required in performance-critical paths
- Thread-safety via `Send + Sync` bounds

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

- **Sumesh Thakur** - [sumeshthakr](https://github.com/sumeshthakr)

## Acknowledgments

- Thanks to all contributors who help improve this project
- Inspired by modern game engines like Unreal, Unity, and Bevy