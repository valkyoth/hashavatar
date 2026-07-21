use super::{Command, Scene};
use crate::{
    CatError,
    fixed::Fixed,
    geometry::{Point, Rect},
};

pub(super) fn command_work(command: Command, scene: &Scene) -> Result<u64, CatError> {
    match command {
        Command::Empty => Err(CatError::InvalidScene),
        Command::Fill(_) => full_work(scene),
        Command::Rect { rect, .. } => rect_work(rect, scene),
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            ..
        } => rect_work(
            Rect::new(
                center.x.checked_sub(radius_x)?,
                center.y.checked_sub(radius_y)?,
                center.x.checked_add(radius_x)?,
                center.y.checked_add(radius_y)?,
            ),
            scene,
        ),
        Command::Triangle { points, .. } => points_work(&points, scene),
        Command::Line { .. } => full_work(scene),
        Command::Path { path_index, .. } => {
            let path = scene.path(path_index)?;
            full_work(scene)?
                .checked_mul(
                    u64::try_from(path.point_count())
                        .map_err(|_| CatError::NumericRange)?
                        .saturating_add(1),
                )
                .ok_or(CatError::NumericRange)
        }
        Command::PushClip(_) | Command::PopClip | Command::PushOpacity(_) | Command::PopOpacity => {
            Ok(0)
        }
    }
}

fn full_work(scene: &Scene) -> Result<u64, CatError> {
    u64::from(scene.width)
        .checked_mul(u64::from(scene.height))
        .ok_or(CatError::NumericRange)
}

fn points_work(points: &[Point], scene: &Scene) -> Result<u64, CatError> {
    let first = points.first().ok_or(CatError::InvalidScene)?;
    let mut rect = Rect::new(first.x, first.y, first.x, first.y);
    for point in points.iter().skip(1) {
        rect.left = rect.left.min(point.x);
        rect.top = rect.top.min(point.y);
        rect.right = rect.right.max(point.x);
        rect.bottom = rect.bottom.max(point.y);
    }
    if rect.left == rect.right {
        rect.right = rect.right.checked_add(Fixed::from_integer(1)?)?;
    }
    if rect.top == rect.bottom {
        rect.bottom = rect.bottom.checked_add(Fixed::from_integer(1)?)?;
    }
    rect_work(rect, scene)
}

fn rect_work(rect: Rect, scene: &Scene) -> Result<u64, CatError> {
    let width = i32::try_from(scene.width).map_err(|_| CatError::NumericRange)?;
    let height = i32::try_from(scene.height).map_err(|_| CatError::NumericRange)?;
    let span_x = rect
        .right
        .ceil()?
        .clamp(0, width)
        .saturating_sub(rect.left.floor()?.clamp(0, width));
    let span_y = rect
        .bottom
        .ceil()?
        .clamp(0, height)
        .saturating_sub(rect.top.floor()?.clamp(0, height));
    u64::try_from(span_x)
        .ok()
        .and_then(|x| u64::try_from(span_y).ok().and_then(|y| x.checked_mul(y)))
        .ok_or(CatError::NumericRange)
}
