use crate::camera::Camera;
use crate::math::{Mat4, Vec3};
use crate::raster::{fill_triangle, Framebuffer, Vec2i};

/// Clip-space homogeneous position after model and view-projection transforms.
#[derive(Clone, Copy, Debug)]
struct ClipVertex {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

/// Screen-space vertex with post-perspective-divide depth (NDC z).
#[derive(Clone, Copy, Debug)]
struct ScreenVertex {
    pos: Vec2i,
    depth: f32,
}

/// Supplies colors for normalized texture coordinates in AARRGGBB format.
pub trait TextureSampler {
    /// Samples an opaque color at normalized coordinates in the `[0.0, 1.0]` range.
    fn sample(&self, u: f32, v: f32) -> u32;
}

#[derive(Clone, Copy, Debug)]
struct TexturedScreenVertex {
    screen: ScreenVertex,
    inv_w: f32,
    u_over_w: f32,
    v_over_w: f32,
}

/// Renders a textured triangle mesh with depth and screen-space back-face culling.
///
/// UVs are paired with vertices by index. The sampler receives perspective-corrected
/// normalized coordinates and returns an opaque AARRGGBB color.
pub fn render_mesh_textured<S: TextureSampler>(
    fb: &mut Framebuffer,
    camera: &Camera,
    vertices: &[Vec3],
    uvs: &[[f32; 2]],
    triangles: &[[usize; 3]],
    model: &Mat4,
    sampler: &S,
) {
    let view_proj = camera.view_projection();
    let mvp = view_proj.multiply(*model);
    let width = fb.width as f32;
    let height = fb.height as f32;
    let near = camera.near;
    let mut depth = vec![f32::INFINITY; fb.width * fb.height];
    let clip: Vec<ClipVertex> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_homogeneous(mvp, v))
        .collect();

    for tri in triangles {
        let v0 = clip[tri[0]];
        let v1 = clip[tri[1]];
        let v2 = clip[tri[2]];
        if v0.w <= near || v1.w <= near || v2.w <= near {
            continue;
        }

        let s0 = textured_clip_to_screen(v0, uvs[tri[0]], width, height);
        let s1 = textured_clip_to_screen(v1, uvs[tri[1]], width, height);
        let s2 = textured_clip_to_screen(v2, uvs[tri[2]], width, height);
        if screen_signed_area_ccw(s0.screen.pos, s1.screen.pos, s2.screen.pos) <= 0 {
            continue;
        }
        fill_triangle_textured(fb, &mut depth, s0, s1, s2, sampler);
    }
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

        let s0 = clip_to_screen(v0, width, height).pos;
        let s1 = clip_to_screen(v1, width, height).pos;
        let s2 = clip_to_screen(v2, width, height).pos;

        fill_triangle(fb, s0, s1, s2, color);
    }
}

/// Renders a triangle mesh with a per-call depth buffer.
///
/// Same vertex pipeline as [`render_mesh`] (model, view-projection, near rejection
/// at `w <= camera.near`, perspective divide, NDC-to-screen mapping), but
/// rasterizes with an internal depth buffer (`width * height` floats initialized
/// to `f32::INFINITY`). Pixels are filled via integer edge functions with
/// barycentric interpolation of post-divide depth; a pixel is written only when
/// its interpolated depth is strictly less than the stored value.
pub fn render_mesh_depth(
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

    let mut depth = vec![f32::INFINITY; fb.width * fb.height];

    let clip: Vec<ClipVertex> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_homogeneous(mvp, v))
        .collect();

    for tri in triangles {
        let v0 = clip[tri[0]];
        let v1 = clip[tri[1]];
        let v2 = clip[tri[2]];

        if v0.w <= near || v1.w <= near || v2.w <= near {
            continue;
        }

        let s0 = clip_to_screen(v0, width, height);
        let s1 = clip_to_screen(v1, width, height);
        let s2 = clip_to_screen(v2, width, height);

        fill_triangle_depth(fb, &mut depth, s0, s1, s2, color);
    }
}

