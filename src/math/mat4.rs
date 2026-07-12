use super::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub data: [f32; 16],
}

impl Mat4 {
    pub const fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn multiply(self, other: Self) -> Self {
        let mut data = [0.0; 16];

        for column in 0..4 {
            for row in 0..4 {
                let mut value = 0.0;
                for index in 0..4 {
                    value += self.data[index * 4 + row] * other.data[column * 4 + index];
                }
                data[column * 4 + row] = value;
            }
        }

        Self { data }
    }

    pub fn transpose(self) -> Self {
        let mut data = [0.0; 16];

        for column in 0..4 {
            for row in 0..4 {
                data[column * 4 + row] = self.data[row * 4 + column];
            }
        }

        Self { data }
    }

    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            data: [
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                translation.x,
                translation.y,
                translation.z,
                1.0,
            ],
        }
    }

    pub const fn from_scale(scale: Vec3) -> Self {
        Self {
            data: [
                scale.x, 0.0, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, 0.0, scale.z, 0.0, 0.0, 0.0,
                0.0, 1.0,
            ],
        }
    }

    pub fn perspective(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let focal_length = 1.0 / (fov_y_radians * 0.5).tan();
        let depth_range = near - far;

        Self {
            data: [
                focal_length / aspect,
                0.0,
                0.0,
                0.0,
                0.0,
                focal_length,
                0.0,
                0.0,
                0.0,
                0.0,
                (far + near) / depth_range,
                -1.0,
                0.0,
                0.0,
                (2.0 * far * near) / depth_range,
                0.0,
            ],
        }
    }
}
