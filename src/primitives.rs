use super::*;

/// RGBA color helper for concise shape drawing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }
}

impl From<Color> for Rgba<u8> {
    fn from(value: Color) -> Self {
        Rgba(value.0)
    }
}

// Raster primitive implementations adapted from imageproc's MIT-licensed
// drawing modules. See docs/THIRD_PARTY_NOTICES.md.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Point<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

impl<T> Point<T> {
    pub(crate) const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Rect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Rect {
    pub(crate) const fn at(left: i32, top: i32) -> RectPosition {
        RectPosition { left, top }
    }

    pub(crate) const fn left(self) -> i32 {
        self.left
    }

    pub(crate) const fn top(self) -> i32 {
        self.top
    }

    pub(crate) fn right(self) -> i32 {
        let width_offset = self.width.saturating_sub(1).min(i32::MAX as u32) as i32;
        self.left.saturating_add(width_offset)
    }

    pub(crate) fn bottom(self) -> i32 {
        let height_offset = self.height.saturating_sub(1).min(i32::MAX as u32) as i32;
        self.top.saturating_add(height_offset)
    }

    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) fn intersect(self, other: Self) -> Option<Self> {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(Self::at(left, top).of_size(
            right.saturating_sub(left).saturating_add(1) as u32,
            bottom.saturating_sub(top).saturating_add(1) as u32,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RectPosition {
    pub(crate) left: i32,
    pub(crate) top: i32,
}

impl RectPosition {
    // Zero-size rectangles are promoted to one pixel so drawing helpers remain
    // non-panicking when integer layout rounds a narrow feature down to zero.
    pub(crate) const fn of_size(self, width: u32, height: u32) -> Rect {
        Rect {
            left: self.left,
            top: self.top,
            width: if width == 0 { 1 } else { width },
            height: if height == 0 { 1 } else { height },
        }
    }
}

pub(crate) fn draw_filled_rect_mut(image: &mut RgbaImage, rect: Rect, color: Rgba<u8>) {
    if image.width() == 0 || image.height() == 0 {
        return;
    }

    let bounds = Rect::at(0, 0).of_size(image.width(), image.height());
    if let Some(intersection) = bounds.intersect(rect) {
        for dy in 0..intersection.height() {
            for dx in 0..intersection.width() {
                let x = intersection.left() as u32 + dx;
                let y = intersection.top() as u32 + dy;
                image.put_pixel(x, y, color);
            }
        }
    }
}

pub(crate) fn draw_line_segment_mut(
    image: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    color: Rgba<u8>,
) {
    for (x, y) in BresenhamLineIter::new(start, end) {
        draw_if_in_bounds(image, x, y, color);
    }
}

pub(crate) struct BresenhamLineIter {
    dx: f32,
    dy: f32,
    x: i32,
    y: i32,
    error: f32,
    end_x: i32,
    is_steep: bool,
    y_step: i32,
}

impl BresenhamLineIter {
    fn new(start: (f32, f32), end: (f32, f32)) -> Self {
        let (mut x0, mut y0) = (start.0, start.1);
        let (mut x1, mut y1) = (end.0, end.1);

        let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
        if is_steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        }

        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        Self {
            dx,
            dy: (y1 - y0).abs(),
            x: x0 as i32,
            y: y0 as i32,
            error: dx / 2.0,
            end_x: x1 as i32,
            is_steep,
            y_step: if y0 < y1 { 1 } else { -1 },
        }
    }
}

impl Iterator for BresenhamLineIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x > self.end_x {
            return None;
        }

        let point = if self.is_steep {
            (self.y, self.x)
        } else {
            (self.x, self.y)
        };

        self.x += 1;
        self.error -= self.dy;
        if self.error < 0.0 {
            self.y += self.y_step;
            self.error += self.dx;
        }

        Some(point)
    }
}

pub(crate) fn draw_antialiased_line_segment_mut<B>(
    image: &mut RgbaImage,
    start: (i32, i32),
    end: (i32, i32),
    color: Rgba<u8>,
    blend: B,
) where
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    let (mut x0, mut y0) = (start.0, start.1);
    let (mut x1, mut y1) = (end.0, end.1);

    let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
    if is_steep {
        if y0 > y1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        plot_wu_line(image, (y0, x0), (y1, x1), color, |x, y| (y, x), blend);
    } else {
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        plot_wu_line(image, (x0, y0), (x1, y1), color, |x, y| (x, y), blend);
    }
}