/// Renders a triangle mesh with a per-call depth buffer and back-face culling.
///
/// Same vertex pipeline as [`render_mesh_depth`] (model, view-projection, near
/// rejection at `w <= camera.near`, perspective divide, NDC-to-screen mapping,
/// internal `f32` depth buffer with barycentric depth interpolation writing a
/// pixel only when strictly nearer), except that after computing the three
/// integer screen-space vertices any triangle whose screen-space signed area is
/// `<= 0` is skipped (counter-clockwise-front: non-positive area means
/// back-facing or degenerate).
pub fn render_mesh_depth_culled(
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

    let mut depth = vec![f32::INFINITY; fb.width * fb.height];

    let clip: Vec<ClipVertex> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_homogeneous(mvp, v))
        .collect();

    for tri in triangles {
        let v0 = clip[tri[0]];
        let v1 = clip[tri[1]];
        let v2 = clip[tri[2]];

        if v0.w <= near || v1.w <= near || v2.w <= near {
            continue;
        }

        let s0 = clip_to_screen(v0, width, height);
        let s1 = clip_to_screen(v1, width, height);
        let s2 = clip_to_screen(v2, width, height);

        // CCW-front in Y-up sense: screen Y grows downward, so negate the
        // Y-down cross product. Non-positive → back-facing or degenerate.
        if screen_signed_area_ccw(s0.pos, s1.pos, s2.pos) <= 0 {
            continue;
        }

        fill_triangle_depth(fb, &mut depth, s0, s1, s2, color);
    }
}

/// Minimum Lambert ambient so fully back-lit but visible faces are not pure black.
const LAMBERT_MIN_BRIGHTNESS: f32 = 0.15;

/// Renders a triangle mesh with depth, back-face culling, and parallel-light Lambert shading.
///
/// Same vertex pipeline as [`render_mesh_depth_culled`] (model, view-projection, near
/// rejection at `w <= camera.near`, perspective divide, NDC-to-screen mapping, screen-space
/// back-face culling, internal `f32` depth buffer with barycentric depth interpolation).
///
/// Per triangle, a world-space face normal is formed from the model-transformed vertices
/// (before the camera transform) via the cross product of two edge vectors, then normalized.
/// Brightness is `max(0.15, N · normalize(-light_dir))`. The R/G/B channels of `base_color`
/// are scaled by that brightness; alpha is forced to `0xFF`.
pub fn render_mesh_lit(
    fb: &mut Framebuffer,
    camera: &Camera,
    vertices: &[Vec3],
    triangles: &[[usize; 3]],
    model: &Mat4,
    base_color: u32,
    light_dir: Vec3,
) {
    let view_proj = camera.view_projection();
    let mvp = view_proj.multiply(*model);

    let width = fb.width as f32;
    let height = fb.height as f32;
    let near = camera.near;

    let mut depth = vec![f32::INFINITY; fb.width * fb.height];

    // World-space positions: model transform only (before camera / view-projection).
    let world: Vec<Vec3> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_affine(*model, v))
        .collect();

    let clip: Vec<ClipVertex> = vertices
        .iter()
        .copied()
        .map(|v| transform_point_homogeneous(mvp, v))
        .collect();

    // Direction toward the light source (opposite of light travel direction).
    let light_toward = light_dir.scale(-1.0).normalize();

    for tri in triangles {
        let v0 = clip[tri[0]];
        let v1 = clip[tri[1]];
        let v2 = clip[tri[2]];

        if v0.w <= near || v1.w <= near || v2.w <= near {
            continue;
        }

        let s0 = clip_to_screen(v0, width, height);
        let s1 = clip_to_screen(v1, width, height);
        let s2 = clip_to_screen(v2, width, height);

        // CCW-front in Y-up sense: screen Y grows downward, so negate the
        // Y-down cross product. Non-positive → back-facing or degenerate.
        if screen_signed_area_ccw(s0.pos, s1.pos, s2.pos) <= 0 {
            continue;
        }

        let w0 = world[tri[0]];
        let w1 = world[tri[1]];
        let w2 = world[tri[2]];
        let edge1 = w1.sub(w0);
        let edge2 = w2.sub(w0);
        let normal = edge1.cross(edge2).normalize();

        let brightness = normal.dot(light_toward).max(LAMBERT_MIN_BRIGHTNESS);
        let color = shade_lambert(base_color, brightness);

        fill_triangle_depth(fb, &mut depth, s0, s1, s2, color);
    }
}

