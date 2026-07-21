use super::common::{LayerRig, curved_line, ellipse, line, rect};
use crate::{
    AvatarAnchorSet, AvatarColorRoles, AvatarExpression, CatError, geometry::Point, scene::Scene,
};

pub(super) fn compile(
    scene: &mut Scene,
    anchors: AvatarAnchorSet,
    colors: AvatarColorRoles,
    expression: AvatarExpression,
) -> Result<(), CatError> {
    if matches!(expression, AvatarExpression::Default) {
        return Ok(());
    }
    let rig = LayerRig::new(scene, anchors, colors)?;
    let left = rig.point(anchors.left_eye(), 0)?;
    let right = rig.point(anchors.right_eye(), 0)?;
    let mouth = rig.point(anchors.mouth(), 0)?;
    let eye = rig.eye_radius()?;
    match expression {
        AvatarExpression::Default => Ok(()),
        AvatarExpression::Happy => mouth_curve(scene, rig, mouth, false),
        AvatarExpression::Grumpy => mouth_curve(scene, rig, mouth, true),
        AvatarExpression::Surprised => {
            ellipse(scene, mouth, rig.size(650)?, rig.size(850)?, rig.ink)
        }
        AvatarExpression::Sleepy => {
            for center in [left, right] {
                line(
                    scene,
                    Point::new(center.x.checked_sub(eye)?, center.y),
                    Point::new(center.x.checked_add(eye)?, center.y),
                    rig.size(260)?,
                    rig.ink,
                )?;
            }
            Ok(())
        }
        AvatarExpression::Winking => line(
            scene,
            Point::new(left.x.checked_sub(eye)?, left.y),
            Point::new(left.x.checked_add(eye)?, left.y),
            rig.size(260)?,
            rig.ink,
        ),
        AvatarExpression::Cool => rect(
            scene,
            left.x
                .checked_sub(eye.checked_mul(crate::fixed::Fixed::from_integer(2)?)?)?,
            left.y.checked_sub(eye)?,
            right
                .x
                .checked_add(eye.checked_mul(crate::fixed::Fixed::from_integer(2)?)?)?,
            right.y.checked_add(eye)?,
            rig.ink,
        ),
        AvatarExpression::Crying => {
            mouth_curve(scene, rig, mouth, true)?;
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
) -> Result<(), CatError> {
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
        rig.ink,
    )
}
