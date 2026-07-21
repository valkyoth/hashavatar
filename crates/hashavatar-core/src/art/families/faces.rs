mod animals;
mod characters;

use super::common::{FamilyRig, ellipse, eyes, line, polygon, rect, triangle};
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
    for x in [41, 59] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(47)?),
            rig.canvas.s(5)?,
            rig.canvas.s(11)?,
            rig.ink,
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(62)?),
        rig.canvas.s(2)?,
        rig.canvas.s(2)?,
        rig.secondary,
    )
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
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(56)?),
        rig.canvas.s(25)?,
        rig.canvas.s(27)?,
        rig.primary,
    )?;
    let eye_count = 1 + usize::from(rig.traits.detail_a() % 3);
    let eye_positions: &[i32] = match eye_count {
        1 => &[50],
        2 => &[39, 61],
        _ => &[36, 50, 64],
    };
    for x in eye_positions.iter().copied() {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(49)?),
            rig.canvas.s(5)?,
            rig.canvas.s(6)?,
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
    eyes(scene, rig, 54, 7, 2)?;
    polygon(
        scene,
        rig,
        &[(36, 64), (50, 91), (64, 64), (58, 79), (50, 73), (42, 79)],
        rig.light,
    )
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
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(50)?),
            rig.canvas.s(6)?,
            rig.canvas.s(8)?,
            rig.ink,
        )?;
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
