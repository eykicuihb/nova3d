//! nova3d — a CPU-first 3D engine built in deliberate, testable increments.
//!
//! This file is the library crate root and also the `spinning_cube` example
//! binary root (see `Cargo.toml`). Shared `crate::` paths work in both roles.
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
//! - Each module is added behind its own wave; keep crate-root exports explicit.

/// Core vector, matrix, and quaternion math types.
pub mod math;

/// Perspective camera (look-at view + projection).
pub mod camera;

/// Transform hierarchy and scene graph types.
pub mod scene;

/// Software rasterizer producing deterministic framebuffers.
pub mod raster;

/// Mesh → screen pipeline (model, view-projection, rasterize).
pub mod render;

/// Built-in mesh primitives (unit cube, etc.).
pub mod mesh {
    use crate::math::Vec3;

    /// Unit cube centered at the origin (side length 1).
    ///
    /// Returns the 8 corner vertices and 12 triangles with consistent
    /// outward-facing (CCW when viewed from outside) winding.
    pub fn cube() -> (Vec<Vec3>, Vec<[usize; 3]>) {
        // Corner index layout (y up, right-handed):
        //   0: (-0.5, -0.5, -0.5)  1: ( 0.5, -0.5, -0.5)
        //   2: ( 0.5,  0.5, -0.5)  3: (-0.5,  0.5, -0.5)
        //   4: (-0.5, -0.5,  0.5)  5: ( 0.5, -0.5,  0.5)
        //   6: ( 0.5,  0.5,  0.5)  7: (-0.5,  0.5,  0.5)
        let vertices = vec![
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ];

        let triangles = vec![
            // Front  (+z)
            [4, 5, 6],
            [4, 6, 7],
            // Back   (-z)
            [1, 0, 3],
            [1, 3, 2],
            // Right  (+x)
            [5, 1, 2],
            [5, 2, 6],
            // Left   (-x)
            [0, 4, 7],
            [0, 7, 3],
            // Top    (+y)
            [3, 7, 6],
            [3, 6, 2],
            // Bottom (-y)
            [0, 1, 5],
            [0, 5, 4],
        ];

        (vertices, triangles)
    }

    #[cfg(test)]
    mod tests {
        use super::cube;

        #[test]
        fn cube_has_eight_vertices_and_twelve_triangles() {
            let (vertices, triangles) = cube();
            assert_eq!(vertices.len(), 8);
            assert_eq!(triangles.len(), 12);
        }

        #[test]
        fn cube_indices_are_in_range() {
            let (_vertices, triangles) = cube();
            for tri in &triangles {
                for &idx in tri {
                    assert!(idx < 8, "index {idx} is not below 8");
                }
            }
        }

        #[test]
        fn cube_vertices_are_half_unit_coordinates() {
            let (vertices, _triangles) = cube();
            for v in &vertices {
                for coord in [v.x, v.y, v.z] {
                    assert!(
                        coord == 0.5 || coord == -0.5,
                        "coordinate {coord} is not ±0.5"
                    );
                }
            }
        }
    }
}

/// Framebuffer export (PPM) for golden-image tests.
pub mod io;

/// Spinning unit-cube demo: three depth-tested frames as plain-text PPM on stdout.
///
/// Renders [`mesh::cube`] at 0/45/90 degree Y rotations (via [`scene::Transform`] and
/// [`math::Quat::from_axis_angle`]) with [`render::render_mesh_depth`], printing each
/// 64x64 frame through [`io::to_ppm`] after a `# frame N` comment line.
///
/// Used as the `spinning_cube` example entry point; unused when this file is the lib root.
#[allow(dead_code)]
fn main() {
    use crate::camera::Camera;
    use crate::io::to_ppm;
    use crate::math::{Quat, Vec3};
    use crate::mesh;
    use crate::raster::Framebuffer;
    use crate::render::render_mesh_depth;
    use crate::scene::Transform;

    const WIDTH: usize = 64;
    const HEIGHT: usize = 64;
    const COLOR: u32 = 0xFFE0E0E0;

    let camera = Camera::look_at(
        Vec3::new(1.5, 1.2, 1.5),
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        std::f32::consts::FRAC_PI_3,
        WIDTH as f32 / HEIGHT as f32,
        0.1,
        100.0,
    );

    let (vertices, triangles) = mesh::cube();
    let angles_deg = [0.0_f32, 45.0, 90.0];

    for (frame, &degrees) in angles_deg.iter().enumerate() {
        let mut fb = Framebuffer::new(WIDTH, HEIGHT);

        let model = Transform {
            rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), degrees.to_radians()),
            ..Transform::IDENTITY
        }
        .to_mat4();

        render_mesh_depth(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        // Comment line separates frames in the stdout PPM stream.
        println!("# frame {frame}");
        print!("{}", to_ppm(&fb));
    }
}

#[cfg(test)]
mod spinning_cube_tests {
    use crate::camera::Camera;
    use crate::io::to_ppm;
    use crate::math::{Quat, Vec3};
    use crate::mesh;
    use crate::raster::Framebuffer;
    use crate::render::render_mesh_depth;
    use crate::scene::Transform;

    #[test]
    fn spinning_cube_three_frames_emit_ppm_with_frame_comments() {
        const WIDTH: usize = 64;
        const HEIGHT: usize = 64;
        const COLOR: u32 = 0xFFE0E0E0;

        let camera = Camera::look_at(
            Vec3::new(1.5, 1.2, 1.5),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            std::f32::consts::FRAC_PI_3,
            WIDTH as f32 / HEIGHT as f32,
            0.1,
            100.0,
        );

        let (vertices, triangles) = mesh::cube();
        let angles_deg = [0.0_f32, 45.0, 90.0];
        let mut stream = String::new();

        for (frame, &degrees) in angles_deg.iter().enumerate() {
            let mut fb = Framebuffer::new(WIDTH, HEIGHT);
            let model = Transform {
                rotation: Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), degrees.to_radians()),
                ..Transform::IDENTITY
            }
            .to_mat4();

            render_mesh_depth(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

            stream.push_str(&format!("# frame {frame}\n"));
            stream.push_str(&to_ppm(&fb));

            let colored = fb.pixels.iter().filter(|&&p| p == COLOR).count();
            assert!(
                colored > 0,
                "frame {frame} ({degrees}°) should rasterize cube pixels"
            );
            assert_eq!(fb.width, 64);
            assert_eq!(fb.height, 64);
        }

        assert!(stream.contains("# frame 0\n"));
        assert!(stream.contains("# frame 1\n"));
        assert!(stream.contains("# frame 2\n"));
        assert!(stream.contains("P3\n64 64\n255\n"));

        let ppm0 = stream.split("# frame 1\n").next().unwrap();
        let rest = stream.split("# frame 1\n").nth(1).unwrap();
        let ppm1 = rest.split("# frame 2\n").next().unwrap();
        let ppm2 = rest.split("# frame 2\n").nth(1).unwrap();
        assert_ne!(ppm0, ppm1);
        assert_ne!(ppm1, ppm2);
    }
}
