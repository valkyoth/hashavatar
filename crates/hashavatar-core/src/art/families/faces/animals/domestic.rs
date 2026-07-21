use crate::{AvatarError, AvatarKind, geometry::Point, paint::Color, scene::Scene};

use crate::art::families::common::{FamilyRig, ellipse, eyes, line, triangle};

use super::{base_head, split_smile};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Cat => cat(scene, rig),
        AvatarKind::Dog => dog(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn cat(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [36, 64] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 9)?, rig.canvas.y(41)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(9)?),
                Point::new(rig.canvas.x(x + 9)?, rig.canvas.y(41)?),
            ],
            rig.primary,
        )?;
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 5)?, rig.canvas.y(37)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(18)?),
                Point::new(rig.canvas.x(x + 5)?, rig.canvas.y(37)?),
            ],
            Color::rgb(219, 143, 151),
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    for x in [45, 55] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(65)?),
            rig.canvas.s(8)?,
            rig.canvas.s(6)?,
            rig.secondary,
        )?;
    }
    if rig.draws_default_eyes() {
        for x in [39, 61] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(4)?,
                rig.canvas.s(5)?,
                Color::rgb(83, 168, 72),
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(2)?,
                rig.canvas.s(4)?,
                rig.ink,
            )?;
        }
    }
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(47)?, rig.canvas.y(63)?),
            Point::new(rig.canvas.x(53)?, rig.canvas.y(63)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(68)?),
        ],
        Color::rgb(210, 112, 127),
    )?;
    split_smile(scene, rig, 50, 70)?;
    for (start_x, end_x, y, end_y) in [
        (43, 20, 64, 60),
        (43, 20, 71, 75),
        (57, 80, 64, 60),
        (57, 80, 71, 75),
    ] {
        line(
            scene,
            Point::new(rig.canvas.x(start_x)?, rig.canvas.y(y)?),
            Point::new(rig.canvas.x(end_x)?, rig.canvas.y(end_y)?),
            rig.canvas.s(1)?,
            rig.ink,
        )?;
    }
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(37)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(46)?),
        rig.canvas.s(1)?,
        rig.ink.with_opacity(120),
    )
}

fn dog(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [28, 72] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
            rig.canvas.s(11)?,
            rig.canvas.s(23)?,
            rig.secondary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    if rig.traits.detail_a().is_multiple_of(2) {
        ellipse(
            scene,
            Point::new(rig.canvas.x(39)?, rig.canvas.y(51)?),
            rig.canvas.s(8)?,
            rig.canvas.s(10)?,
            rig.secondary.with_opacity(150),
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(66)?),
        rig.canvas.s(14)?,
        rig.canvas.s(9)?,
        rig.light,
    )?;
    eyes(scene, rig, 50, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(63)?),
        rig.canvas.s(4)?,
        rig.canvas.s(3)?,
        rig.ink,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(69)?),
        rig.canvas.s(1)?,
        rig.ink,
    )?;
    split_smile(scene, rig, 50, 69)?;
    if rig.draws_default_mouth() && !rig.traits.detail_b().is_multiple_of(3) {
        ellipse(
            scene,
            Point::new(rig.canvas.x(50)?, rig.canvas.y(77)?),
            rig.canvas.s(4)?,
            rig.canvas.s(6)?,
            Color::rgb(225, 132, 149),
        )?;
    }
    Ok(())
}
