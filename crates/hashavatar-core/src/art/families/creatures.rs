use super::common::{FamilyRig, ellipse, eyes, polygon, rect, smile, triangle};
use crate::{
    AvatarError, AvatarKind,
    geometry::{Point, Rect},
    scene::Scene,
};

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
    for x in [23, 77] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(59)?),
            rig.canvas.s(7)?,
            rig.canvas.s(4)?,
            rig.light.with_opacity(215),
        )?;
    }
    eyes(scene, rig, 50, 6, 4)?;
    if rig.draws_default_mouth() {
        ellipse(
            scene,
            Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
            rig.canvas.s(4)?,
            rig.canvas.s(6)?,
            rig.ink,
        )
    } else {
        Ok(())
    }
}

fn slime(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(54)?),
        rig.canvas.s(24)?,
        rig.canvas.s(21)?,
        rig.primary,
    )?;
    for (index, x) in [34, 45, 56, 67].iter().copied().enumerate() {
        let extra = i32::from((rig.traits.pattern_seed() >> (index * 3)) & 3);
        let bottom = 70 + extra * 3;
        rect(
            scene,
            Rect::new(
                rig.canvas.x(x - 4)?,
                rig.canvas.y(54)?,
                rig.canvas.x(x + 4)?,
                rig.canvas.y(bottom)?,
            ),
            rig.primary,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(bottom)?),
            rig.canvas.s(4)?,
            rig.canvas.s(4)?,
            rig.primary,
        )?;
    }
    for (x, y, size) in [(39, 44, 3), (56, 38, 2), (63, 55, 3)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(size)?,
            rig.canvas.s(size)?,
            rig.light.with_opacity(100),
        )?;
    }
    eyes(scene, rig, 48, 10, 3)?;
    smile(scene, rig, 64)
}

fn bird(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(57)?),
        rig.canvas.s(23)?,
        rig.canvas.s(23)?,
        rig.primary,
    )?;
    for x in [43, 50, 57] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 4)?, rig.canvas.y(38)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(25)?),
                Point::new(rig.canvas.x(x + 4)?, rig.canvas.y(39)?),
            ],
            rig.secondary,
        )?;
    }
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
    eyes(scene, rig, 51, 6, 3)
}

fn octopus(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(47)?),
        rig.canvas.s(25)?,
        rig.canvas.s(24)?,
        rig.primary,
    )?;
    for (index, x) in [31, 43, 57, 69].iter().copied().enumerate() {
        let bend = if index.is_multiple_of(2) { -3 } else { 3 };
        let bottom = 77 + i32::from((rig.traits.detail_b() >> (index * 2)) & 3);
        rect(
            scene,
            Rect::new(
                rig.canvas.x(x - 3)?,
                rig.canvas.y(59)?,
                rig.canvas.x(x + 3)?,
                rig.canvas.y(bottom)?,
            ),
            rig.primary,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x + bend)?, rig.canvas.y(bottom)?),
            rig.canvas.s(5)?,
            rig.canvas.s(5)?,
            rig.primary,
        )?;
    }
    eyes(scene, rig, 52, 8, 3)?;
    smile(scene, rig, 63)
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
