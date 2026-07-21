use super::common::{
    LayerRig, curved_line, ellipse, line, outline_ellipse, polygon, rect, triangle,
};
use crate::{
    AvatarAccessory, AvatarAnchorSet, AvatarColorRoles, AvatarError, geometry::Point, scene::Scene,
};

pub(super) fn compile(
    scene: &mut Scene,
    anchors: AvatarAnchorSet,
    colors: AvatarColorRoles,
    accessory: AvatarAccessory,
    adjustment: i16,
) -> Result<(), AvatarError> {
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

fn glasses(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let left = rig.point(rig.anchors.left_eye(), adjustment)?;
    let right = rig.point(rig.anchors.right_eye(), adjustment)?;
    let radius = rig
        .eye_radius()?
        .checked_mul(crate::fixed::Fixed::from_integer(2)?)?;
    for center in [left, right] {
        outline_ellipse(scene, center, radius, radius, rig.size(180)?, rig.ink)?;
    }
    line(
        scene,
        Point::new(left.x.checked_add(radius)?, left.y),
        Point::new(right.x.checked_sub(radius)?, right.y),
        rig.size(180)?,
        rig.ink,
    )
}

fn hat(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let top = rig.point(rig.anchors.crown(), adjustment)?;
    let brim_width = rig
        .face_half()?
        .checked_mul(crate::fixed::Fixed::from_ratio(14, 10)?)?;
    let brim_half = brim_width.checked_mul(crate::fixed::Fixed::from_ratio(1, 2)?)?;
    let body_half = brim_width.checked_mul(crate::fixed::Fixed::from_ratio(7, 20)?)?;
    let height = rig.size(1_800)?;
    let brim_height = rig.size(500)?;
    let body_top = top
        .y
        .checked_sub(height.checked_mul(crate::fixed::Fixed::from_ratio(1, 2)?)?)?;
    let body_bottom = body_top.checked_add(height)?;
    rect(
        scene,
        top.x.checked_sub(body_half)?,
        body_top,
        top.x.checked_add(body_half)?,
        body_bottom,
        rig.accent,
    )?;
    rect(
        scene,
        top.x.checked_sub(brim_half)?,
        body_bottom,
        top.x.checked_add(brim_half)?,
        body_bottom.checked_add(brim_height)?,
        rig.ink,
    )
}

fn headphones(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let left_eye = rig.point(rig.anchors.left_eye(), adjustment)?;
    let right_eye = rig.point(rig.anchors.right_eye(), adjustment)?;
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let eye_y = left_eye
        .y
        .checked_add(right_eye.y)?
        .checked_mul(crate::fixed::Fixed::from_ratio(1, 2)?)?;
    let left = Point::new(crown.x.checked_sub(half)?, eye_y);
    let right = Point::new(crown.x.checked_add(half)?, eye_y);
    let control_y = crown.y.checked_add(crown.y)?.checked_sub(eye_y)?;
    curved_line(
        scene,
        left,
        Point::new(crown.x, control_y),
        right,
        rig.size(180)?,
        rig.ink,
    )?;
    for center in [left, right] {
        ellipse(scene, center, rig.size(700)?, rig.size(1_000)?, rig.ink)?;
        ellipse(scene, center, rig.size(480)?, rig.size(780)?, rig.accent)?;
    }
    Ok(())
}

fn crown(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let top = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let x = |numerator: i32| {
        half.checked_mul(crate::fixed::Fixed::from_ratio(numerator, 100)?)
            .and_then(|offset| top.x.checked_add(offset))
    };
    let points = [
        Point::new(x(-70)?, top.y.checked_add(rig.size(1_200)?)?),
        Point::new(x(-45)?, top.y),
        Point::new(x(-16)?, top.y.checked_add(rig.size(1_000)?)?),
        Point::new(top.x, top.y.checked_sub(rig.size(400)?)?),
        Point::new(x(16)?, top.y.checked_add(rig.size(1_000)?)?),
        Point::new(x(45)?, top.y),
        Point::new(x(70)?, top.y.checked_add(rig.size(1_200)?)?),
    ];
    polygon(scene, &points, rig.accent)?;
    line(scene, points[0], points[6], rig.size(180)?, rig.ink)
}

fn bowtie(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let neck = rig.point(rig.anchors.neck(), adjustment)?;
    let width = rig.size(2_000)?;
    let height = rig.size(1_000)?;
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

fn eyepatch(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let left = rig.point(rig.anchors.left_eye(), adjustment)?;
    let mouth = rig.point(rig.anchors.mouth(), adjustment)?;
    let crown = rig.point(rig.anchors.crown(), adjustment / 2)?;
    let half = rig.face_half()?;
    line(
        scene,
        Point::new(crown.x.checked_sub(half)?, crown.y),
        Point::new(
            crown
                .x
                .checked_add(half.checked_mul(crate::fixed::Fixed::from_ratio(7, 10)?)?)?,
            mouth.y.checked_sub(rig.eye_radius()?)?,
        ),
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

fn scarf(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
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

fn halo(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    outline_ellipse(
        scene,
        Point::new(crown.x, crown.y.checked_sub(rig.size(700)?)?),
        rig.face_half()?
            .checked_mul(crate::fixed::Fixed::from_ratio(7, 10)?)?,
        rig.size(700)?,
        rig.size(220)?,
        rig.accent,
    )
}

fn horns(scene: &mut Scene, rig: LayerRig, adjustment: i16) -> Result<(), AvatarError> {
    let crown = rig.point(rig.anchors.crown(), adjustment)?;
    let half = rig.face_half()?;
    let y = crown.y.checked_add(rig.size(500)?)?;
    for direction in [-1_i32, 1] {
        let offset = |numerator: i32| {
            half.checked_mul(crate::fixed::Fixed::from_ratio(direction * numerator, 10)?)
                .and_then(|value| crown.x.checked_add(value))
        };
        triangle(
            scene,
            [
                Point::new(offset(6)?, y.checked_add(rig.size(700)?)?),
                Point::new(offset(10)?, y.checked_sub(rig.size(1_200)?)?),
                Point::new(offset(3)?, y.checked_add(rig.size(200)?)?),
            ],
            rig.accent,
        )?;
    }
    Ok(())
}
