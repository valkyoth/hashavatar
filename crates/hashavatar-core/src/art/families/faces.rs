mod animals;
mod characters;

use super::common::{FamilyRig, ellipse, eyes, line, polygon, rect, smile, triangle};
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
        AvatarKind::Cat
        | AvatarKind::Dog
        | AvatarKind::Fox
        | AvatarKind::Frog
        | AvatarKind::Panda
        | AvatarKind::Bear
        | AvatarKind::Dragon => animals::compile(scene, kind, rig),
        AvatarKind::Robot => robot(scene, rig),
        AvatarKind::Alien => alien(scene, rig),
        AvatarKind::Monster => monster(scene, rig),
        AvatarKind::Wizard => wizard(scene, rig),
        AvatarKind::Skull => skull(scene, rig),
        AvatarKind::Knight => knight(scene, rig),
        AvatarKind::Ninja | AvatarKind::Astronaut => characters::compile(scene, kind, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn robot(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(29)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(16)?),
        rig.canvas.s(2)?,
        rig.secondary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(14)?),
        rig.canvas.s(3)?,
        rig.canvas.s(3)?,
        rig.accent,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(23)?,
            rig.canvas.y(29)?,
            rig.canvas.x(77)?,
            rig.canvas.y(78)?,
        ),
        rig.primary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(28)?,
            rig.canvas.y(38)?,
            rig.canvas.x(72)?,
            rig.canvas.y(59)?,
        ),
        rig.secondary,
    )?;
    if rig.draws_default_eyes() {
        for x in [39, 61] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(48)?),
                rig.canvas.s(5)?,
                rig.canvas.s(5)?,
                rig.accent,
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(48)?),
                rig.canvas.s(2)?,
                rig.canvas.s(2)?,
                rig.ink,
            )?;
        }
    }
    if rig.draws_default_mouth() {
        rect(
            scene,
            Rect::new(
                rig.canvas.x(36)?,
                rig.canvas.y(67)?,
                rig.canvas.x(64)?,
                rig.canvas.y(73)?,
            ),
            rig.ink,
        )?;
        for x in [42, 50, 58] {
            line(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(68)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(72)?),
                rig.canvas.s(1)?,
                rig.light,
            )?;
        }
    }
    Ok(())
}

fn alien(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(52)?),
        rig.canvas.s(20)?,
        rig.canvas.s(33)?,
        rig.primary,
    )?;
    if rig.draws_default_eyes() {
        for x in [41, 59] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(47)?),
                rig.canvas.s(5)?,
                rig.canvas.s(11)?,
                rig.ink,
            )?;
        }
    }
    if rig.draws_default_mouth() {
        line(
            scene,
            Point::new(rig.canvas.x(46)?, rig.canvas.y(65)?),
            Point::new(rig.canvas.x(54)?, rig.canvas.y(65)?),
            rig.canvas.s(1)?,
            rig.secondary,
        )
    } else {
        Ok(())
    }
}

fn monster(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [34, 66] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 7)?, rig.canvas.y(35)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(15)?),
                Point::new(rig.canvas.x(x + 7)?, rig.canvas.y(35)?),
            ],
            rig.secondary,
        )?;
    }
    for (start_x, end_x, end_y) in [(36, 30, 84), (50, 50, 87), (64, 70, 84)] {
        line(
            scene,
            Point::new(rig.canvas.x(start_x)?, rig.canvas.y(72)?),
            Point::new(rig.canvas.x(end_x)?, rig.canvas.y(end_y)?),
            rig.canvas.s(6)?,
            rig.secondary,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(end_x)?, rig.canvas.y(end_y)?),
            rig.canvas.s(4)?,
            rig.canvas.s(4)?,
            rig.secondary,
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(56)?),
        rig.canvas.s(25)?,
        rig.canvas.s(27)?,
        rig.primary,
    )?;
    for (x, y, size) in [(35, 58, 3), (51, 37, 4), (66, 60, 3)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(size)?,
            rig.canvas.s(size)?,
            rig.secondary.with_opacity(145),
        )?;
    }
    if rig.draws_default_eyes() {
        for x in [40, 60] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(5)?,
                rig.canvas.s(6)?,
                rig.light,
            )?;
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
                rig.canvas.s(2)?,
                rig.canvas.s(3)?,
                rig.ink,
            )?;
        }
    }
    if rig.draws_default_mouth() {
        rect(
            scene,
            Rect::new(
                rig.canvas.x(36)?,
                rig.canvas.y(66)?,
                rig.canvas.x(64)?,
                rig.canvas.y(76)?,
            ),
            rig.ink,
        )?;
        for x in [42, 58] {
            triangle(
                scene,
                [
                    Point::new(rig.canvas.x(x - 3)?, rig.canvas.y(66)?),
                    Point::new(rig.canvas.x(x + 3)?, rig.canvas.y(66)?),
                    Point::new(rig.canvas.x(x)?, rig.canvas.y(73)?),
                ],
                rig.light,
            )?;
        }
    }
    Ok(())
}

