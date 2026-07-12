use super::framebuffer::Framebuffer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

pub fn fill_triangle(fb: &mut Framebuffer, v0: Vec2i, v1: Vec2i, v2: Vec2i, color: u32) {
    let signed_area = edge_function(v0, v1, 2 * i128::from(v2.x), 2 * i128::from(v2.y));
    if signed_area == 0 {
        return;
    }

    let orientation = if signed_area < 0 { -1 } else { 1 };
    let top_left_0 = if orientation > 0 {
        is_top_left(v1, v2)
    } else {
        is_top_left(v2, v1)
    };
    let top_left_1 = if orientation > 0 {
        is_top_left(v2, v0)
    } else {
        is_top_left(v0, v2)
    };
    let top_left_2 = if orientation > 0 {
        is_top_left(v0, v1)
    } else {
        is_top_left(v1, v0)
    };

    let min_x = i128::from(v0.x.min(v1.x).min(v2.x)).max(0);
    let max_x = i128::from(v0.x.max(v1.x).max(v2.x)).min(fb.width as i128);
    let min_y = i128::from(v0.y.min(v1.y).min(v2.y)).max(0);
    let max_y = i128::from(v0.y.max(v1.y).max(v2.y)).min(fb.height as i128);

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample_x = 2 * x + 1;
            let sample_y = 2 * y + 1;
            let edge_0 = orientation * edge_function(v1, v2, sample_x, sample_y);
            let edge_1 = orientation * edge_function(v2, v0, sample_x, sample_y);
            let edge_2 = orientation * edge_function(v0, v1, sample_x, sample_y);

            if edge_contains(edge_0, top_left_0)
                && edge_contains(edge_1, top_left_1)
                && edge_contains(edge_2, top_left_2)
            {
                fb.set_pixel(x as usize, y as usize, color);
            }
        }
    }
}

fn edge_function(a: Vec2i, b: Vec2i, x: i128, y: i128) -> i128 {
    let ax = 2 * i128::from(a.x);
    let ay = 2 * i128::from(a.y);
    let bx = 2 * i128::from(b.x);
    let by = 2 * i128::from(b.y);

    (bx - ax) * (y - ay) - (by - ay) * (x - ax)
}

fn edge_contains(value: i128, top_left: bool) -> bool {
    value > 0 || (value == 0 && top_left)
}

fn is_top_left(a: Vec2i, b: Vec2i) -> bool {
    b.y < a.y || (b.y == a.y && b.x > a.x)
}

#[cfg(test)]
mod tests {
    use super::super::framebuffer::Framebuffer;
    use super::{fill_triangle, Vec2i};

    const COLOR: u32 = 0xFF112233;

    #[test]
    fn right_triangle_fills_expected_pixel_count() {
        let mut framebuffer = Framebuffer::new(4, 4);

        fill_triangle(
            &mut framebuffer,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 4, y: 0 },
            Vec2i { x: 0, y: 4 },
            COLOR,
        );

        assert_eq!(
            framebuffer
                .pixels
                .iter()
                .filter(|&&pixel| pixel == COLOR)
                .count(),
            6
        );
    }

    #[test]
    fn degenerate_triangle_draws_zero_pixels() {
        let mut framebuffer = Framebuffer::new(4, 4);

        fill_triangle(
            &mut framebuffer,
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 2, y: 2 },
            Vec2i { x: 4, y: 4 },
            COLOR,
        );

        assert!(framebuffer.pixels.iter().all(|&pixel| pixel != COLOR));
    }

    #[test]
    fn winding_order_does_not_change_pixels() {
        let mut counter_clockwise = Framebuffer::new(5, 5);
        let mut clockwise = Framebuffer::new(5, 5);

        fill_triangle(
            &mut counter_clockwise,
            Vec2i { x: 1, y: 1 },
            Vec2i { x: 4, y: 1 },
            Vec2i { x: 1, y: 4 },
            COLOR,
        );
        fill_triangle(
            &mut clockwise,
            Vec2i { x: 1, y: 1 },
            Vec2i { x: 1, y: 4 },
            Vec2i { x: 4, y: 1 },
            COLOR,
        );

        assert_eq!(counter_clockwise.pixels, clockwise.pixels);
    }

    #[test]
    fn shared_diagonal_tiles_rectangle_without_gaps_or_double_draws() {
        let first_triangle = [
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 4, y: 0 },
            Vec2i { x: 4, y: 4 },
        ];
        let second_triangle = [
            Vec2i { x: 0, y: 0 },
            Vec2i { x: 4, y: 4 },
            Vec2i { x: 0, y: 4 },
        ];

        let mut framebuffer = Framebuffer::new(4, 4);
        fill_triangle(
            &mut framebuffer,
            first_triangle[0],
            first_triangle[1],
            first_triangle[2],
            0xFF000001,
        );
        fill_triangle(
            &mut framebuffer,
            second_triangle[0],
            second_triangle[1],
            second_triangle[2],
            0xFF000002,
        );

        assert!(framebuffer.pixels.iter().all(|&pixel| pixel != 0xFF000000));

        let mut first_draw = Framebuffer::new(4, 4);
        fill_triangle(
            &mut first_draw,
            first_triangle[0],
            first_triangle[1],
            first_triangle[2],
            COLOR,
        );
        let first_count = first_draw
            .pixels
            .iter()
            .filter(|&&pixel| pixel == COLOR)
            .count();

        let mut second_draw = Framebuffer::new(4, 4);
        fill_triangle(
            &mut second_draw,
            second_triangle[0],
            second_triangle[1],
            second_triangle[2],
            COLOR,
        );
        let second_count = second_draw
            .pixels
            .iter()
            .filter(|&&pixel| pixel == COLOR)
            .count();

        assert_eq!(first_count + second_count, 16);
    }

    #[test]
    fn partially_off_screen_triangle_only_writes_in_bounds() {
        let mut framebuffer = Framebuffer::new(3, 3);

        fill_triangle(
            &mut framebuffer,
            Vec2i { x: -1, y: -1 },
            Vec2i { x: 4, y: -1 },
            Vec2i { x: -1, y: 4 },
            COLOR,
        );

        assert_eq!(
            framebuffer
                .pixels
                .iter()
                .filter(|&&pixel| pixel == COLOR)
                .count(),
            3
        );
        assert_eq!(framebuffer.pixels.len(), 9);
    }
}
