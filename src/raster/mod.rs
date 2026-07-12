mod framebuffer;
mod triangle;

pub use framebuffer::Framebuffer;
pub use triangle::{fill_triangle, Vec2i};

#[cfg(test)]
mod tests {
    use super::{fill_triangle, Framebuffer, Vec2i};

    #[test]
    fn tiled_triangles_match_a_cleared_framebuffer_checksum() {
        let color = 0xFF336699;
        let mut rendered = Framebuffer::new(16, 16);

        fill_triangle(
            &mut rendered,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 16, y: 0 },
            Vec2i { x: 16, y: 16 },
            color,
        );
        fill_triangle(
            &mut rendered,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 16, y: 16 },
            Vec2i { x: 0, y: 16 },
            color,
        );

        let mut cleared = Framebuffer::new(16, 16);
        cleared.clear(color);

        assert_eq!(rendered.checksum(), cleared.checksum());
    }
}
