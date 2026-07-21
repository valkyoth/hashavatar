use crate::{AvatarError, AvatarKind, geometry::Point, scene::Scene};

use crate::art::families::common::{FamilyRig, ellipse, eyes, polygon, smile, triangle};

use super::base_head;

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Frog => frog(scene, rig),
        AvatarKind::Dragon => dragon(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn frog(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [36, 64] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(39)?),
            rig.canvas.s(10)?,
            rig.canvas.s(10)?,
            rig.primary,
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(58)?),
        rig.canvas.s(27)?,
        rig.canvas.s(21)?,
        rig.primary,
    )?;
    eyes(scene, rig, 39, 13, 4)?;
    for x in [36, 64] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(64)?),
            rig.canvas.s(4)?,
            rig.canvas.s(4)?,
            rig.accent.with_opacity(170),
        )?;
    }
    smile(scene, rig, 63)
}

fn dragon(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [32, 68] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 8)?, rig.canvas.y(38)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(10)?),
                Point::new(rig.canvas.x(x + 8)?, rig.canvas.y(38)?),
            ],
            rig.light,
        )?;
    }
    for x in [43, 50, 57] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 4)?, rig.canvas.y(35)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(20)?),
                Point::new(rig.canvas.x(x + 4)?, rig.canvas.y(35)?),
            ],
            rig.accent,
        )?;
    }
    for direction in [-1, 1] {
        let center = 50 + direction * 27;
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(center)?, rig.canvas.y(48)?),
                Point::new(rig.canvas.x(center + direction * 13)?, rig.canvas.y(42)?),
                Point::new(rig.canvas.x(center + direction * 5)?, rig.canvas.y(58)?),
            ],
            rig.accent,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    if rig.draws_default_eyes() {
        for x in [39, 61] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
                rig.canvas.s(5)?,
                rig.canvas.s(4)?,
                rig.light,
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
                rig.canvas.s(2)?,
                rig.canvas.s(4)?,
                rig.ink,
            )?;
        }
    }
    polygon(
        scene,
        rig,
        &[(34, 60), (66, 60), (70, 70), (50, 79), (30, 70)],
        rig.secondary,
    )?;
    for x in [44, 56] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(65)?),
            rig.canvas.s(2)?,
            rig.canvas.s(2)?,
            rig.ink,
        )?;
    }
    smile(scene, rig, 72)?;
    if rig.draws_default_mouth() {
        for x in [42, 58] {
            triangle(
                scene,
                [
                    Point::new(rig.canvas.x(x - 3)?, rig.canvas.y(72)?),
                    Point::new(rig.canvas.x(x + 3)?, rig.canvas.y(72)?),
                    Point::new(rig.canvas.x(x)?, rig.canvas.y(78)?),
                ],
                rig.light,
            )?;
        }
    }
    Ok(())
}
