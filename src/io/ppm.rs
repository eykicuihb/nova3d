use crate::raster::Framebuffer;

/// Export a framebuffer as a plain-text PPM (P3) string.
///
/// Pixel format is 0xAARRGGBB; alpha is dropped. Output ends with a trailing newline.
pub fn to_ppm(fb: &Framebuffer) -> String {
    let mut out = String::new();
    out.push_str("P3\n");
    out.push_str(&format!("{} {}\n", fb.width, fb.height));
    out.push_str("255\n");

    for y in 0..fb.height {
        for x in 0..fb.width {
            let pixel = fb.pixels[y * fb.width + x];
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;
            if x > 0 {
                out.push(' ');
            }
            out.push_str(&format!("{r} {g} {b}"));
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::to_ppm;
    use crate::raster::Framebuffer;

    #[test]
    fn one_by_one_opaque_red() {
        let mut fb = Framebuffer::new(1, 1);
        fb.set_pixel(0, 0, 0xFFFF0000);
        assert_eq!(to_ppm(&fb), "P3\n1 1\n255\n255 0 0\n");
    }

    #[test]
    fn two_by_two_distinct_colors_row_major() {
        let mut fb = Framebuffer::new(2, 2);
        // Row 0: red, green
        fb.set_pixel(0, 0, 0xFFFF0000);
        fb.set_pixel(1, 0, 0xFF00FF00);
        // Row 1: blue, white
        fb.set_pixel(0, 1, 0xFF0000FF);
        fb.set_pixel(1, 1, 0xFFFFFFFF);

        assert_eq!(
            to_ppm(&fb),
            "P3\n2 2\n255\n255 0 0 0 255 0\n0 0 255 255 255 255\n"
        );
    }

    #[test]
    fn cleared_framebuffer_has_width_times_height_triplets() {
        let width = 3;
        let height = 4;
        let mut fb = Framebuffer::new(width, height);
        fb.clear(0xFF112233);

        let ppm = to_ppm(&fb);
        let body = ppm.lines().skip(3).collect::<Vec<_>>().join(" ");
        let triplets = body.split_whitespace().collect::<Vec<_>>();
        // Each triplet is R G B → 3 numbers per pixel
        assert_eq!(triplets.len(), width * height * 3);
    }
}
