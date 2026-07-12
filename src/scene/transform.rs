use crate::math::{Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::identity(),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };

    pub fn to_mat4(&self) -> Mat4 {
        let Quat { x, y, z, w } = self.rotation.normalize();
        let rotation = Mat4 {
            data: [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + z * w),
                2.0 * (x * z - y * w),
                0.0,
                2.0 * (x * y - z * w),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + x * w),
                0.0,
                2.0 * (x * z + y * w),
                2.0 * (y * z - x * w),
                1.0 - 2.0 * (x * x + y * y),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        };

        Mat4::from_translation(self.translation)
            .multiply(rotation)
            .multiply(Mat4::from_scale(self.scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1.0e-6;

    fn assert_vec3_near(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
        assert!((actual.z - expected.z).abs() < EPSILON);
    }

    fn assert_mat4_near(actual: Mat4, expected: Mat4) {
        for (actual, expected) in actual.data.iter().zip(expected.data.iter()) {
            assert!((actual - expected).abs() < EPSILON);
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

    #[test]
    fn identity_maps_to_identity_matrix() {
        assert_mat4_near(Transform::IDENTITY.to_mat4(), Mat4::identity());
    }

    #[test]
    fn pure_translation_moves_a_point() {
        let transform = Transform {
            translation: Vec3::new(2.0, 3.0, 4.0),
            ..Transform::IDENTITY
        };

        assert_vec3_near(
            transform_point(transform.to_mat4(), Vec3::new(1.0, 2.0, 3.0)),
            Vec3::new(3.0, 5.0, 7.0),
        );
    }

    #[test]
    fn ninety_degree_z_rotation_maps_unit_x_toward_unit_y() {
        let transform = Transform {
            rotation: Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), std::f32::consts::FRAC_PI_2),
            ..Transform::IDENTITY
        };

        assert_vec3_near(
            transform_point(transform.to_mat4(), Vec3::new(1.0, 0.0, 0.0)),
            Vec3::new(0.0, 1.0, 0.0),
        );
    }

    #[test]
    fn non_uniform_scale_scales_axes_independently() {
        let transform = Transform {
            scale: Vec3::new(2.0, 3.0, 4.0),
            ..Transform::IDENTITY
        };

        assert_vec3_near(
            transform_point(transform.to_mat4(), Vec3::new(1.0, 1.0, 1.0)),
            Vec3::new(2.0, 3.0, 4.0),
        );
    }
}
