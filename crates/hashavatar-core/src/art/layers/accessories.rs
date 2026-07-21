use super::common::{LayerRig, ellipse, line, rect, triangle};
use crate::{
    AvatarAccessory, AvatarAnchorSet, AvatarColorRoles, CatError, geometry::Point, paint::Color,
    scene::Scene,
};

pub(super) fn compile(
    scene: &mut Scene,
    anchors: AvatarAnchorSet,
    colors: AvatarColorRoles,
    accessory: AvatarAccessory,
    adjustment: i16,
) -> Result<(), CatError> {
    let rig = LayerRig::new(scene, anchors, colors)?;
    match accessory {
        AvatarAccessory::Glasses => glasses(scene, rig, adjustment),
        AvatarAccessory::Hat => hat(scene, rig, adjustment),
        AvatarAccessory::Headphones => headphones(scene, rig, adjustment),
        AvatarAccessory::Crown => crown(scene, rig, adjustment),
        AvatarAccessory::Bowtie => bowtie(scene, rig, adjustment),
        AvatarAccessory::Eyepatch => eyepatch(scene, rig, adjustment),
        AvatarAccessory::Scarf => scarf(scene, rig, adjustment),
        AvatarAccessory::Halo => halo(scene, rig, adjustment),
        AvatarAccessory::Horns => horns(scene, rig, adjustment),
    }
}

fn glasses(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let left = rig.point(rig.anchors.left_eye(), adjustment)?;
    let right = rig.point(rig.anchors.right_eye(), adjustment)?;
    let outer = rig
        .eye_radius()?
        .checked_mul(crate::fixed::Fixed::from_integer(2)?)?;
    let inner = outer.checked_mul(crate::fixed::Fixed::from_ratio(3, 4)?)?;
    for center in [left, right] {
        ellipse(scene, center, outer, outer, rig.ink)?;
        ellipse(
            scene,
            center,
            inner,
            inner,
            Color::rgba(rig.light.red, rig.light.green, rig.light.blue, 150),
        )?;
    }
    line(scene, left, right, rig.size(100)?, rig.ink)
}

fn hat(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let top = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let body_half = half.checked_mul(crate::fixed::Fixed::from_ratio(7, 10)?)?;
    let height = rig.size(1_800)?;
    let brim = rig.size(450)?;
    rect(
        scene,
        top.x.checked_sub(body_half)?,
        top.y.checked_sub(height)?,
        top.x.checked_add(body_half)?,
        top.y,
        rig.accent,
    )?;
    rect(
        scene,
        top.x.checked_sub(half)?,
        top.y.checked_sub(brim)?,
        top.x.checked_add(half)?,
        top.y.checked_add(brim)?,
        rig.ink,
    )
}

fn headphones(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let left_eye = rig.point(rig.anchors.left_eye(), adjustment)?;
    let right_eye = rig.point(rig.anchors.right_eye(), adjustment)?;
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let left = Point::new(crown.x.checked_sub(half)?, left_eye.y);
    let right = Point::new(crown.x.checked_add(half)?, right_eye.y);
    line(scene, left, crown, rig.size(180)?, rig.ink)?;
    line(scene, crown, right, rig.size(180)?, rig.ink)?;
    for center in [left, right] {
        ellipse(scene, center, rig.size(550)?, rig.size(850)?, rig.accent)?;
    }
    Ok(())
}

