use super::common::{FamilyRig, ellipse, eyes, line, polygon, smile, triangle};
use crate::{AvatarError, AvatarKind, geometry::Point, scene::Scene};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Ghost => ghost(scene, rig),
        AvatarKind::Slime => slime(scene, rig),
        AvatarKind::Bird => bird(scene, rig),
        AvatarKind::Octopus => octopus(scene, rig),
        AvatarKind::Penguin => penguin(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn ghost(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(49)?),
        rig.canvas.s(23)?,
        rig.canvas.s(27)?,
        rig.light,
    )?;
    polygon(
        scene,
        rig,
        &[
            (27, 48),
            (73, 48),
            (73, 76),
            (66, 72),
            (61, 81),
            (53, 74),
            (46, 82),
            (39, 74),
            (33, 81),
            (27, 76),
        ],
        rig.light,
    )?;
    eyes(scene, rig, 50, 10, 4)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        rig.canvas.s(4)?,
        rig.canvas.s(6)?,
        rig.ink,
    )
}

fn slime(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    polygon(
        scene,
        rig,
        &[
            (24, 76),
            (27, 50),
            (35, 33),
            (50, 26),
            (65, 33),
            (73, 50),
            (76, 76),
            (66, 72),
            (58, 78),
            (50, 72),
            (42, 78),
            (34, 72),
        ],
        rig.primary,
    )?;
    eyes(scene, rig, 53, 10, 3)?;
    smile(scene, rig, 65)
}

fn bird(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(57)?),
        rig.canvas.s(23)?,
        rig.canvas.s(23)?,
        rig.primary,
    )?;
    for x in [34, 66] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(62)?),
            rig.canvas.s(8)?,
            rig.canvas.s(13)?,
            rig.secondary,
        )?;
    }
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(46)?, rig.canvas.y(58)?),
            Point::new(rig.canvas.x(54)?, rig.canvas.y(58)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        ],
        rig.accent,
    )?;
    eyes(scene, rig, 49, 8, 3)
}

fn octopus(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(47)?),
        rig.canvas.s(25)?,
        rig.canvas.s(24)?,
        rig.primary,
    )?;
    for x in [31, 43, 57, 69] {
        line(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(63)?),
            Point::new(rig.canvas.x(x - 4 + (x % 3) * 4)?, rig.canvas.y(82)?),
            rig.canvas.s(6)?,
            rig.primary,
        )?;
    }
    eyes(scene, rig, 48, 10, 3)?;
    smile(scene, rig, 60)
}

fn penguin(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(55)?),
        rig.canvas.s(25)?,
        rig.canvas.s(34)?,
        rig.ink,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(62)?),
        rig.canvas.s(17)?,
        rig.canvas.s(24)?,
        rig.light,
    )?;
    for x in [25, 75] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(61)?),
            rig.canvas.s(7)?,
            rig.canvas.s(18)?,
            rig.ink,
        )?;
    }
    for x in [27, 73] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(50)?, rig.canvas.y(48)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(57)?),
                Point::new(
                    rig.canvas.x(if x < 50 { 31 } else { 69 })?,
                    rig.canvas.y(73)?,
                ),
            ],
            rig.ink,
        )?;
    }
    eyes(scene, rig, 43, 9, 3)?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(45)?, rig.canvas.y(54)?),
            Point::new(rig.canvas.x(55)?, rig.canvas.y(54)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(61)?),
        ],
        rig.accent,
    )?;
    for x in [41, 59] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(87)?),
            rig.canvas.s(6)?,
            rig.canvas.s(3)?,
            rig.accent,
        )?;
    }
    Ok(())
}
