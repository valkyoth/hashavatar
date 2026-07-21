use crate::{
    AvatarError,
    fixed::Fixed,
    geometry::{FillRule, Path, Point, Rect},
    paint::Paint,
    raster::SurfaceWriter,
    scene::{Clip, Scene, Stroke, triangle_area},
};

pub(crate) fn draw_rect(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    rect: Rect,
    paint: Paint,
) -> Result<(), AvatarError> {
    let bounds = Bounds::new(rect.left, rect.top, rect.right, rect.bottom, writer)?;
    visit_bounds(
        writer,
        bounds,
        |sample| {
            Ok(sample.x >= rect.left
                && sample.x < rect.right
                && sample.y >= rect.top
                && sample.y < rect.bottom)
        },
        paint,
    )
}

pub(crate) fn draw_ellipse(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    paint: Paint,
) -> Result<(), AvatarError> {
    let bounds = Bounds::new(
        center.x.checked_sub(radius_x)?,
        center.y.checked_sub(radius_y)?,
        center.x.checked_add(radius_x)?,
        center.y.checked_add(radius_y)?,
        writer,
    )?;
    let rx = i128::from(radius_x.raw());
    let ry = i128::from(radius_y.raw());
    let rx2 = rx.checked_mul(rx).ok_or(AvatarError::NumericRange)?;
    let ry2 = ry.checked_mul(ry).ok_or(AvatarError::NumericRange)?;
    let limit = rx2.checked_mul(ry2).ok_or(AvatarError::NumericRange)?;
    visit_bounds(
        writer,
        bounds,
        |sample| {
            let dx = i128::from(sample.x.raw()) - i128::from(center.x.raw());
            let dy = i128::from(sample.y.raw()) - i128::from(center.y.raw());
            let value = dx
                .checked_mul(dx)
                .and_then(|number| number.checked_mul(ry2))
                .and_then(|number| {
                    dy.checked_mul(dy)
                        .and_then(|term| term.checked_mul(rx2))
                        .and_then(|term| number.checked_add(term))
                })
                .ok_or(AvatarError::NumericRange)?;
            Ok(value <= limit)
        },
        paint,
    )
}

pub(crate) fn draw_triangle(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    points: [Point; 3],
    paint: Paint,
) -> Result<(), AvatarError> {
    let bounds = Bounds::for_points(&points, writer)?;
    let winding = triangle_area(points);
    visit_bounds(
        writer,
        bounds,
        |sample| {
            let a = *points.first().ok_or(AvatarError::InvalidScene)?;
            let b = *points.get(1).ok_or(AvatarError::InvalidScene)?;
            let c = *points.get(2).ok_or(AvatarError::InvalidScene)?;
            let edges = [edge(a, b, sample), edge(b, c, sample), edge(c, a, sample)];
            Ok(if winding > 0 {
                edges.iter().all(|value| *value >= 0)
            } else {
                edges.iter().all(|value| *value <= 0)
            })
        },
        paint,
    )
}

pub(crate) fn draw_line(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    start: Point,
    end: Point,
    stroke: Stroke,
) -> Result<(), AvatarError> {
    let half = stroke.width.checked_mul(Fixed::from_ratio(1, 2)?)?;
    let bounds = Bounds::new(
        start.x.min(end.x).checked_sub(half)?,
        start.y.min(end.y).checked_sub(half)?,
        start.x.max(end.x).checked_add(half)?,
        start.y.max(end.y).checked_add(half)?,
        writer,
    )?;
    visit_bounds(
        writer,
        bounds,
        |sample| line_contains(start, end, half, sample),
        stroke.paint,
    )
}

