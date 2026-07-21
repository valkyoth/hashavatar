use super::common::{LayerRig, curved_line, ellipse, line, outline_ellipse, rect};
use crate::{
    AvatarAnchorSet, AvatarColorRoles, AvatarError, AvatarExpression, AvatarKind, geometry::Point,
    paint::Color, scene::Scene,
};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    anchors: AvatarAnchorSet,
    colors: AvatarColorRoles,
    expression: AvatarExpression,
) -> Result<(), AvatarError> {
    if matches!(expression, AvatarExpression::Default) {
        return Ok(());
    }
    let rig = LayerRig::new(scene, anchors, colors)?;
    let left = rig.point(anchors.left_eye(), 0)?;
    let right = rig.point(anchors.right_eye(), 0)?;
    let mouth = rig.point(anchors.mouth(), 0)?;
    let eye = rig.eye_radius()?;
    let mouth_color = if matches!(kind, AvatarKind::Ninja) {
        rig.light
    } else {
        rig.ink
    };
    match expression {
        AvatarExpression::Default => Ok(()),
        AvatarExpression::Happy => mouth_curve(scene, rig, mouth, false, mouth_color),
        AvatarExpression::Grumpy => mouth_curve(
            scene,
            rig,
            Point::new(mouth.x, mouth.y.checked_add(rig.size(700)?)?),
            true,
            mouth_color,
        ),
        AvatarExpression::Surprised => outline_ellipse(
            scene,
            mouth,
            rig.size(700)?,
            rig.size(700)?,
            rig.size(180)?,
            mouth_color,
        ),
        AvatarExpression::Sleepy => {
            for center in [left, right] {
                line(
                    scene,
                    Point::new(center.x.checked_sub(eye)?, center.y),
                    Point::new(center.x.checked_add(eye)?, center.y),
                    rig.size(180)?,
                    rig.ink,
                )?;
            }
            Ok(())
        }
        AvatarExpression::Winking => {
            line(
                scene,
                Point::new(left.x.checked_sub(eye)?, left.y),
                Point::new(left.x.checked_add(eye)?, left.y),
                rig.size(180)?,
                rig.ink,
            )?;
            ellipse(scene, right, eye, eye, rig.light)?;
            ellipse(
                scene,
                right,
                eye.checked_mul(crate::fixed::Fixed::from_ratio(2, 5)?)?,
                eye.checked_mul(crate::fixed::Fixed::from_ratio(2, 5)?)?,
                rig.ink,
            )
        }
        AvatarExpression::Cool => {
            let lens_x = eye.checked_mul(crate::fixed::Fixed::from_integer(2)?)?;
            for center in [left, right] {
                rect(
                    scene,
                    center.x.checked_sub(lens_x)?,
                    center.y.checked_sub(eye)?,
                    center.x.checked_add(lens_x)?,
                    center.y.checked_add(eye)?,
                    rig.ink,
                )?;
            }
            line(scene, left, right, rig.size(180)?, rig.ink)
        }
        AvatarExpression::Crying => {
            mouth_curve(
                scene,
                rig,
                Point::new(mouth.x, mouth.y.checked_add(rig.size(700)?)?),
                true,
                mouth_color,
            )?;
            ellipse(
                scene,
                Point::new(
                    right.x.checked_add(eye)?,
                    right.y.checked_add(rig.size(900)?)?,
                ),
                rig.size(350)?,
                rig.size(700)?,
                rig.accent,
            )
        }
    }
}

fn mouth_curve(
    scene: &mut Scene,
    rig: LayerRig,
    mouth: Point,
    inverted: bool,
    color: Color,
) -> Result<(), AvatarError> {
    let half = rig.size(1_200)?;
    let bend = rig.size(700)?;
    curved_line(
        scene,
        Point::new(mouth.x.checked_sub(half)?, mouth.y),
        Point::new(
            mouth.x,
            if inverted {
                mouth.y.checked_sub(bend)?
            } else {
                mouth.y.checked_add(bend)?
            },
        ),
        Point::new(mouth.x.checked_add(half)?, mouth.y),
        rig.size(140)?,
        color,
    )
}
