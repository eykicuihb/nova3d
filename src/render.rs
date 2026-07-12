use crate::camera::Camera;
use crate::math::{Mat4, Vec3};
use crate::raster::{fill_triangle, Framebuffer, Vec2i};

/// Clip-space homogeneous position after model and view-projection transforms.
/// Only x/y/w are needed for 2D rasterization (no depth buffer yet).
#[derive(Clone, Copy, Debug)]
struct ClipVertex {
    x: f32,
    y: f32,
    w: f32,
}

/// Renders a triangle mesh into `fb` using a perspective camera.
///
/// Each vertex is transformed by `model`, then by `camera.view_projection()`.
/// Triangles with any vertex where clip `w <= camera.near` are rejected.
/// Surviving vertices are perspective-divided and mapped to screen space, then
/// filled via [`fill_triangle`].
pub fn render_mesh(
    fb: &mut Framebuffer,
    camera: &Camera,
    vertices: &[Vec3],
    triangles: &[[usize; 3]],
    model: &Mat4,
    color: u32,
) {
    let view_proj = camera.view_projection();
    let mvp = view_proj.multiply(*model);

    let width = fb.width as f32;
    let height = fb.height as f32;
    let near = camera.near;

    let clip: Vec<ClipVertex> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_homogeneous(mvp, v))
        .collect();

    for tri in triangles {
        let v0 = clip[tri[0]];
        let v1 = clip[tri[1]];
        let v2 = clip[tri[2]];

        // Simple near-plane rejection: drop the whole triangle if any w is too small.
        if v0.w <= near || v1.w <= near || v2.w <= near {
            continue;
        }

        let s0 = clip_to_screen(v0, width, height);
        let s1 = clip_to_screen(v1, width, height);
        let s2 = clip_to_screen(v2, width, height);

        fill_triangle(fb, s0, s1, s2, color);
    }
}

/// Column-major Mat4 × (x, y, z, 1) → clip-space (x, y, z, w).
fn transform_point_homogeneous(matrix: Mat4, point: Vec3) -> ClipVertex {
    let d = &matrix.data;
    ClipVertex {
        x: d[0] * point.x + d[4] * point.y + d[8] * point.z + d[12],
        y: d[1] * point.x + d[5] * point.y + d[9] * point.z + d[13],
        w: d[3] * point.x + d[7] * point.y + d[11] * point.z + d[15],
    }
}

fn clip_to_screen(v: ClipVertex, width: f32, height: f32) -> Vec2i {
    let inv_w = 1.0 / v.w;
    let ndc_x = v.x * inv_w;
    let ndc_y = v.y * inv_w;
    let x_screen = (ndc_x + 1.0) * 0.5 * width;
    let y_screen = (1.0 - ndc_y) * 0.5 * height;
    Vec2i {
        x: x_screen.round() as i32,
        y: y_screen.round() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    const COLOR: u32 = 0xFFFF0000;
    const BG: u32 = 0xFF000000;

    fn sample_camera(aspect: f32) -> Camera {
        Camera::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            aspect,
            0.1,
            100.0,
        )
    }

    /// Unit triangle in the XY plane at z = 0 (in front of the sample camera).
    fn front_triangle() -> (Vec<Vec3>, Vec<[usize; 3]>) {
        let vertices = vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let triangles = vec![[0, 1, 2]];
        (vertices, triangles)
    }

    fn count_colored(fb: &Framebuffer, color: u32) -> usize {
        fb.pixels.iter().filter(|&&p| p == color).count()
    }

    #[test]
    fn triangle_in_front_of_camera_rasterizes_nonzero_pixels() {
        let mut fb = Framebuffer::new(32, 32);
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();

        render_mesh(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        assert!(
            count_colored(&fb, COLOR) > 0,
            "expected at least one colored pixel for a front-facing triangle"
        );
        // Background should still be present somewhere (triangle does not cover all).
        assert!(fb.pixels.contains(&BG));
    }

    #[test]
    fn triangle_behind_camera_rasterizes_zero_pixels() {
        let mut fb = Framebuffer::new(32, 32);
        let camera = sample_camera(1.0);
        // Same local triangle, but translated behind the camera (z = 10 > eye z = 5).
        let (vertices, triangles) = front_triangle();
        let model = Mat4::from_translation(Vec3::new(0.0, 0.0, 10.0));

        render_mesh(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        assert_eq!(
            count_colored(&fb, COLOR),
            0,
            "triangle behind the camera should be near-rejected"
        );
        assert!(fb.pixels.iter().all(|&p| p == BG));
    }

    #[test]
    fn identical_scene_renders_byte_identical_checksums() {
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();

        let mut fb_a = Framebuffer::new(32, 32);
        let mut fb_b = Framebuffer::new(32, 32);

        render_mesh(&mut fb_a, &camera, &vertices, &triangles, &model, COLOR);
        render_mesh(&mut fb_b, &camera, &vertices, &triangles, &model, COLOR);

        assert_eq!(fb_a.checksum(), fb_b.checksum());
        assert_eq!(fb_a.pixels, fb_b.pixels);
    }
}