pub(crate) fn draw_path(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    path: &Path,
    fill_rule: FillRule,
    fill: Option<Paint>,
    stroke: Option<Stroke>,
) -> Result<(), AvatarError> {
    let points = path.points()?;
    if let Some(paint) = fill {
        let bounds = Bounds::for_points(points, writer)?;
        visit_bounds(
            writer,
            bounds,
            |sample| Ok(path_contains(points, sample, fill_rule)),
            paint,
        )?;
    }
    if let Some(stroke) = stroke {
        for pair in points.windows(2) {
            let start = *pair.first().ok_or(AvatarError::InvalidScene)?;
            let end = *pair.get(1).ok_or(AvatarError::InvalidScene)?;
            draw_line(writer, start, end, stroke)?;
        }
        if path.is_closed() {
            let start = *points.last().ok_or(AvatarError::InvalidScene)?;
            let end = *points.first().ok_or(AvatarError::InvalidScene)?;
            draw_line(writer, start, end, stroke)?;
        }
    }
    Ok(())
}

fn visit_bounds(
    writer: &mut SurfaceWriter<'_, '_, '_>,
    bounds: Bounds,
    mut contains: impl FnMut(Point) -> Result<bool, AvatarError>,
    paint: Paint,
) -> Result<(), AvatarError> {
    for y in bounds.min_y..bounds.max_y {
        let sample_y = Fixed::pixel_center(y)?;
        for x in bounds.min_x..bounds.max_x {
            let sample = Point::new(Fixed::pixel_center(x)?, sample_y);
            if contains(sample)? {
                writer.paint_pixel(x, y, paint.sample(sample)?)?;
            }
        }
    }
    Ok(())
}

fn line_contains(
    start: Point,
    end: Point,
    radius: Fixed,
    sample: Point,
) -> Result<bool, AvatarError> {
    let vx = i128::from(end.x.raw()) - i128::from(start.x.raw());
    let vy = i128::from(end.y.raw()) - i128::from(start.y.raw());
    let wx = i128::from(sample.x.raw()) - i128::from(start.x.raw());
    let wy = i128::from(sample.y.raw()) - i128::from(start.y.raw());
    let length_squared = vx
        .checked_mul(vx)
        .and_then(|value| value.checked_add(vy.checked_mul(vy)?))
        .ok_or(AvatarError::NumericRange)?;
    if length_squared == 0 {
        return Err(AvatarError::InvalidScene);
    }
    let projection = wx
        .checked_mul(vx)
        .and_then(|value| value.checked_add(wy.checked_mul(vy)?))
        .ok_or(AvatarError::NumericRange)?;
    let radius_squared = i128::from(radius.raw())
        .checked_mul(i128::from(radius.raw()))
        .ok_or(AvatarError::NumericRange)?;
    if projection <= 0 {
        return point_distance_squared(sample, start).map(|value| value <= radius_squared);
    }
    if projection >= length_squared {
        return point_distance_squared(sample, end).map(|value| value <= radius_squared);
    }
    let cross = vx
        .checked_mul(wy)
        .and_then(|value| value.checked_sub(vy.checked_mul(wx)?))
        .ok_or(AvatarError::NumericRange)?;
    cross
        .checked_mul(cross)
        .and_then(|value| {
            radius_squared
                .checked_mul(length_squared)
                .map(|limit| value <= limit)
        })
        .ok_or(AvatarError::NumericRange)
}

fn point_distance_squared(first: Point, second: Point) -> Result<i128, AvatarError> {
    let dx = i128::from(first.x.raw()) - i128::from(second.x.raw());
    let dy = i128::from(first.y.raw()) - i128::from(second.y.raw());
    dx.checked_mul(dx)
        .and_then(|value| value.checked_add(dy.checked_mul(dy)?))
        .ok_or(AvatarError::NumericRange)
}

pub(crate) fn clip_contains(scene: &Scene, clip: Clip, sample: Point) -> Result<bool, AvatarError> {
    match clip {
        Clip::Rect(rect) => Ok(sample.x >= rect.left
            && sample.x < rect.right
            && sample.y >= rect.top
            && sample.y < rect.bottom),
        Clip::Ellipse {
            center,
            radius_x,
            radius_y,
        } => ellipse_contains(center, radius_x, radius_y, sample),
        Clip::Path {
            path_index,
            fill_rule,
        } => Ok(path_contains(
            scene.path(path_index)?.points()?,
            sample,
            fill_rule,
        )),
    }
}