fn crown(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let top = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig
        .face_half()?
        .checked_mul(crate::fixed::Fixed::from_ratio(4, 5)?)?;
    let rise = rig.size(1_500)?;
    let base = top.y.checked_add(rig.size(900)?)?;
    for offset in [-1_i32, 0, 1] {
        let center = top
            .x
            .checked_add(half.checked_mul(crate::fixed::Fixed::from_ratio(offset, 2)?)?)?;
        triangle(
            scene,
            [
                Point::new(
                    center
                        .checked_sub(half.checked_mul(crate::fixed::Fixed::from_ratio(1, 2)?)?)?,
                    base,
                ),
                Point::new(center, top.y.checked_sub(rise)?),
                Point::new(
                    center
                        .checked_add(half.checked_mul(crate::fixed::Fixed::from_ratio(1, 2)?)?)?,
                    base,
                ),
            ],
            rig.accent,
        )?;
    }
    rect(
        scene,
        top.x.checked_sub(half)?,
        base.checked_sub(rig.size(350)?)?,
        top.x.checked_add(half)?,
        base.checked_add(rig.size(350)?)?,
        rig.ink,
    )
}

fn bowtie(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let neck = rig.point(rig.anchors.neck(), adjustment)?;
    let width = rig.size(1_250)?;
    let height = rig.size(850)?;
    triangle(
        scene,
        [
            neck,
            Point::new(neck.x.checked_sub(width)?, neck.y.checked_sub(height)?),
            Point::new(neck.x.checked_sub(width)?, neck.y.checked_add(height)?),
        ],
        rig.accent,
    )?;
    triangle(
        scene,
        [
            neck,
            Point::new(neck.x.checked_add(width)?, neck.y.checked_sub(height)?),
            Point::new(neck.x.checked_add(width)?, neck.y.checked_add(height)?),
        ],
        rig.accent,
    )?;
    ellipse(scene, neck, rig.size(350)?, rig.size(350)?, rig.ink)
}

fn eyepatch(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let left = rig.point(rig.anchors.left_eye(), adjustment)?;
    let right = rig.point(rig.anchors.right_eye(), adjustment)?;
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    line(
        scene,
        Point::new(crown.x.checked_sub(rig.face_half()?)?, crown.y),
        right,
        rig.size(140)?,
        rig.ink,
    )?;
    ellipse(
        scene,
        left,
        rig.eye_radius()?
            .checked_mul(crate::fixed::Fixed::from_integer(2)?)?,
        rig.eye_radius()?
            .checked_mul(crate::fixed::Fixed::from_ratio(3, 2)?)?,
        rig.ink,
    )
}

fn scarf(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let neck = rig.point(rig.anchors.neck(), adjustment)?;
    let half = rig
        .face_half()?
        .checked_mul(crate::fixed::Fixed::from_ratio(4, 5)?)?;
    let height = rig.size(700)?;
    rect(
        scene,
        neck.x.checked_sub(half)?,
        neck.y,
        neck.x.checked_add(half)?,
        neck.y.checked_add(height)?,
        rig.accent,
    )?;
    rect(
        scene,
        neck.x,
        neck.y.checked_add(height)?,
        neck.x.checked_add(rig.size(700)?)?,
        neck.y.checked_add(rig.size(2_500)?)?,
        rig.accent,
    )
}

fn halo(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    ellipse(
        scene,
        Point::new(crown.x, crown.y.checked_sub(rig.size(900)?)?),
        rig.face_half()?
            .checked_mul(crate::fixed::Fixed::from_ratio(4, 5)?)?,
        rig.size(500)?,
        Color::rgba(rig.accent.red, rig.accent.green, rig.accent.blue, 120),
    )
}

fn horns(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), CatError> {
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let rise = rig.size(1_800)?;
    for direction in [-1_i32, 1] {
        let side = half.checked_mul(crate::fixed::Fixed::from_ratio(direction, 1)?)?;
        let inner = half.checked_mul(crate::fixed::Fixed::from_ratio(direction, 2)?)?;
        triangle(
            scene,
            [
                Point::new(
                    crown.x.checked_add(side)?,
                    crown.y.checked_add(rig.size(700)?)?,
                ),
                Point::new(crown.x.checked_add(side)?, crown.y.checked_sub(rise)?),
                Point::new(crown.x.checked_add(inner)?, crown.y),
            ],
            rig.accent,
        )?;
    }
    Ok(())
}
