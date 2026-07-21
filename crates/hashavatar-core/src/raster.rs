use alloc::vec::Vec;

use crate::{
    CatError, RGBA8_BYTES_PER_PIXEL,
    fixed::Fixed,
    scene::{Color, Command, Point, Scene, triangle_area},
};

/// Tightly packed canonical straight-alpha RGBA8 output.
#[must_use = "use or explicitly discard the rendered pixel buffer"]
pub struct CanonicalRgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl CanonicalRgbaImage {
    /// Returns the image width.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the image height.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the image dimensions.
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Borrows the tightly packed straight-alpha RGBA8 pixels.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Transfers ownership of the tightly packed RGBA8 pixels.
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }
}

pub(crate) fn render_scene(scene: &Scene) -> Result<CanonicalRgbaImage, CatError> {
    let report = scene.validate()?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(report.rgba_bytes())
        .map_err(|_| CatError::Allocation)?;
    pixels.resize(report.rgba_bytes(), 0);

    for command in scene.commands()? {
        match *command {
            Command::Empty => return Err(CatError::InvalidScene),
            Command::Fill(color) => fill(&mut pixels, color),
            Command::Ellipse {
                center,
                radius_x,
                radius_y,
                color,
            } => draw_ellipse(
                &mut pixels,
                scene.width(),
                scene.height(),
                center,
                radius_x,
                radius_y,
                color,
            )?,
            Command::Triangle { points, color } => {
                draw_triangle(&mut pixels, scene.width(), scene.height(), points, color)?
            }
        }
    }
    Ok(CanonicalRgbaImage {
        width: scene.width(),
        height: scene.height(),
        pixels,
    })
}

