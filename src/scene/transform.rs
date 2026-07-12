use crate::math::mat4::Mat4;
use crate::math::quat::Quat;
use crate::math::vec3::Vec3;

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
        Mat4::from_translation(self.translation)
            .multiply(rotation_matrix(self.rotation))
            .multiply(Mat4::from_scale(self.scale))
    }
}

fn rotation_matrix(rotation: Quat) -> Mat4 {
    let rotation = rotation.normalize();
    let xx = rotation.x * rotation.x;
    let yy = rotation.y * rotation.y;
    let zz = rotation.z * rotation.z;
    let xy = rotation.x * rotation.y;
    let xz = rotation.x * rotation.z;
    let yz = rotation.y * rotation.z;
    let xw = rotation.x * rotation.w;
    let yw = rotation.y * rotation.w;
    let zw = rotation.z * rotation.w;

    Mat4 {
        data: [
            1.0 - 2.0 * (yy + zz),
            2.0 * (xy + zw),
            2.0 * (xz - yw),
            0.0,
            2.0 * (xy - zw),
            1.0 - 2.0 * (xx + zz),
            2.0 * (yz + xw),
            0.0,
            2.0 * (xz + yw),
            2.0 * (yz - xw),
            1.0 - 2.0 * (xx + yy),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ],
    }
}

#[cfg(test)]
mod tests {
    use crate::math::mat4::Mat4;
    use crate::math::quat::Quat;
    use crate::math::vec3::Vec3;

    use super::Transform;

    const EPSILON: f32 = 1.0e-6;

    fn assert_mat4_near(actual: Mat4, expected: Mat4) {
        for (actual, expected) in actual.data.iter().zip(expected.data.iter()) {
            assert!(
                (actual - expected).abs() < EPSILON,
                "{actual} != {expected}"
            );
        }
    }

    fn assert_vec3_near(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
        assert!((actual.z - expected.z).abs() < EPSILON);
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
    fn identity_transform_maps_to_identity_matrix() {
        assert_mat4_near(Transform::IDENTITY.to_mat4(), Mat4::identity());
    }

    #[test]
    fn pure_translation_moves_a_point() {
        let transform = Transform {
            translation: Vec3::new(2.0, 3.0, 4.0),
            ..Transform::IDENTITY
        };

        assert_vec3_near(
            transform_point(transform.to_mat4(), Vec3::new(1.0, 1.0, 1.0)),
            Vec3::new(3.0, 4.0, 5.0),
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