fn wizard(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(58)?),
        rig.canvas.s(17)?,
        rig.canvas.s(18)?,
        rig.secondary,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(26)?, rig.canvas.y(40)?),
            Point::new(rig.canvas.x(58)?, rig.canvas.y(7)?),
            Point::new(rig.canvas.x(73)?, rig.canvas.y(40)?),
        ],
        rig.primary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(22)?,
            rig.canvas.y(36)?,
            rig.canvas.x(78)?,
            rig.canvas.y(42)?,
        ),
        rig.accent,
    )?;
    for (x, y, size) in [(42, 24, 2), (59, 19, 1)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(size)?,
            rig.canvas.s(size)?,
            rig.accent,
        )?;
    }
    eyes(scene, rig, 55, 5, 2)?;
    polygon(
        scene,
        rig,
        &[(36, 64), (50, 91), (64, 64), (58, 79), (50, 73), (42, 79)],
        rig.light,
    )?;
    smile(scene, rig, 63)
}

fn skull(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(51)?),
        rig.canvas.s(24)?,
        rig.canvas.s(25)?,
        rig.light,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(39)?,
            rig.canvas.y(68)?,
            rig.canvas.x(61)?,
            rig.canvas.y(81)?,
        ),
        rig.light,
    )?;
    if rig.draws_default_eyes() {
        for x in [39, 61] {
            ellipse(
                scene,
                Point::new(rig.canvas.x(x)?, rig.canvas.y(50)?),
                rig.canvas.s(6)?,
                rig.canvas.s(8)?,
                rig.ink,
            )?;
        }
    }
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(46)?, rig.canvas.y(65)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(58)?),
            Point::new(rig.canvas.x(54)?, rig.canvas.y(65)?),
        ],
        rig.ink,
    )?;
    for x in [43, 50, 57] {
        line(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(70)?),
            Point::new(rig.canvas.x(x)?, rig.canvas.y(80)?),
            rig.canvas.s(1)?,
            rig.ink,
        )?;
    }
    line(
        scene,
        Point::new(rig.canvas.x(43)?, rig.canvas.y(35)?),
        Point::new(rig.canvas.x(51)?, rig.canvas.y(55)?),
        rig.canvas.s(1)?,
        rig.ink,
    )
}

fn knight(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(48)?),
        rig.canvas.s(23)?,
        rig.canvas.s(27)?,
        rig.secondary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(27)?,
            rig.canvas.y(48)?,
            rig.canvas.x(73)?,
            rig.canvas.y(78)?,
        ),
        rig.secondary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(27)?,
            rig.canvas.y(45)?,
            rig.canvas.x(73)?,
            rig.canvas.y(58)?,
        ),
        rig.ink,
    )?;
    for x in [34, 44, 54, 64] {
        rect(
            scene,
            Rect::new(
                rig.canvas.x(x)?,
                rig.canvas.y(48)?,
                rig.canvas.x(x + 4)?,
                rig.canvas.y(56)?,
            ),
            rig.light,
        )?;
    }
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(25)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(78)?),
        rig.canvas.s(1)?,
        rig.light,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(50)?, rig.canvas.y(25)?),
            Point::new(rig.canvas.x(45)?, rig.canvas.y(10)?),
            Point::new(rig.canvas.x(58)?, rig.canvas.y(16)?),
        ],
        rig.accent,
    )
}