pub(crate) fn plot_wu_line<T, B>(
    image: &mut RgbaImage,
    start: (i32, i32),
    end: (i32, i32),
    color: Rgba<u8>,
    transform: T,
    blend: B,
) where
    T: Fn(i32, i32) -> (i32, i32),
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    if dx == 0 {
        plot_antialiased_pixel(image, transform(start.0, start.1), color, 1.0, &blend);
        return;
    }
    let gradient = dy as f32 / dx as f32;
    let mut fy = start.1 as f32;

    for x in start.0..=end.0 {
        plot_antialiased_pixel(
            image,
            transform(x, fy as i32),
            color,
            1.0 - fy.fract(),
            &blend,
        );
        plot_antialiased_pixel(
            image,
            transform(x, fy as i32 + 1),
            color,
            fy.fract(),
            &blend,
        );
        fy += gradient;
    }
}

pub(crate) fn plot_antialiased_pixel<B>(
    image: &mut RgbaImage,
    (x, y): (i32, i32),
    color: Rgba<u8>,
    weight: f32,
    blend: &B,
) where
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    if in_bounds(image, x, y) {
        let original = *image.get_pixel(x as u32, y as u32);
        image.put_pixel(x as u32, y as u32, blend(color, original, weight));
    }
}

pub(crate) fn draw_filled_ellipse_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
    color: Rgba<u8>,
) {
    if width_radius == height_radius {
        draw_filled_circle_mut(image, center, width_radius, color);
        return;
    }

    draw_ellipse(
        |image, x0, y0, x, y| {
            draw_line_segment_mut(
                image,
                ((x0 - x) as f32, (y0 + y) as f32),
                ((x0 + x) as f32, (y0 + y) as f32),
                color,
            );
            draw_line_segment_mut(
                image,
                ((x0 - x) as f32, (y0 - y) as f32),
                ((x0 + x) as f32, (y0 - y) as f32),
                color,
            );
        },
        image,
        center,
        width_radius,
        height_radius,
    );
}

pub(crate) fn draw_ellipse<F>(
    mut render: F,
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
) where
    F: FnMut(&mut RgbaImage, i32, i32, i32, i32),
{
    let (x0, y0) = center;
    let w2 = (width_radius as f64).powi(2);
    let h2 = (height_radius as f64).powi(2);
    let mut x = 0;
    let mut y = height_radius;
    let mut px = 0.0;
    let mut py = 2.0 * w2 * y as f64;

    render(image, x0, y0, x, y);

    let mut p = h2 - (w2 * height_radius as f64) + (0.25 * w2);
    while px < py {
        x += 1;
        px += 2.0 * h2;
        if p < 0.0 {
            p += h2 + px;
        } else {
            y -= 1;
            py += -2.0 * w2;
            p += h2 + px - py;
        }
        render(image, x0, y0, x, y);
    }

    p = h2 * (x as f64 + 0.5).powi(2) + w2 * ((y - 1) as f64).powi(2) - w2 * h2;
    while y > 0 {
        y -= 1;
        py += -2.0 * w2;
        if p > 0.0 {
            p += w2 - py;
        } else {
            x += 1;
            px += 2.0 * h2;
            p += w2 - py + px;
        }
        render(image, x0, y0, x, y);
    }
}

