use crate::math::{Mat4, Vec3};

/// A perspective camera positioned in world space via a look-at frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_radians: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    /// Builds a camera that looks from `position` toward `target`.
    pub fn look_at(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov_y_radians,
            aspect,
            near,
            far,
        }
    }

    /// Right-handed look-at view transform (world → camera).
    ///
    /// Basis vectors are formed from normalized cross products; translation is
    /// folded into the matrix so the camera origin maps to the world position.
    pub fn view_matrix(&self) -> Mat4 {
        // Camera looks down -Z: forward-away axis points from target toward eye.
        let forward = (self.position - self.target).normalize();
        let right = self.up.cross(forward).normalize();
        let up = forward.cross(right);

        Mat4 {
            data: [
                right.x,
                up.x,
                forward.x,
                0.0,
                right.y,
                up.y,
                forward.y,
                0.0,
                right.z,
                up.z,
                forward.z,
                0.0,
                -right.dot(self.position),
                -up.dot(self.position),
                -forward.dot(self.position),
                1.0,
            ],
        }
    }

    /// Perspective projection matrix for this camera's frustum parameters.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y_radians, self.aspect, self.near, self.far)
    }

    /// Combined projection * view transform (world → clip).
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix().multiply(self.view_matrix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1.0e-5;

    fn assert_vec3_near(actual: Vec3, expected: Vec3) {
        assert!(
            (actual.x - expected.x).abs() < EPSILON,
            "x: {actual:?} vs {expected:?}"
        );
        assert!(
            (actual.y - expected.y).abs() < EPSILON,
            "y: {actual:?} vs {expected:?}"
        );
        assert!(
            (actual.z - expected.z).abs() < EPSILON,
            "z: {actual:?} vs {expected:?}"
        );
    }

    fn assert_mat4_near(actual: Mat4, expected: Mat4) {
        for (index, (a, e)) in actual.data.iter().zip(expected.data.iter()).enumerate() {
            assert!((a - e).abs() < EPSILON, "element {index}: {a} vs {e}");
        }
    }

    fn transform_point(matrix: Mat4, point: Vec3) -> Vec3 {
        Vec3::new(
            matrix.data[0] * point.x
                + matrix.data[4] * point.y
                + matrix.data[8] * point.z
                + matrix.data[12],
            matrix.data[1] * point.x
                + matrix.data[5] * point.y
                + matrix.data[9] * point.z
                + matrix.data[13],
            matrix.data[2] * point.x
                + matrix.data[6] * point.y
                + matrix.data[10] * point.z
                + matrix.data[14],
        )
    }

    fn sample_camera() -> Camera {
        Camera::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            std::f32::consts::FRAC_PI_2,
            16.0 / 9.0,
            0.1,
            100.0,
        )
    }

    #[test]
    fn view_matrix_maps_camera_position_to_origin() {
        let camera = sample_camera();
        let view = camera.view_matrix();
        assert_vec3_near(transform_point(view, camera.position), Vec3::ZERO);
    }

    #[test]
    fn view_matrix_maps_target_onto_negative_z_axis() {
        let camera = sample_camera();
        let view = camera.view_matrix();
        let mapped = transform_point(view, camera.target);

        assert!(
            (mapped.x).abs() < EPSILON,
            "x should be ~0, got {}",
            mapped.x
        );
        assert!(
            (mapped.y).abs() < EPSILON,
            "y should be ~0, got {}",
            mapped.y
        );
        assert!(
            mapped.z < -EPSILON,
            "target should lie on negative Z, got z={}",
            mapped.z
        );
    }

    #[test]
    fn view_projection_equals_projection_times_view() {
        let camera = sample_camera();
        let expected = camera.projection_matrix().multiply(camera.view_matrix());
        assert_mat4_near(camera.view_projection(), expected);
    }
}