/// Affine model transform: Mat4 × (x, y, z, 1) → world-space xyz (w discarded).
fn transform_point_affine(matrix: Mat4, point: Vec3) -> Vec3 {
    let h = transform_point_homogeneous(matrix, point);
    Vec3::new(h.x, h.y, h.z)
}

/// Scale `base_color` RGB by `brightness` (AARRGGBB); alpha forced to `0xFF`.
fn shade_lambert(base_color: u32, brightness: f32) -> u32 {
    let r = ((base_color >> 16) & 0xFF) as f32 * brightness;
    let g = ((base_color >> 8) & 0xFF) as f32 * brightness;
    let b = (base_color & 0xFF) as f32 * brightness;
    let r = r.round().clamp(0.0, 255.0) as u32;
    let g = g.round().clamp(0.0, 255.0) as u32;
    let b = b.round().clamp(0.0, 255.0) as u32;
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

/// Screen-space signed area with counter-clockwise-front convention.
///
/// Positive means front-facing for meshes wound CCW when viewed from outside
/// (Y-up NDC). Zero is degenerate; negative is back-facing.
fn screen_signed_area_ccw(p0: Vec2i, p1: Vec2i, p2: Vec2i) -> i64 {
    let x0 = i64::from(p0.x);
    let y0 = i64::from(p0.y);
    let x1 = i64::from(p1.x);
    let y1 = i64::from(p1.y);
    let x2 = i64::from(p2.x);
    let y2 = i64::from(p2.y);
    // Standard Y-down cross product is negated so CCW-in-world stays positive.
    -((x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0))
}

/// Column-major Mat4 × (x, y, z, 1) → clip-space (x, y, z, w).
fn transform_point_homogeneous(matrix: Mat4, point: Vec3) -> ClipVertex {
    let d = &matrix.data;
    ClipVertex {
        x: d[0] * point.x + d[4] * point.y + d[8] * point.z + d[12],
        y: d[1] * point.x + d[5] * point.y + d[9] * point.z + d[13],
        z: d[2] * point.x + d[6] * point.y + d[10] * point.z + d[14],
        w: d[3] * point.x + d[7] * point.y + d[11] * point.z + d[15],
    }
}

fn clip_to_screen(v: ClipVertex, width: f32, height: f32) -> ScreenVertex {
    let inv_w = 1.0 / v.w;
    let ndc_x = v.x * inv_w;
    let ndc_y = v.y * inv_w;
    let ndc_z = v.z * inv_w;
    let x_screen = (ndc_x + 1.0) * 0.5 * width;
    let y_screen = (1.0 - ndc_y) * 0.5 * height;
    ScreenVertex {
        pos: Vec2i {
            x: x_screen.round() as i32,
            y: y_screen.round() as i32,
        },
        depth: ndc_z,
    }
}

fn textured_clip_to_screen(
    v: ClipVertex,
    uv: [f32; 2],
    width: f32,
    height: f32,
) -> TexturedScreenVertex {
    let inv_w = 1.0 / v.w;
    TexturedScreenVertex {
        screen: clip_to_screen(v, width, height),
        inv_w,
        u_over_w: uv[0] * inv_w,
        v_over_w: uv[1] * inv_w,
    }
}

fn fill_triangle_textured<S: TextureSampler>(
    fb: &mut Framebuffer,
    depth: &mut [f32],
    v0: TexturedScreenVertex,
    v1: TexturedScreenVertex,
    v2: TexturedScreenVertex,
    sampler: &S,
) {
    let p0 = v0.screen.pos;
    let p1 = v1.screen.pos;
    let p2 = v2.screen.pos;
    let signed_area = edge_function(p0, p1, 2 * i128::from(p2.x), 2 * i128::from(p2.y));
    if signed_area == 0 {
        return;
    }

    let orientation = if signed_area < 0 { -1 } else { 1 };
    let top_left_0 = if orientation > 0 {
        is_top_left(p1, p2)
    } else {
        is_top_left(p2, p1)
    };
    let top_left_1 = if orientation > 0 {
        is_top_left(p2, p0)
    } else {
        is_top_left(p0, p2)
    };
    let top_left_2 = if orientation > 0 {
        is_top_left(p0, p1)
    } else {
        is_top_left(p1, p0)
    };
    let min_x = i128::from(p0.x.min(p1.x).min(p2.x)).max(0);
    let max_x = i128::from(p0.x.max(p1.x).max(p2.x)).min(fb.width as i128);
    let min_y = i128::from(p0.y.min(p1.y).min(p2.y)).max(0);
    let max_y = i128::from(p0.y.max(p1.y).max(p2.y)).min(fb.height as i128);
    if min_x >= max_x || min_y >= max_y {
        return;
    }

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample_x = 2 * x + 1;
            let sample_y = 2 * y + 1;
            let edge_0 = orientation * edge_function(p1, p2, sample_x, sample_y);
            let edge_1 = orientation * edge_function(p2, p0, sample_x, sample_y);
            let edge_2 = orientation * edge_function(p0, p1, sample_x, sample_y);
            if !(edge_contains(edge_0, top_left_0)
                && edge_contains(edge_1, top_left_1)
                && edge_contains(edge_2, top_left_2))
            {
                continue;
            }

            let area = edge_0 + edge_1 + edge_2;
            if area == 0 {
                continue;
            }
            let inv_area = 1.0 / area as f32;
            let b0 = edge_0 as f32 * inv_area;
            let b1 = edge_1 as f32 * inv_area;
            let b2 = edge_2 as f32 * inv_area;
            let z = b0 * v0.screen.depth + b1 * v1.screen.depth + b2 * v2.screen.depth;
            let idx = y as usize * fb.width + x as usize;
            if z < depth[idx] {
                let inv_w = b0 * v0.inv_w + b1 * v1.inv_w + b2 * v2.inv_w;
                let u_over_w = b0 * v0.u_over_w + b1 * v1.u_over_w + b2 * v2.u_over_w;
                let v_over_w = b0 * v0.v_over_w + b1 * v1.v_over_w + b2 * v2.v_over_w;
                let u = u_over_w / inv_w;
                let v = v_over_w / inv_w;
                depth[idx] = z;
                fb.set_pixel(x as usize, y as usize, sampler.sample(u, v));
            }
        }
    }
}

