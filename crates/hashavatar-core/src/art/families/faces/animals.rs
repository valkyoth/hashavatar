use crate::{
    AvatarError, AvatarKind,
    geometry::{FillRule, Path, Point},
    paint::Paint,
    scene::{Command, Scene, Stroke},
};

use super::super::common::{FamilyRig, ellipse, eyes, line, polygon, smile, triangle};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Cat => cat(scene, rig),
        AvatarKind::Dog => dog(scene, rig),
        AvatarKind::Fox => fox(scene, rig),
        AvatarKind::Frog => frog(scene, rig),
        AvatarKind::Panda => panda(scene, rig),
        AvatarKind::Bear => bear(scene, rig),
        AvatarKind::Dragon => dragon(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn base_head(
    scene: &mut Scene,
    rig: FamilyRig,
    color: crate::paint::Color,
) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(55)?),
        rig.head_rx()?,
        rig.head_ry()?,
        color,
    )
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
            crate::paint::Color::rgb(219, 143, 151),
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
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(4)?,
            rig.canvas.s(5)?,
            crate::paint::Color::rgb(83, 168, 72),
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(2)?,
            rig.canvas.s(4)?,
            rig.ink,
        )?;
    }
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(47)?, rig.canvas.y(63)?),
            Point::new(rig.canvas.x(53)?, rig.canvas.y(63)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(68)?),
        ],
        crate::paint::Color::rgb(210, 112, 127),
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
    )?;
    Ok(())
}

fn dog(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [27, 73] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(48)?),
            rig.canvas.s(10)?,
            rig.canvas.s(22)?,
            rig.secondary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(67)?),
        rig.canvas.s(13)?,
        rig.canvas.s(9)?,
        rig.light,
    )?;
    eyes(scene, rig, 51, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(63)?),
        rig.canvas.s(4)?,
        rig.canvas.s(3)?,
        rig.ink,
    )?;
    split_smile(scene, rig, 50, 67)
}

fn fox(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [30, 70] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 11)?, rig.canvas.y(41)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(16)?),
                Point::new(rig.canvas.x(x + 11)?, rig.canvas.y(41)?),
            ],
            rig.primary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    polygon(scene, rig, &[(31, 57), (50, 78), (50, 64)], rig.light)?;
    polygon(scene, rig, &[(69, 57), (50, 78), (50, 64)], rig.light)?;
    eyes(scene, rig, 49, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(66)?),
        rig.canvas.s(3)?,
        rig.canvas.s(2)?,
        rig.ink,
    )?;
    smile(scene, rig, 68)
}

fn frog(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [36, 64] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(35)?),
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
    eyes(scene, rig, 35, 14, 4)?;
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
    eyes(scene, rig, 51, 11, 3)?;
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

fn dragon(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [32, 68] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 8)?, rig.canvas.y(38)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(13)?),
                Point::new(rig.canvas.x(x + 8)?, rig.canvas.y(38)?),
            ],
            rig.light,
        )?;
    }
    for x in [43, 50, 57] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 3)?, rig.canvas.y(34)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(23)?),
                Point::new(rig.canvas.x(x + 3)?, rig.canvas.y(34)?),
            ],
            rig.accent,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    eyes(scene, rig, 49, 11, 4)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(67)?),
        rig.canvas.s(14)?,
        rig.canvas.s(8)?,
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
    smile(scene, rig, 71)
}

fn split_smile(
    scene: &mut Scene,
    rig: FamilyRig,
    center_x: i32,
    y: i32,
) -> Result<(), AvatarError> {
    for direction in [-1, 1] {
        let start_x = center_x;
        let end_x = center_x + direction * 8;
        let control_x = center_x + direction * 4;
        let mut path = Path::builder(Point::new(rig.canvas.x(start_x)?, rig.canvas.y(y)?))?;
        path.quad_to(
            Point::new(rig.canvas.x(control_x)?, rig.canvas.y(y + 5)?),
            Point::new(rig.canvas.x(end_x)?, rig.canvas.y(y)?),
        )?;
        let path_index = scene.push_path(path.finish(false)?)?;
        scene.push(Command::Path {
            path_index,
            fill_rule: FillRule::NonZero,
            fill: None,
            stroke: Some(Stroke {
                width: rig.canvas.s(1)?,
                paint: Paint::solid(rig.ink),
            }),
        })?;
    }
    Ok(())
}
