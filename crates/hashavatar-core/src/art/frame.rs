use super::util::Canvas;
use crate::{
    AvatarError, AvatarShape,
    geometry::{FillRule, Path, Point},
    paint::{Color, Paint},
    scene::{Clip, Command, Scene},
};

pub(super) struct Frame(bool);

pub(super) fn begin(scene: &mut Scene, shape: AvatarShape) -> Result<Frame, AvatarError> {
    let canvas = Canvas::new(scene)?;
    scene.push(Command::Fill(Paint::solid(Color::TRANSPARENT)))?;
    let clip = match shape {
        AvatarShape::Square => return Ok(Frame(false)),
        AvatarShape::Circle => Clip::Ellipse {
            center: canvas.center,
            radius_x: canvas.s(49)?,
            radius_y: canvas.s(49)?,
        },
        AvatarShape::Squircle => Clip::Path {
            path_index: push_squircle(scene, canvas)?,
            fill_rule: FillRule::NonZero,
        },
        AvatarShape::Hexagon => Clip::Path {
            path_index: push_polygon(
                scene,
                canvas,
                &[(50, 2), (94, 25), (94, 75), (50, 98), (6, 75), (6, 25)],
            )?,
            fill_rule: FillRule::NonZero,
        },
        AvatarShape::Octagon => Clip::Path {
            path_index: push_polygon(
                scene,
                canvas,
                &[
                    (30, 2),
                    (70, 2),
                    (98, 30),
                    (98, 70),
                    (70, 98),
                    (30, 98),
                    (2, 70),
                    (2, 30),
                ],
            )?,
            fill_rule: FillRule::NonZero,
        },
    };
    scene.push(Command::PushClip(clip))?;
    Ok(Frame(true))
}

pub(super) fn finish(scene: &mut Scene, frame: Frame) -> Result<(), AvatarError> {
    if frame.0 {
        scene.push(Command::PopClip)?;
    }
    Ok(())
}

fn push_squircle(scene: &mut Scene, canvas: Canvas) -> Result<u8, AvatarError> {
    let left = canvas.x(3)?;
    let right = canvas.x(97)?;
    let top = canvas.y(3)?;
    let bottom = canvas.y(97)?;
    let radius_x = canvas.x(18)?;
    let radius_y = canvas.y(18)?;
    let mut path = Path::builder(Point::new(left.checked_add(radius_x)?, top))?;
    path.line_to(Point::new(right.checked_sub(radius_x)?, top))?;
    path.quad_to(
        Point::new(right, top),
        Point::new(right, top.checked_add(radius_y)?),
    )?;
    path.line_to(Point::new(right, bottom.checked_sub(radius_y)?))?;
    path.quad_to(
        Point::new(right, bottom),
        Point::new(right.checked_sub(radius_x)?, bottom),
    )?;
    path.line_to(Point::new(left.checked_add(radius_x)?, bottom))?;
    path.quad_to(
        Point::new(left, bottom),
        Point::new(left, bottom.checked_sub(radius_y)?),
    )?;
    path.line_to(Point::new(left, top.checked_add(radius_y)?))?;
    path.quad_to(
        Point::new(left, top),
        Point::new(left.checked_add(radius_x)?, top),
    )?;
    scene.push_path(path.finish(true)?)
}

fn push_polygon(
    scene: &mut Scene,
    canvas: Canvas,
    points: &[(i32, i32)],
) -> Result<u8, AvatarError> {
    let first = points.first().ok_or(AvatarError::InvalidScene)?;
    let mut path = Path::builder(Point::new(canvas.x(first.0)?, canvas.y(first.1)?))?;
    for point in points.iter().skip(1) {
        path.line_to(Point::new(canvas.x(point.0)?, canvas.y(point.1)?))?;
    }
    scene.push_path(path.finish(true)?)
}