pub(crate) fn draw_hollow_circle_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    radius: i32,
    color: Rgba<u8>,
) {
    let mut x = 0;
    let mut y = radius;
    let mut p = 1 - radius;
    let (x0, y0) = center;

    while x <= y {
        draw_if_in_bounds(image, x0 + x, y0 + y, color);
        draw_if_in_bounds(image, x0 + y, y0 + x, color);
        draw_if_in_bounds(image, x0 - y, y0 + x, color);
        draw_if_in_bounds(image, x0 - x, y0 + y, color);
        draw_if_in_bounds(image, x0 - x, y0 - y, color);
        draw_if_in_bounds(image, x0 - y, y0 - x, color);
        draw_if_in_bounds(image, x0 + y, y0 - x, color);
        draw_if_in_bounds(image, x0 + x, y0 - y, color);

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

pub(crate) fn draw_filled_circle_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    radius: i32,
    color: Rgba<u8>,
) {
    let mut x = 0;
    let mut y = radius;
    let mut p = 1 - radius;
    let (x0, y0) = center;

    while x <= y {
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

pub(crate) fn draw_polygon_mut(image: &mut RgbaImage, poly: &[Point<i32>], color: Rgba<u8>) {
    if poly.is_empty() || image.width() == 0 || image.height() == 0 {
        return;
    }

    let mut y_min = i32::MAX;
    let mut y_max = i32::MIN;
    for point in poly {
        y_min = y_min.min(point.y);
        y_max = y_max.max(point.y);
    }

    y_min = 0.max(y_min.min(image.height() as i32 - 1));
    y_max = 0.max(y_max.min(image.height() as i32 - 1));

    let mut closed = poly.to_vec();
    let first = poly[0];
    let last = poly[poly.len() - 1];
    if first != last {
        closed.push(first);
    }

    let edges: Vec<&[Point<i32>]> = closed.windows(2).collect();
    let mut intersections = Vec::new();

    for y in y_min..=y_max {
        for edge in &edges {
            let p0 = edge[0];
            let p1 = edge[1];

            if p0.y <= y && p1.y >= y || p1.y <= y && p0.y >= y {
                if p0.y == p1.y {
                    intersections.push(p0.x);
                    intersections.push(p1.x);
                } else if p0.y == y || p1.y == y {
                    if p1.y > y {
                        intersections.push(p0.x);
                    }
                    if p0.y > y {
                        intersections.push(p1.x);
                    }
                } else {
                    let dy = i64::from(p1.y) - i64::from(p0.y);
                    let dx = i64::from(p1.x) - i64::from(p0.x);
                    let fraction = (i64::from(y) - i64::from(p0.y)) as f64 / dy as f64;
                    let inter = p0.x as f64 + fraction * dx as f64;
                    intersections.push(round_f64_to_i32_saturating(inter));
                }
            }
        }

        intersections.sort_unstable();
        for range in intersections.chunks(2) {
            if range.len() < 2 {
                continue;
            }
            let mut from = range[0].min(image.width() as i32);
            let mut to = range[1].min(image.width() as i32 - 1);
            if from < image.width() as i32 && to >= 0 {
                from = from.max(0);
                to = to.max(0);
                for x in from..=to {
                    image.put_pixel(x as u32, y as u32, color);
                }
            }
        }

        intersections.clear();
    }

    for edge in edges {
        draw_line_segment_mut(
            image,
            (edge[0].x as f32, edge[0].y as f32),
            (edge[1].x as f32, edge[1].y as f32),
            color,
        );
    }
}

pub(crate) fn round_f64_to_i32_saturating(value: f64) -> i32 {
    if !value.is_finite() {
        0
    } else if value <= i32::MIN as f64 {
        i32::MIN
    } else if value >= i32::MAX as f64 {
        i32::MAX
    } else {
        value.round() as i32
    }
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
/// Internal fuzz harness entry point.
///
/// This is not a stable consumer API. The crate refuses non-fuzzing release
/// builds with the `fuzzing` feature enabled.
pub fn fuzz_draw_polygon_rgba(width: u32, height: u32, points: &[(i32, i32)], color: [u8; 4]) {
    let mut image = RgbaImage::new(width, height);
    let polygon: Vec<_> = points.iter().map(|&(x, y)| Point::new(x, y)).collect();
    draw_polygon_mut(&mut image, &polygon, Rgba(color));
}

pub(crate) fn interpolate(left: Rgba<u8>, right: Rgba<u8>, left_weight: f32) -> Rgba<u8> {
    let right_weight = 1.0 - left_weight;
    Rgba([
        weighted_channel_sum(left.0[0], right.0[0], left_weight, right_weight),
        weighted_channel_sum(left.0[1], right.0[1], left_weight, right_weight),
        weighted_channel_sum(left.0[2], right.0[2], left_weight, right_weight),
        weighted_channel_sum(left.0[3], right.0[3], left_weight, right_weight),
    ])
}

pub(crate) fn weighted_channel_sum(left: u8, right: u8, left_weight: f32, right_weight: f32) -> u8 {
    let total_weight = left_weight + right_weight;
    if !total_weight.is_finite() || total_weight <= 0.0 {
        return u8::MIN;
    }

    let value = (left as f32 * left_weight + right as f32 * right_weight) / total_weight;
    if value.is_finite() {
        value.clamp(u8::MIN as f32, u8::MAX as f32) as u8
    } else {
        u8::MIN
    }
}

pub(crate) fn draw_if_in_bounds(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if in_bounds(image, x, y) {
        image.put_pixel(x as u32, y as u32, color);
    }
}

pub(crate) fn in_bounds(image: &RgbaImage, x: i32, y: i32) -> bool {
    x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32
}
