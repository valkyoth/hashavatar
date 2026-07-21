use crate::{AvatarError, AvatarKind, geometry::Point, scene::Scene};

use crate::art::families::common::{FamilyRig, ellipse, eyes, polygon, triangle};

use super::{base_head, split_smile};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Fox => fox(scene, rig),
        AvatarKind::Panda => panda(scene, rig),
        AvatarKind::Bear => bear(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn fox(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [30, 70] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 11)?, rig.canvas.y(41)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(14)?),
                Point::new(rig.canvas.x(x + 11)?, rig.canvas.y(41)?),
            ],
            rig.primary,
        )?;
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 6)?, rig.canvas.y(37)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(22)?),
                Point::new(rig.canvas.x(x + 6)?, rig.canvas.y(37)?),
            ],
            rig.secondary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    polygon(
        scene,
        rig,
        &[(30, 54), (47, 60), (43, 69), (31, 63)],
        rig.light,
    )?;
    polygon(
        scene,
        rig,
        &[(70, 54), (53, 60), (57, 69), (69, 63)],
        rig.light,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        rig.canvas.s(13)?,
        rig.canvas.s(8)?,
        rig.light,
    )?;
    if rig.draws_default_eyes() {
        for x in [39, 61] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
                rig.canvas.s(4)?,
                rig.canvas.s(3)?,
                rig.light,
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
                rig.canvas.s(2)?,
                rig.canvas.s(3)?,
                rig.ink,
            )?;
        }
    }
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(47)?, rig.canvas.y(65)?),
            Point::new(rig.canvas.x(53)?, rig.canvas.y(65)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(69)?),
        ],
        rig.ink,
    )?;
    split_smile(scene, rig, 50, 69)
}

fn panda(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [30, 70] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(34)?),
            rig.canvas.s(10)?,
            rig.canvas.s(10)?,
            rig.primary,
        )?;
    }
    base_head(scene, rig, rig.secondary)?;
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(8)?,
            rig.canvas.s(10)?,
            rig.primary,
        )?;
        if rig.draws_default_eyes() {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(3)?,
                rig.canvas.s(4)?,
                rig.light,
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(1)?,
                rig.canvas.s(2)?,
                rig.ink,
            )?;
        }
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        rig.canvas.s(4)?,
        rig.canvas.s(3)?,
        rig.ink,
    )?;
    split_smile(scene, rig, 50, 67)
}

fn bear(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [31, 69] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(32)?),
            rig.canvas.s(11)?,
            rig.canvas.s(11)?,
            rig.primary,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(32)?),
            rig.canvas.s(5)?,
            rig.canvas.s(5)?,
            rig.accent,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    eyes(scene, rig, 48, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(66)?),
        rig.canvas.s(11)?,
        rig.canvas.s(8)?,
        rig.secondary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(63)?),
        rig.canvas.s(4)?,
        rig.canvas.s(3)?,
        rig.ink,
    )?;
    split_smile(scene, rig, 50, 67)
}
