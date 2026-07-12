mod ppm;

pub use ppm::to_ppm;

#[cfg(test)]
mod tests {
    use super::to_ppm;
    use crate::raster::{fill_triangle, Framebuffer, Vec2i};

    fn render_tiled_8x8() -> Framebuffer {
        let color = 0xFF336699;
        let mut fb = Framebuffer::new(8, 8);

        // Two triangles tiling the 8×8 rectangle along the shared diagonal.
        fill_triangle(
            &mut fb,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 8, y: 0 },
            Vec2i { x: 8, y: 8 },
            color,
        );
        fill_triangle(
            &mut fb,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 8, y: 8 },
            Vec2i { x: 0, y: 8 },
            color,
        );

        fb
    }

    #[test]
    fn golden_image_determinism_across_two_renders() {
        let fb1 = render_tiled_8x8();
        let fb2 = render_tiled_8x8();

        assert_eq!(fb1.checksum(), fb2.checksum());

        let ppm1 = to_ppm(&fb1);
        let ppm2 = to_ppm(&fb2);

        assert_eq!(ppm1, ppm2);
        assert!(ppm1.starts_with("P3\n8 8\n255\n"));
    }
}