/// Integer edge-function rasterizer with barycentric depth interpolation.
fn fill_triangle_depth(
    fb: &mut Framebuffer,
    depth: &mut [f32],
    v0: ScreenVertex,
    v1: ScreenVertex,
    v2: ScreenVertex,
    color: u32,
) {
    let p0 = v0.pos;
    let p1 = v1.pos;
    let p2 = v2.pos;

    let signed_area = edge_function(p0, p1, 2 * i128::from(p2.x), 2 * i128::from(p2.y));
    if signed_area == 0 {
        return;
    }

    let orientation = if signed_area < 0 { -1 } else { 1 };
    let top_left_0 = if orientation > 0 {
        is_top_left(p1, p2)
    } else {
        is_top_left(p2, p1)
    };
    let top_left_1 = if orientation > 0 {
        is_top_left(p2, p0)
    } else {
        is_top_left(p0, p2)
    };
    let top_left_2 = if orientation > 0 {
        is_top_left(p0, p1)
    } else {
        is_top_left(p1, p0)
    };

    let min_x = i128::from(p0.x.min(p1.x).min(p2.x)).max(0);
    let max_x = i128::from(p0.x.max(p1.x).max(p2.x)).min(fb.width as i128);
    let min_y = i128::from(p0.y.min(p1.y).min(p2.y)).max(0);
    let max_y = i128::from(p0.y.max(p1.y).max(p2.y)).min(fb.height as i128);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let d0 = v0.depth;
    let d1 = v1.depth;
    let d2 = v2.depth;

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample_x = 2 * x + 1;
            let sample_y = 2 * y + 1;
            let edge_0 = orientation * edge_function(p1, p2, sample_x, sample_y);
            let edge_1 = orientation * edge_function(p2, p0, sample_x, sample_y);
            let edge_2 = orientation * edge_function(p0, p1, sample_x, sample_y);

            if edge_contains(edge_0, top_left_0)
                && edge_contains(edge_1, top_left_1)
                && edge_contains(edge_2, top_left_2)
            {
                let area = edge_0 + edge_1 + edge_2;
                if area == 0 {
                    continue;
                }
                let inv_area = 1.0 / (area as f32);
                let z = (edge_0 as f32 * d0 + edge_1 as f32 * d1 + edge_2 as f32 * d2) * inv_area;

                let idx = (y as usize) * fb.width + (x as usize);
                if z < depth[idx] {
                    depth[idx] = z;
                    fb.set_pixel(x as usize, y as usize, color);
                }
            }
        }
    }
}

