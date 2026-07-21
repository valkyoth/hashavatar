use super::common::{FamilyRig, ellipse, eyes, line, polygon, smile, triangle};
use crate::{AvatarKind, CatError, geometry::Point, scene::Scene};

pub(super) fn compile(scene: &mut Scene, kind: AvatarKind, rig: FamilyRig) -> Result<(), CatError> {
    match kind {
        AvatarKind::Ghost => ghost(scene, rig),
        AvatarKind::Slime => slime(scene, rig),
        AvatarKind::Bird => bird(scene, rig),
        AvatarKind::Octopus => octopus(scene, rig),
        AvatarKind::Penguin => penguin(scene, rig),
        _ => Err(CatError::InvalidScene),
    }
}

fn ghost(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
    polygon(
        scene,
        rig,
        &[
            (27, 76),
            (27, 49),
            (32, 31),
            (50, 21),
            (68, 31),
            (73, 49),
            (73, 80),
            (65, 73),
            (58, 82),
            (50, 74),
            (42, 82),
            (35, 73),
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

fn slime(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
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

fn bird(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(57)?),
        rig.canvas.s(25)?,
        rig.canvas.s(29)?,
        rig.primary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(39)?, rig.canvas.y(60)?),
        rig.canvas.s(11)?,
        rig.canvas.s(18)?,
        rig.secondary,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(67)?, rig.canvas.y(53)?),
            Point::new(rig.canvas.x(84)?, rig.canvas.y(60)?),
            Point::new(rig.canvas.x(67)?, rig.canvas.y(65)?),
        ],
        rig.accent,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(59)?, rig.canvas.y(48)?),
        rig.canvas.s(5)?,
        rig.canvas.s(5)?,
        rig.light,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(60)?, rig.canvas.y(48)?),
        rig.canvas.s(2)?,
        rig.canvas.s(2)?,
        rig.ink,
    )
}

fn octopus(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
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

fn penguin(scene: &mut Scene, rig: FamilyRig) -> Result<(), CatError> {
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
    )
}
