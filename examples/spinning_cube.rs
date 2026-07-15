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
use nova3d::render::{render_mesh_lit, render_mesh_textured, TextureSampler};
use nova3d::scene::Transform;

struct CheckerboardSampler;

impl TextureSampler for CheckerboardSampler {
    fn sample(&self, u: f32, v: f32) -> u32 {
        let tile_u = usize::from(u >= 0.5);
        let tile_v = usize::from(v >= 0.5);
        if (tile_u + tile_v) % 2 == 0 {
            0x00F0F0F0
        } else {
            0x00303030
        }
    }
}

fn textured_cube() -> (Vec<Vec3>, Vec<[f32; 2]>, Vec<[usize; 3]>) {
    const FACE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    const FACES: [[usize; 4]; 6] = [
        [4, 5, 6, 7],
        [1, 0, 3, 2],
        [5, 1, 2, 6],
        [0, 4, 7, 3],
        [3, 7, 6, 2],
        [0, 1, 5, 4],
    ];

    let (cube_vertices, _) = mesh::cube();
    let mut vertices = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut triangles = Vec::with_capacity(12);

    for face in FACES {
        let base = vertices.len();
        for (corner, &vertex_index) in face.iter().enumerate() {
            vertices.push(cube_vertices[vertex_index]);
            uvs.push(FACE_UVS[corner]);
        }
        triangles.push([base, base + 1, base + 2]);
        triangles.push([base, base + 2, base + 3]);
    }

    (vertices, uvs, triangles)
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

    let (textured_vertices, textured_uvs, textured_triangles) = textured_cube();
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    render_mesh_textured(
        &mut fb,
        &camera,
        &textured_vertices,
        &textured_uvs,
        &textured_triangles,
        &Transform::IDENTITY.to_mat4(),
        &CheckerboardSampler,
    );

    println!("# frame {}", angles_deg.len());
    print!("{}", to_ppm(&fb));
}
