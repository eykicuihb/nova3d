use crate::math::vec3::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle_radians: f32) -> Self {
        let axis_length = axis.length();
        if axis_length == 0.0 {
            return Self::identity();
        }

        let axis = axis.scale(1.0 / axis_length);
        let (half_angle_sin, half_angle_cos) = (angle_radians * 0.5).sin_cos();
        Self {
            x: axis.x * half_angle_sin,
            y: axis.y * half_angle_sin,
            z: axis.z * half_angle_sin,
            w: half_angle_cos,
        }
    }

    pub fn multiply(self, other: Self) -> Self {
        Self {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }

    pub fn normalize(self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if length == 0.0 {
            Self::identity()
        } else {
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
                w: self.w / length,
            }
        }
    }

    pub fn rotate_vec3(self, vector: Vec3) -> Vec3 {
        let quaternion = self.normalize();
        let rotation_axis = Vec3::new(quaternion.x, quaternion.y, quaternion.z);
        let perpendicular = rotation_axis.cross(vector).scale(2.0);

        vector + perpendicular.scale(quaternion.w) + rotation_axis.cross(perpendicular)
    }
}
