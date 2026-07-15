//! Spinning unit-cube demo: three Lambert-lit frames as plain-text PPM on stdout.
//!
//! Renders [`nova3d::mesh::cube`] at 0/45/90 degree Y rotations via
//! [`nova3d::scene::Transform`] and [`nova3d::math::Quat::from_axis_angle`] with
//! [`nova3d::render::render_mesh_lit`], printing each 64x64 frame through
//! [`nova3d::io::to_ppm`] after a `# frame N` comment line.
//!
//! Note: Cargo.toml also registers `[[example]] name = "spinning_cube"` with
//! `path = "src/mesh.rs"` so the example remains runnable when this file is the
//! canonical auto-discovered target or when the shared crate-root entry point is used.

use nova3d::camera::Camera;
use nova3d::io::to_ppm;
use nova3d::math::{Quat, Vec3};
use nova3d::mesh;
use nova3d::raster::Framebuffer;
use nova3d::render::{render_mesh_textured, render_mesh_lit, TextureSampler};
use nova3d::scene::Transform;

struct Checkerboard;

impl TextureSampler for Checkerboard {
    fn sample(&self, u: f32, v: f32) -> u32 {
        let x = (u.clamp(0.0, 1.0) * 2.0).floor().min(1.0) as usize;
        let y = (v.clamp(0.0, 1.0) * 2.0).floor().min(1.0) as usize;
        if (x + y) % 2 == 0 {
            0xFFFFFFFF
        } else {
            0xFF202020
        }
    }
}

fn main() {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 64;
    const COLOR: u32 = 0xFFE0E0E0;

    let light_dir = Vec3::new(-1.0, -1.0, -0.5).normalize();

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

        render_mesh_lit(
            &mut fb,
            &camera,
            &vertices,
            &triangles,
            &model,
            COLOR,
            light_dir,
        );

        // Comment line separates frames in the stdout PPM stream.
        println!("# frame {frame}");
        print!("{}", to_ppm(&fb));
    }

    const FACE_CORNERS: [[usize; 4]; 6] = [
        [4, 5, 6, 7],
        [1, 0, 3, 2],
        [5, 1, 2, 6],
        [0, 4, 7, 3],
        [3, 7, 6, 2],
        [0, 1, 5, 4],
    ];
    const FACE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let textured_vertices: Vec<Vec3> = FACE_CORNERS
        .iter()
        .flat_map(|face| face.iter().map(|&index| vertices[index]))
        .collect();
    let textured_uvs: Vec<[f32; 2]> = (0..FACE_CORNERS.len())
        .flat_map(|_| FACE_UVS)
        .collect();
    let textured_triangles: Vec<[usize; 3]> = (0..FACE_CORNERS.len())
        .flat_map(|face| {
            let base = face * 4;
            [[base, base + 1, base + 2], [base, base + 2, base + 3]]
        })
        .collect();

    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    let model = Transform::IDENTITY.to_mat4();
    render_mesh_textured(
        &mut fb,
        &camera,
        &textured_vertices,
        &textured_uvs,
        &textured_triangles,
        &model,
        &Checkerboard,
    );

    println!("# frame 3");
    print!("{}", to_ppm(&fb));
}
