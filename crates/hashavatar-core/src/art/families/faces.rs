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
        AvatarKind::Cat => cat(scene, rig),
        AvatarKind::Dog => dog(scene, rig),
        AvatarKind::Robot => robot(scene, rig),
        AvatarKind::Fox => fox(scene, rig),
        AvatarKind::Alien => alien(scene, rig),
        AvatarKind::Monster => monster(scene, rig),
        AvatarKind::Wizard => wizard(scene, rig),
        AvatarKind::Skull => skull(scene, rig),
        AvatarKind::Frog => frog(scene, rig),
        AvatarKind::Panda => panda(scene, rig),
        AvatarKind::Knight => knight(scene, rig),
        AvatarKind::Bear => bear(scene, rig),
        AvatarKind::Dragon => dragon(scene, rig),
        AvatarKind::Ninja | AvatarKind::Astronaut => characters::compile(scene, kind, rig),
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
    for x in [31, 69] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 10)?, rig.canvas.y(39)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(18)?),
                Point::new(rig.canvas.x(x + 10)?, rig.canvas.y(40)?),
            ],
            rig.primary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        rig.canvas.s(11)?,
        rig.canvas.s(7)?,
        rig.secondary,
    )?;
    eyes(scene, rig, 50, 11, 4)?;
    smile(scene, rig, 68)
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
    smile(scene, rig, 68)
}

fn robot(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(25)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(16)?),
        rig.canvas.s(2)?,
        rig.ink,
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
            rig.canvas.y(35)?,
            rig.canvas.x(72)?,
            rig.canvas.y(60)?,
        ),
        rig.ink,
    )?;
    eyes(scene, rig, 48, 11, 4)?;
    for x in [40, 47, 54, 61] {
        rect(
            scene,
            Rect::new(
                rig.canvas.x(x)?,
                rig.canvas.y(68)?,
                rig.canvas.x(x + 3)?,
                rig.canvas.y(73)?,
            ),
            rig.ink,
        )?;
    }
    Ok(())
}

fn fox(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [30, 70] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 11)?, rig.canvas.y(41)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(17)?),
                Point::new(rig.canvas.x(x + 11)?, rig.canvas.y(41)?),
            ],
            rig.primary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    polygon(
        scene,
        rig,
        &[(34, 59), (50, 78), (66, 59), (50, 69)],
        rig.light,
    )?;
    eyes(scene, rig, 49, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(66)?),
        rig.canvas.s(3)?,
        rig.canvas.s(2)?,
        rig.ink,
    )
}

fn alien(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(52)?),
        rig.canvas.s(25)?,
        rig.canvas.s(34)?,
        rig.primary,
    )?;
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(48)?),
            rig.canvas.s(7)?,
            rig.canvas.s(12)?,
            rig.ink,
        )?;
    }
    line(
        scene,
        Point::new(rig.canvas.x(45)?, rig.canvas.y(69)?),
        Point::new(rig.canvas.x(55)?, rig.canvas.y(69)?),
        rig.canvas.s(1)?,
        rig.ink,
    )
}

fn monster(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [34, 66] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 7)?, rig.canvas.y(36)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(17)?),
                Point::new(rig.canvas.x(x + 7)?, rig.canvas.y(36)?),
            ],
            rig.accent,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    for x in [36, 50, 64] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(50)?),
            rig.canvas.s(5)?,
            rig.canvas.s(6)?,
            rig.light,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(50)?),
            rig.canvas.s(2)?,
            rig.canvas.s(2)?,
            rig.ink,
        )?;
    }
    smile(scene, rig, 66)
}

fn wizard(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    base_head(scene, rig, rig.secondary)?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(27)?, rig.canvas.y(37)?),
            Point::new(rig.canvas.x(58)?, rig.canvas.y(8)?),
            Point::new(rig.canvas.x(72)?, rig.canvas.y(37)?),
        ],
        rig.primary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(23)?,
            rig.canvas.y(34)?,
            rig.canvas.x(77)?,
            rig.canvas.y(41)?,
        ),
        rig.accent,
    )?;
    eyes(scene, rig, 52, 10, 3)?;
    polygon(
        scene,
        rig,
        &[(36, 64), (50, 88), (64, 64), (58, 78), (50, 72), (42, 78)],
        rig.light,
    )
}

fn skull(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    base_head(scene, rig, rig.light)?;
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(7)?,
            rig.canvas.s(9)?,
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
    rect(
        scene,
        Rect::new(
            rig.canvas.x(37)?,
            rig.canvas.y(69)?,
            rig.canvas.x(63)?,
            rig.canvas.y(79)?,
        ),
        rig.light,
    )?;
    for x in [42, 50, 58] {
        line(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(70)?),
            Point::new(rig.canvas.x(x)?, rig.canvas.y(78)?),
            rig.canvas.s(1)?,
            rig.ink,
        )?;
    }
    Ok(())
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
    base_head(scene, rig, rig.primary)?;
    eyes(scene, rig, 38, 14, 4)?;
    smile(scene, rig, 65)
}

fn panda(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [30, 70] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(34)?),
            rig.canvas.s(10)?,
            rig.canvas.s(10)?,
            rig.ink,
        )?;
    }
    base_head(scene, rig, rig.light)?;
    for x in [39, 61] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(8)?,
            rig.canvas.s(10)?,
            rig.ink,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(51)?),
            rig.canvas.s(3)?,
            rig.canvas.s(4)?,
            rig.light,
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(65)?),
        rig.canvas.s(4)?,
        rig.canvas.s(3)?,
        rig.ink,
    )
}

fn knight(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    base_head(scene, rig, ColorChoice::metal(rig))?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(23)?,
            rig.canvas.y(45)?,
            rig.canvas.x(77)?,
            rig.canvas.y(59)?,
        ),
        rig.ink,
    )?;
    for x in [34, 43, 52, 61] {
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
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(45)?, rig.canvas.y(26)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(12)?),
            Point::new(rig.canvas.x(55)?, rig.canvas.y(26)?),
        ],
        rig.accent,
    )
}

struct ColorChoice;
impl ColorChoice {
    const fn metal(rig: FamilyRig) -> crate::paint::Color {
        rig.secondary
    }
}

fn bear(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [31, 69] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(34)?),
            rig.canvas.s(10)?,
            rig.canvas.s(10)?,
            rig.primary,
        )?;
    }
    base_head(scene, rig, rig.primary)?;
    eyes(scene, rig, 51, 11, 3)?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(66)?),
        rig.canvas.s(10)?,
        rig.canvas.s(7)?,
        rig.secondary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(63)?),
        rig.canvas.s(3)?,
        rig.canvas.s(2)?,
        rig.ink,
    )
}

fn dragon(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    for x in [32, 68] {
        triangle(
            scene,
            [
                Point::new(rig.canvas.x(x - 8)?, rig.canvas.y(38)?),
                Point::new(rig.canvas.x(x)?, rig.canvas.y(15)?),
                Point::new(rig.canvas.x(x + 8)?, rig.canvas.y(38)?),
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
    Ok(())
}