fn fill(pixels: &mut [u8], color: Color) {
    for pixel in pixels.chunks_exact_mut(RGBA8_BYTES_PER_PIXEL) {
        pixel.copy_from_slice(&color.rgba());
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ellipse(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    color: Color,
) -> Result<(), CatError> {
    let bounds = Bounds::for_ellipse(center, radius_x, radius_y, width, height)?;
    let radius_x_raw = i128::from(radius_x.raw());
    let radius_y_raw = i128::from(radius_y.raw());
    let radius_x_squared = radius_x_raw
        .checked_mul(radius_x_raw)
        .ok_or(CatError::NumericRange)?;
    let radius_y_squared = radius_y_raw
        .checked_mul(radius_y_raw)
        .ok_or(CatError::NumericRange)?;
    let limit = radius_x_squared
        .checked_mul(radius_y_squared)
        .ok_or(CatError::NumericRange)?;

    for y in bounds.min_y..bounds.max_y {
        let sample_y = Fixed::pixel_center(y)?;
        let dy = i128::from(sample_y.raw()) - i128::from(center.y.raw());
        let dy_term = dy
            .checked_mul(dy)
            .and_then(|value| value.checked_mul(radius_x_squared))
            .ok_or(CatError::NumericRange)?;
        for x in bounds.min_x..bounds.max_x {
            let sample_x = Fixed::pixel_center(x)?;
            let dx = i128::from(sample_x.raw()) - i128::from(center.x.raw());
            let value = dx
                .checked_mul(dx)
                .and_then(|number| number.checked_mul(radius_y_squared))
                .and_then(|number| number.checked_add(dy_term))
                .ok_or(CatError::NumericRange)?;
            if value <= limit {
                put_pixel(pixels, width, x, y, color)?;
            }
        }
    }
    Ok(())
}

fn draw_triangle(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    points: [Point; 3],
    color: Color,
) -> Result<(), CatError> {
    let bounds = Bounds::for_triangle(points, width, height)?;
    let winding = triangle_area(points);
    if winding == 0 {
        return Err(CatError::InvalidScene);
    }
    for y in bounds.min_y..bounds.max_y {
        let sample_y = Fixed::pixel_center(y)?;
        for x in bounds.min_x..bounds.max_x {
            let sample = Point::new(Fixed::pixel_center(x)?, sample_y);
            let first_point = points.first().ok_or(CatError::InvalidScene)?;
            let second_point = points.get(1).ok_or(CatError::InvalidScene)?;
            let third_point = points.get(2).ok_or(CatError::InvalidScene)?;
            let first = edge(*first_point, *second_point, sample);
            let second = edge(*second_point, *third_point, sample);
            let third = edge(*third_point, *first_point, sample);
            let inside = if winding > 0 {
                first >= 0 && second >= 0 && third >= 0
            } else {
                first <= 0 && second <= 0 && third <= 0
            };
            if inside {
                put_pixel(pixels, width, x, y, color)?;
            }
        }
    }
    Ok(())
}

fn edge(start: Point, end: Point, sample: Point) -> i128 {
    let edge_x = i128::from(end.x.raw()) - i128::from(start.x.raw());
    let edge_y = i128::from(end.y.raw()) - i128::from(start.y.raw());
    let sample_x = i128::from(sample.x.raw()) - i128::from(start.x.raw());
    let sample_y = i128::from(sample.y.raw()) - i128::from(start.y.raw());
    edge_x * sample_y - edge_y * sample_x
}

fn put_pixel(pixels: &mut [u8], width: u32, x: u32, y: u32, color: Color) -> Result<(), CatError> {
    let pixel_index = usize::try_from(y)
        .map_err(|_| CatError::NumericRange)?
        .checked_mul(usize::try_from(width).map_err(|_| CatError::NumericRange)?)
        .and_then(|value| value.checked_add(usize::try_from(x).ok()?))
        .ok_or(CatError::NumericRange)?;
    let offset = pixel_index
        .checked_mul(RGBA8_BYTES_PER_PIXEL)
        .ok_or(CatError::NumericRange)?;
    let end = offset
        .checked_add(RGBA8_BYTES_PER_PIXEL)
        .ok_or(CatError::NumericRange)?;
    let pixel = pixels.get_mut(offset..end).ok_or(CatError::InvalidScene)?;
    pixel.copy_from_slice(&color.rgba());
    Ok(())
}

struct Bounds {
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

impl Bounds {
    fn for_ellipse(
        center: Point,
        radius_x: Fixed,
        radius_y: Fixed,
        width: u32,
        height: u32,
    ) -> Result<Self, CatError> {
        Self::new(
            center.x.checked_sub(radius_x)?,
            center.y.checked_sub(radius_y)?,
            center.x.checked_add(radius_x)?,
            center.y.checked_add(radius_y)?,
            width,
            height,
        )
    }

    fn for_triangle(points: [Point; 3], width: u32, height: u32) -> Result<Self, CatError> {
        let first = points.first().ok_or(CatError::InvalidScene)?;
        let mut min_x = first.x;
        let mut max_x = first.x;
        let mut min_y = first.y;
        let mut max_y = first.y;
        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        Self::new(min_x, min_y, max_x, max_y, width, height)
    }

    fn new(
        min_x: Fixed,
        min_y: Fixed,
        max_x: Fixed,
        max_y: Fixed,
        width: u32,
        height: u32,
    ) -> Result<Self, CatError> {
        let width_i32 = i32::try_from(width).map_err(|_| CatError::NumericRange)?;
        let height_i32 = i32::try_from(height).map_err(|_| CatError::NumericRange)?;
        Ok(Self {
            min_x: u32::try_from(min_x.floor()?.clamp(0, width_i32))
                .map_err(|_| CatError::NumericRange)?,
            max_x: u32::try_from(max_x.ceil()?.clamp(0, width_i32))
                .map_err(|_| CatError::NumericRange)?,
            min_y: u32::try_from(min_y.floor()?.clamp(0, height_i32))
                .map_err(|_| CatError::NumericRange)?,
            max_y: u32::try_from(max_y.ceil()?.clamp(0, height_i32))
                .map_err(|_| CatError::NumericRange)?,
        })
    }
}
