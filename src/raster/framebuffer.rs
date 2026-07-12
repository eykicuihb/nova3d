pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0xFF000000; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            Some(self.pixels[y * self.width + x])
        } else {
            None
        }
    }

    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf29ce484222325;

        for pixel in &self.pixels {
            for byte in pixel.to_be_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::Framebuffer;

    #[test]
    fn new_fills_every_pixel_with_opaque_black() {
        let framebuffer = Framebuffer::new(2, 3);

        for y in 0..3 {
            for x in 0..2 {
                assert_eq!(framebuffer.get_pixel(x, y), Some(0xFF000000));
            }
        }
    }

    #[test]
    fn clear_overwrites_every_pixel() {
        let mut framebuffer = Framebuffer::new(2, 2);
        framebuffer.set_pixel(0, 0, 0x01020304);

        framebuffer.clear(0xAABBCCDD);

        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(framebuffer.get_pixel(x, y), Some(0xAABBCCDD));
            }
        }
    }

    #[test]
    fn set_and_get_round_trip() {
        let mut framebuffer = Framebuffer::new(2, 2);

        framebuffer.set_pixel(1, 0, 0x11223344);

        assert_eq!(framebuffer.get_pixel(1, 0), Some(0x11223344));
    }

    #[test]
    fn out_of_bounds_access_is_ignored_or_returns_none() {
        let mut framebuffer = Framebuffer::new(2, 2);
        let before = framebuffer.checksum();

        framebuffer.set_pixel(2, 0, 0x11223344);
        framebuffer.set_pixel(0, 2, 0x55667788);

        assert_eq!(framebuffer.get_pixel(2, 0), None);
        assert_eq!(framebuffer.get_pixel(0, 2), None);
        assert_eq!(framebuffer.checksum(), before);
    }

    #[test]
    fn checksum_changes_when_a_pixel_changes() {
        let mut framebuffer = Framebuffer::new(2, 2);
        let before = framebuffer.checksum();

        framebuffer.set_pixel(1, 1, 0x11223344);

        assert_ne!(framebuffer.checksum(), before);
    }
}
