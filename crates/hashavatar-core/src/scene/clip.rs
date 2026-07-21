use super::{Clip, Scene, validate_point, validate_rect};
use crate::{AvatarError, fixed::Fixed, geometry::Rect};

pub(super) fn validate_clip(
    clip: Clip,
    scene: &Scene,
    minimum: Fixed,
    maximum: Fixed,
) -> Result<(), AvatarError> {
    match clip {
        Clip::Rect(rect) => validate_rect(rect, minimum, maximum),
        Clip::Ellipse {
            center,
            radius_x,
            radius_y,
        } => {
            if radius_x <= Fixed::ZERO || radius_y <= Fixed::ZERO {
                return Err(AvatarError::InvalidScene);
            }
            validate_rect(
                Rect::new(
                    center.x.checked_sub(radius_x)?,
                    center.y.checked_sub(radius_y)?,
                    center.x.checked_add(radius_x)?,
                    center.y.checked_add(radius_y)?,
                ),
                minimum,
                maximum,
            )
        }
        Clip::Path { path_index, .. } => {
            let path = scene.path(path_index)?;
            if !path.is_closed() {
                return Err(AvatarError::InvalidScene);
            }
            for point in path.points()? {
                validate_point(*point, minimum, maximum)?;
            }
            Ok(())
        }
    }
}

pub(super) fn clip_test_cost(clip: Clip, scene: &Scene) -> Result<u64, AvatarError> {
    match clip {
        Clip::Rect(_) | Clip::Ellipse { .. } => Ok(1),
        Clip::Path { path_index, .. } => u64::try_from(scene.path(path_index)?.point_count())
            .map_err(|_| AvatarError::NumericRange),
    }
}