fn ellipse_contains(
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    sample: Point,
) -> Result<bool, AvatarError> {
    let rx = i128::from(radius_x.raw());
    let ry = i128::from(radius_y.raw());
    let rx2 = rx.checked_mul(rx).ok_or(AvatarError::NumericRange)?;
    let ry2 = ry.checked_mul(ry).ok_or(AvatarError::NumericRange)?;
    let dx = i128::from(sample.x.raw()) - i128::from(center.x.raw());
    let dy = i128::from(sample.y.raw()) - i128::from(center.y.raw());
    let value = dx
        .checked_mul(dx)
        .and_then(|number| number.checked_mul(ry2))
        .and_then(|number| {
            dy.checked_mul(dy)
                .and_then(|term| term.checked_mul(rx2))
                .and_then(|term| number.checked_add(term))
        })
        .ok_or(AvatarError::NumericRange)?;
    Ok(value <= rx2.checked_mul(ry2).ok_or(AvatarError::NumericRange)?)
}

fn path_contains(points: &[Point], sample: Point, fill_rule: FillRule) -> bool {
    let mut winding = 0_i32;
    for edge_points in path_edges(points) {
        let (start, end) = edge_points;
        let upward = start.y <= sample.y && end.y > sample.y && edge(start, end, sample) > 0;
        let downward = start.y > sample.y && end.y <= sample.y && edge(start, end, sample) < 0;
        if upward {
            winding = winding.saturating_add(1);
        } else if downward {
            winding = winding.saturating_sub(1);
        }
    }
    match fill_rule {
        FillRule::EvenOdd => winding.unsigned_abs() % 2 == 1,
        FillRule::NonZero => winding != 0,
    }
}

fn path_edges(points: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    points
        .windows(2)
        .filter_map(|pair| Some((*pair.first()?, *pair.get(1)?)))
        .chain(points.last().copied().zip(points.first().copied()))
}

fn edge(start: Point, end: Point, sample: Point) -> i128 {
    let edge_x = i128::from(end.x.raw()) - i128::from(start.x.raw());
    let edge_y = i128::from(end.y.raw()) - i128::from(start.y.raw());
    let sample_x = i128::from(sample.x.raw()) - i128::from(start.x.raw());
    let sample_y = i128::from(sample.y.raw()) - i128::from(start.y.raw());
    edge_x * sample_y - edge_y * sample_x
}

struct Bounds {
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
}

impl Bounds {
    fn for_points(
        points: &[Point],
        writer: &SurfaceWriter<'_, '_, '_>,
    ) -> Result<Self, AvatarError> {
        let first = points.first().ok_or(AvatarError::InvalidScene)?;
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
        Self::new(min_x, min_y, max_x, max_y, writer)
    }

    fn new(
        min_x: Fixed,
        min_y: Fixed,
        max_x: Fixed,
        max_y: Fixed,
        writer: &SurfaceWriter<'_, '_, '_>,
    ) -> Result<Self, AvatarError> {
        let width = i32::try_from(writer.width()).map_err(|_| AvatarError::NumericRange)?;
        let height = i32::try_from(writer.height()).map_err(|_| AvatarError::NumericRange)?;
        Ok(Self {
            min_x: u32::try_from(min_x.floor()?.clamp(0, width))
                .map_err(|_| AvatarError::NumericRange)?,
            max_x: u32::try_from(max_x.ceil()?.clamp(0, width))
                .map_err(|_| AvatarError::NumericRange)?,
            min_y: u32::try_from(min_y.floor()?.clamp(0, height))
                .map_err(|_| AvatarError::NumericRange)?,
            max_y: u32::try_from(max_y.ceil()?.clamp(0, height))
                .map_err(|_| AvatarError::NumericRange)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_length_line_is_rejected_without_division() -> Result<(), AvatarError> {
        let point = Point::new(Fixed::ZERO, Fixed::ZERO);
        assert_eq!(
            line_contains(point, point, Fixed::from_integer(1)?, point),
            Err(AvatarError::InvalidScene)
        );
        Ok(())
    }
}
