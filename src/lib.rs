//! nova3d — a CPU-first 3D engine built in deliberate, testable increments.
//!
//! Roadmap (each wave lands as a reviewed PR):
//! 1. `math`: vectors, matrices, quaternions — pure, no_std-friendly, fully unit tested
//! 2. `scene`: transform hierarchy and scene graph
//! 3. `raster`: software rasterizer producing deterministic framebuffers
//! 4. `io`: PPM/PNG framebuffer export for golden-image tests
//!
//! Design rules:
//! - No GPU or windowing dependencies; everything must be testable headless in CI.
//! - Determinism first: identical inputs produce byte-identical framebuffers.
//! - Each module is added behind its own wave; keep `lib.rs` exports explicit.

/// Core vector, matrix, and quaternion math types.
pub mod math {
    pub mod mat4 {
        include!("math/mat4.rs");
    }

    pub mod quat {
        include!("math/quat.rs");
    }

    pub mod vec3 {
        include!("math/vec3.rs");
    }
}
