mod transform {
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
}

mod graph;

pub use graph::{NodeId, SceneGraph};
pub use transform::Transform;
