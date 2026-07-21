use crate::{
    AvatarKind, CatError,
    geometry::{Point, Rect},
    scene::Scene,
};

use super::super::common::{FamilyRig, ellipse, eyes, line, rect};

pub(super) fn compile(scene: &mut Scene, kind: AvatarKind, rig: FamilyRig) -> Result<(), CatError> {
    match kind {
        AvatarKind::Ninja => ninja(scene, rig),
        AvatarKind::Astronaut => astronaut(scene, rig),
        _ => Err(CatError::InvalidScene),
    }
}

fn ninja(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(55)?),
        rig.head_rx()?,
        rig.head_ry()?,
        rig.ink,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(25)?,
            rig.canvas.y(43)?,
            rig.canvas.x(75)?,
            rig.canvas.y(59)?,
        ),
        rig.primary,
    )?;
    eyes(scene, rig, 51, 11, 3)?;
    line(
        scene,
        Point::new(rig.canvas.x(71)?, rig.canvas.y(44)?),
        Point::new(rig.canvas.x(86)?, rig.canvas.y(36)?),
        rig.canvas.s(4)?,
        rig.primary,
    )
}

fn astronaut(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
    for (radius, color) in [(31, rig.light), (25, rig.ink), (19, rig.primary)] {
        ellipse(
            scene,
            Point::new(
                rig.canvas.x(50)?,
                rig.canvas.y(if radius == 31 { 53 } else { 52 })?,
            ),
            rig.canvas.s(radius)?,
            rig.canvas.s(radius + if radius == 31 { 3 } else { 0 })?,
            color,
        )?;
    }
    eyes(scene, rig, 51, 9, 3)?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(38)?,
            rig.canvas.y(79)?,
            rig.canvas.x(62)?,
            rig.canvas.y(87)?,
        ),
        rig.accent,
    )
}