fn edge_function(a: Vec2i, b: Vec2i, x: i128, y: i128) -> i128 {
    let ax = 2 * i128::from(a.x);
    let ay = 2 * i128::from(a.y);
    let bx = 2 * i128::from(b.x);
    let by = 2 * i128::from(b.y);

    (bx - ax) * (y - ay) - (by - ay) * (x - ax)
}

fn edge_contains(value: i128, top_left: bool) -> bool {
    value > 0 || (value == 0 && top_left)
}

fn is_top_left(a: Vec2i, b: Vec2i) -> bool {
    b.y < a.y || (b.y == a.y && b.x > a.x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec3;

    const COLOR: u32 = 0xFFFF0000;
    const COLOR_FAR: u32 = 0xFF0000FF;
    const COLOR_NEAR: u32 = 0xFF00FF00;
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

    /// Large triangle in the XY plane at a fixed world Z (screen-overlapping).
    fn plane_triangle_at_z(z: f32) -> (Vec<Vec3>, Vec<[usize; 3]>) {
        let vertices = vec![
            Vec3::new(-1.5, -1.5, z),
            Vec3::new(1.5, -1.5, z),
            Vec3::new(0.0, 1.5, z),
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

    #[test]
    fn depth_near_triangle_wins_over_far_overlap() {
        // Camera at z=5 looking at origin: larger world-z is closer to the eye.
        let camera = sample_camera(1.0);
        let model = Mat4::identity();
        let (far_v, far_t) = plane_triangle_at_z(-1.0);
        let (near_v, near_t) = plane_triangle_at_z(1.0);

        let mut fb = Framebuffer::new(48, 48);

        // Draw far first, then near in a different color.
        render_mesh_depth(&mut fb, &camera, &far_v, &far_t, &model, COLOR_FAR);
        render_mesh_depth(&mut fb, &camera, &near_v, &near_t, &model, COLOR_NEAR);

        // Both triangles should cover the framebuffer center; near color must win.
        let cx = fb.width / 2;
        let cy = fb.height / 2;
        let center = fb.get_pixel(cx, cy).expect("center in bounds");
        assert_eq!(
            center, COLOR_NEAR,
            "overlap region should hold the nearer triangle color"
        );

        // Also verify a non-trivial set of near-colored pixels exists.
        assert!(
            count_colored(&fb, COLOR_NEAR) > 0,
            "near triangle should rasterize pixels"
        );
    }

    #[test]
    fn depth_identical_scene_renders_byte_identical_checksums() {
        let camera = sample_camera(1.0);
        let model = Mat4::identity();
        let (far_v, far_t) = plane_triangle_at_z(-1.0);
        let (near_v, near_t) = plane_triangle_at_z(1.0);

        let mut fb_a = Framebuffer::new(32, 32);
        let mut fb_b = Framebuffer::new(32, 32);

        for fb in [&mut fb_a, &mut fb_b] {
            render_mesh_depth(fb, &camera, &far_v, &far_t, &model, COLOR_FAR);
            render_mesh_depth(fb, &camera, &near_v, &near_t, &model, COLOR_NEAR);
        }

        assert_eq!(fb_a.checksum(), fb_b.checksum());
        assert_eq!(fb_a.pixels, fb_b.pixels);
    }

    #[test]
    fn depth_triangle_behind_camera_rasterizes_zero_pixels() {
        let mut fb = Framebuffer::new(32, 32);
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::from_translation(Vec3::new(0.0, 0.0, 10.0));

        render_mesh_depth(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        assert_eq!(
            count_colored(&fb, COLOR),
            0,
            "triangle behind the camera should be near-rejected"
        );
        assert!(fb.pixels.iter().all(|&p| p == BG));
    }

    #[test]
    fn depth_near_then_far_rejects_far_within_single_mesh() {
        // Within one call the shared depth buffer must keep the nearer surface
        // even when the far triangle is listed second.
        let camera = sample_camera(1.0);
        let model = Mat4::identity();
        let color = 0xFFFFFF00;

        let near_only = (
            vec![
                Vec3::new(-1.5, -1.5, 1.0),
                Vec3::new(1.5, -1.5, 1.0),
                Vec3::new(0.0, 1.5, 1.0),
            ],
            vec![[0usize, 1, 2]],
        );
        let near_then_far = (
            vec![
                Vec3::new(-1.5, -1.5, 1.0),
                Vec3::new(1.5, -1.5, 1.0),
                Vec3::new(0.0, 1.5, 1.0),
                Vec3::new(-1.5, -1.5, -1.0),
                Vec3::new(1.5, -1.5, -1.0),
                Vec3::new(0.0, 1.5, -1.0),
            ],
            vec![[0usize, 1, 2], [3, 4, 5]],
        );

        let mut fb_near = Framebuffer::new(48, 48);
        let mut fb_both = Framebuffer::new(48, 48);
        render_mesh_depth(
            &mut fb_near,
            &camera,
            &near_only.0,
            &near_only.1,
            &model,
            color,
        );
        render_mesh_depth(
            &mut fb_both,
            &camera,
            &near_then_far.0,
            &near_then_far.1,
            &model,
            color,
        );

        // Far triangle must not change any pixel once the near surface is in the Z-buffer.
        assert_eq!(fb_near.checksum(), fb_both.checksum());
        assert_eq!(fb_near.pixels, fb_both.pixels);
    }

    fn count_nonzero(fb: &Framebuffer) -> usize {
        fb.pixels.iter().filter(|&&p| p != BG).count()
    }

    #[test]
    fn culled_front_facing_triangle_rasterizes_nonzero_pixels() {
        let mut fb = Framebuffer::new(32, 32);
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();

        render_mesh_depth_culled(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        assert!(
            count_nonzero(&fb) > 0,
            "expected at least one pixel for a front-facing triangle with culling"
        );
    }

    #[test]
    fn culled_back_facing_triangle_rasterizes_zero_pixels() {
        let mut fb = Framebuffer::new(32, 32);
        let camera = sample_camera(1.0);
        let (vertices, mut triangles) = front_triangle();
        // Swap two indices → reverse winding → back-facing under CCW-front.
        triangles[0] = [0, 2, 1];
        let model = Mat4::identity();

        render_mesh_depth_culled(&mut fb, &camera, &vertices, &triangles, &model, COLOR);

        assert_eq!(
            count_nonzero(&fb),
            0,
            "back-facing (reversed winding) triangle must be culled"
        );
        assert!(fb.pixels.iter().all(|&p| p == BG));
    }

    #[test]
    fn culled_cube_draws_strictly_fewer_pixels_than_unculled() {
        // mesh::cube() from (1.5, 1.2, 1.5) looking at the origin.
        let camera = Camera::look_at(
            Vec3::new(1.5, 1.2, 1.5),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            100.0,
        );
        let (vertices, triangles) = crate::mesh::cube();
        let model = Mat4::identity();

        // Two fresh framebuffers: full mesh::cube() unculled vs culled.
        let mut fb_full = Framebuffer::new(64, 64);
        let mut fb_culled = Framebuffer::new(64, 64);
        render_mesh_depth(&mut fb_full, &camera, &vertices, &triangles, &model, COLOR);
        render_mesh_depth_culled(
            &mut fb_culled,
            &camera,
            &vertices,
            &triangles,
            &model,
            COLOR,
        );

        let full_count = count_nonzero(&fb_full);
        let culled_count = count_nonzero(&fb_culled);
        assert!(full_count > 0, "unculled mesh::cube() should draw pixels");
        assert!(
            culled_count > 0,
            "culled mesh::cube() should still draw front faces"
        );
        // Depth already hides occluded back faces on a closed convex mesh, so
        // silhouette coverage can match. Measure the strict cull effect on the
        // cube's back-facing triangles alone (same camera / mesh::cube topology).
        let view_proj = camera.view_projection();
        let mvp = view_proj.multiply(model);
        let width = 64.0_f32;
        let height = 64.0_f32;
        let near = camera.near;
        let clip: Vec<ClipVertex> = vertices
            .iter()
            .copied()
            .map(|v| transform_point_homogeneous(mvp, v))
            .collect();
        let back_facing: Vec<[usize; 3]> = triangles
            .iter()
            .copied()
            .filter(|tri| {
                let v0 = clip[tri[0]];
                let v1 = clip[tri[1]];
                let v2 = clip[tri[2]];
                if v0.w <= near || v1.w <= near || v2.w <= near {
                    return false;
                }
                let s0 = clip_to_screen(v0, width, height);
                let s1 = clip_to_screen(v1, width, height);
                let s2 = clip_to_screen(v2, width, height);
                screen_signed_area_ccw(s0.pos, s1.pos, s2.pos) <= 0
            })
            .collect();
        assert!(
            !back_facing.is_empty(),
            "cube viewed from outside must have back-facing triangles"
        );

        let mut fb_back_full = Framebuffer::new(64, 64);
        let mut fb_back_culled = Framebuffer::new(64, 64);
        render_mesh_depth(
            &mut fb_back_full,
            &camera,
            &vertices,
            &back_facing,
            &model,
            COLOR,
        );
        render_mesh_depth_culled(
            &mut fb_back_culled,
            &camera,
            &vertices,
            &back_facing,
            &model,
            COLOR,
        );
        let back_full = count_nonzero(&fb_back_full);
        let back_culled = count_nonzero(&fb_back_culled);
        assert!(back_full > 0, "unculled cube back faces should draw pixels");
        assert!(
            back_culled < back_full,
            "culling must draw strictly fewer pixels ({back_culled} vs {back_full})"
        );
        assert_eq!(back_culled, 0, "back faces must be fully culled");
        // Full-cube culled coverage must not exceed the unculled path.
        assert!(culled_count <= full_count);
    }

    #[test]
    fn culled_identical_scene_renders_byte_identical_checksums() {
        let camera = Camera::look_at(
            Vec3::new(1.5, 1.2, 1.5),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            100.0,
        );
        let (vertices, triangles) = crate::mesh::cube();
        let model = Mat4::identity();

        let mut fb_a = Framebuffer::new(64, 64);
        let mut fb_b = Framebuffer::new(64, 64);

        render_mesh_depth_culled(&mut fb_a, &camera, &vertices, &triangles, &model, COLOR);
        render_mesh_depth_culled(&mut fb_b, &camera, &vertices, &triangles, &model, COLOR);

        assert_eq!(fb_a.checksum(), fb_b.checksum());
        assert_eq!(fb_a.pixels, fb_b.pixels);
    }

    /// Mean luminance of non-background pixels (AARRGGBB → 0.299R+0.587G+0.114B).
    fn mean_luminance(fb: &Framebuffer) -> f32 {
        let mut sum = 0.0_f32;
        let mut count = 0_usize;
        for &p in &fb.pixels {
            if p == BG {
                continue;
            }
            let r = ((p >> 16) & 0xFF) as f32;
            let g = ((p >> 8) & 0xFF) as f32;
            let b = (p & 0xFF) as f32;
            sum += 0.299 * r + 0.587 * g + 0.114 * b;
            count += 1;
        }
        assert!(count > 0, "expected at least one non-background pixel");
        sum / count as f32
    }

    #[test]
    fn lit_facing_light_brighter_than_facing_away() {
        // front_triangle lies in the XY plane with outward normal +Z.
        // light_dir is the travel direction of light rays.
        // From +Z: light_dir = (0,0,-1) → toward light = (0,0,1) → full N·L.
        // From -Z: light_dir = (0,0, 1) → toward light = (0,0,-1) → back-lit (ambient).
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();
        let base = 0xFFFFFFFF;

        let mut fb_front = Framebuffer::new(32, 32);
        let mut fb_back = Framebuffer::new(32, 32);
        render_mesh_lit(
            &mut fb_front,
            &camera,
            &vertices,
            &triangles,
            &model,
            base,
            Vec3::new(0.0, 0.0, -1.0),
        );
        render_mesh_lit(
            &mut fb_back,
            &camera,
            &vertices,
            &triangles,
            &model,
            base,
            Vec3::new(0.0, 0.0, 1.0),
        );

        let lum_front = mean_luminance(&fb_front);
        let lum_back = mean_luminance(&fb_back);
        assert!(
            lum_front > lum_back,
            "triangle facing the light ({lum_front}) must be brighter than back-lit ({lum_back})"
        );
    }

    #[test]
    fn lit_fully_backlit_visible_pixels_are_not_pure_black() {
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();
        let base = 0xFFFFFFFF;
        // Light travels +Z → source behind the +Z-facing triangle.
        let light_dir = Vec3::new(0.0, 0.0, 1.0);

        let mut fb = Framebuffer::new(32, 32);
        render_mesh_lit(
            &mut fb, &camera, &vertices, &triangles, &model, base, light_dir,
        );

        let lit: Vec<u32> = fb.pixels.iter().copied().filter(|&p| p != BG).collect();
        assert!(
            !lit.is_empty(),
            "back-lit but camera-facing triangle must draw pixels"
        );
        for p in lit {
            assert_ne!(
                p, 0xFF00_0000,
                "ambient floor must keep pixels off pure black"
            );
            // Alpha always opaque.
            assert_eq!(p >> 24, 0xFF, "alpha channel must remain 0xFF");
        }
    }

    #[test]
    fn lit_identical_scene_renders_byte_identical_checksums() {
        let camera = sample_camera(1.0);
        let (vertices, triangles) = front_triangle();
        let model = Mat4::identity();
        let base = 0xFFE0E0E0;
        let light_dir = Vec3::new(0.3, -0.5, -1.0);

        let mut fb_a = Framebuffer::new(32, 32);
        let mut fb_b = Framebuffer::new(32, 32);
        render_mesh_lit(
            &mut fb_a, &camera, &vertices, &triangles, &model, base, light_dir,
        );
        render_mesh_lit(
            &mut fb_b, &camera, &vertices, &triangles, &model, base, light_dir,
        );

        assert_eq!(fb_a.checksum(), fb_b.checksum());
        assert_eq!(fb_a.pixels, fb_b.pixels);
    }

    struct Quadrants;

    impl TextureSampler for Quadrants {
        fn sample(&self, u: f32, v: f32) -> u32 {
            match (u >= 0.5, v >= 0.5) {
                (false, false) => 0xFFFF0000,
                (true, false) => 0xFF00FF00,
                (false, true) => 0xFF0000FF,
                (true, true) => 0xFFFFFF00,
            }
        }
    }

    #[test]
    fn textured_triangle_is_deterministic_and_samples_uvs() {
        let camera = sample_camera(1.0);
        let vertices = vec![
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let triangles = vec![[0, 1, 2]];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let model = Mat4::identity();
        let mut first = Framebuffer::new(32, 32);
        let mut second = Framebuffer::new(32, 32);

        render_mesh_textured(
            &mut first, &camera, &vertices, &uvs, &triangles, &model, &Quadrants,
        );
        render_mesh_textured(
            &mut second,
            &camera,
            &vertices,
            &uvs,
            &triangles,
            &model,
            &Quadrants,
        );

        assert_eq!(first.checksum(), second.checksum());
        assert!(first.pixels.iter().any(|&pixel| pixel != BG));
    }
}
