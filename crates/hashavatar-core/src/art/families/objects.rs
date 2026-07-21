use super::common::{FamilyRig, ellipse, line, polygon, rect, triangle};
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
        AvatarKind::Paws => paws(scene, rig),
        AvatarKind::Planet => planet(scene, rig),
        AvatarKind::Rocket => rocket(scene, rig),
        AvatarKind::Mushroom => mushroom(scene, rig),
        AvatarKind::Cactus => cactus(scene, rig),
        AvatarKind::Cupcake => cupcake(scene, rig),
        AvatarKind::Pizza => pizza(scene, rig),
        AvatarKind::Icecream => icecream(scene, rig),
        AvatarKind::Diamond => diamond(scene, rig),
        AvatarKind::CoffeeCup => coffee(scene, rig),
        AvatarKind::Shield => shield(scene, rig),
        _ => Err(AvatarError::InvalidScene),
    }
}

fn paws(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(64)?),
        rig.canvas.s(18)?,
        rig.canvas.s(14)?,
        rig.primary,
    )?;
    for (x, y) in [(31, 43), (43, 34), (57, 34), (69, 43)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(7)?,
            rig.canvas.s(9)?,
            rig.primary,
        )?;
    }
    Ok(())
}

fn planet(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    line(
        scene,
        Point::new(rig.canvas.x(20)?, rig.canvas.y(68)?),
        Point::new(rig.canvas.x(80)?, rig.canvas.y(38)?),
        rig.canvas.s(8)?,
        rig.secondary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(53)?),
        rig.canvas.s(24)?,
        rig.canvas.s(24)?,
        rig.primary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(43)?, rig.canvas.y(47)?),
        rig.canvas.s(5)?,
        rig.canvas.s(3)?,
        rig.accent,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(20)?, rig.canvas.y(68)?),
        Point::new(rig.canvas.x(80)?, rig.canvas.y(38)?),
        rig.canvas.s(3)?,
        rig.light,
    )
}

fn rocket(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(50)?),
        rig.canvas.s(15)?,
        rig.canvas.s(29)?,
        rig.primary,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(35)?, rig.canvas.y(66)?),
            Point::new(rig.canvas.x(25)?, rig.canvas.y(79)?),
            Point::new(rig.canvas.x(42)?, rig.canvas.y(73)?),
        ],
        rig.secondary,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(65)?, rig.canvas.y(66)?),
            Point::new(rig.canvas.x(75)?, rig.canvas.y(79)?),
            Point::new(rig.canvas.x(58)?, rig.canvas.y(73)?),
        ],
        rig.secondary,
    )?;
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(43)?, rig.canvas.y(76)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(91)?),
            Point::new(rig.canvas.x(57)?, rig.canvas.y(76)?),
        ],
        rig.accent,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(46)?),
        rig.canvas.s(7)?,
        rig.canvas.s(7)?,
        rig.light,
    )
}

fn mushroom(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    rect(
        scene,
        Rect::new(
            rig.canvas.x(42)?,
            rig.canvas.y(52)?,
            rig.canvas.x(58)?,
            rig.canvas.y(82)?,
        ),
        rig.light,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(47)?),
        rig.canvas.s(28)?,
        rig.canvas.s(19)?,
        rig.primary,
    )?;
    for (x, y) in [(38, 43), (52, 35), (64, 48)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(4)?,
            rig.canvas.s(4)?,
            rig.light,
        )?;
    }
    Ok(())
}

fn cactus(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    rect(
        scene,
        Rect::new(
            rig.canvas.x(43)?,
            rig.canvas.y(27)?,
            rig.canvas.x(57)?,
            rig.canvas.y(82)?,
        ),
        rig.primary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(27)?,
            rig.canvas.y(46)?,
            rig.canvas.x(45)?,
            rig.canvas.y(58)?,
        ),
        rig.primary,
    )?;
    rect(
        scene,
        Rect::new(
            rig.canvas.x(55)?,
            rig.canvas.y(39)?,
            rig.canvas.x(73)?,
            rig.canvas.y(51)?,
        ),
        rig.primary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(25)?),
        rig.canvas.s(7)?,
        rig.canvas.s(7)?,
        rig.accent,
    )
}

fn cupcake(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    polygon(
        scene,
        rig,
        &[(31, 55), (69, 55), (62, 83), (38, 83)],
        rig.primary,
    )?;
    for (x, y, radius) in [(38, 52, 11), (50, 44, 14), (63, 52, 11)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(radius)?,
            rig.canvas.s(radius)?,
            rig.secondary,
        )?;
    }
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(27)?),
        rig.canvas.s(4)?,
        rig.canvas.s(4)?,
        rig.accent,
    )
}

fn pizza(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(50)?, rig.canvas.y(84)?),
            Point::new(rig.canvas.x(24)?, rig.canvas.y(28)?),
            Point::new(rig.canvas.x(76)?, rig.canvas.y(28)?),
        ],
        rig.secondary,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(25)?, rig.canvas.y(29)?),
        Point::new(rig.canvas.x(75)?, rig.canvas.y(29)?),
        rig.canvas.s(8)?,
        rig.primary,
    )?;
    for (x, y) in [(42, 45), (60, 48), (51, 64)] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(4)?,
            rig.canvas.s(4)?,
            rig.accent,
        )?;
    }
    Ok(())
}

fn icecream(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    triangle(
        scene,
        [
            Point::new(rig.canvas.x(34)?, rig.canvas.y(52)?),
            Point::new(rig.canvas.x(66)?, rig.canvas.y(52)?),
            Point::new(rig.canvas.x(50)?, rig.canvas.y(87)?),
        ],
        rig.secondary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(42)?),
        rig.canvas.s(19)?,
        rig.canvas.s(19)?,
        rig.primary,
    )?;
    ellipse(
        scene,
        Point::new(rig.canvas.x(43)?, rig.canvas.y(36)?),
        rig.canvas.s(5)?,
        rig.canvas.s(4)?,
        rig.light,
    )
}

fn diamond(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    polygon(
        scene,
        rig,
        &[(50, 84), (23, 43), (34, 25), (66, 25), (77, 43)],
        rig.primary,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(23)?, rig.canvas.y(43)?),
        Point::new(rig.canvas.x(77)?, rig.canvas.y(43)?),
        rig.canvas.s(1)?,
        rig.light,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(34)?, rig.canvas.y(25)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(84)?),
        rig.canvas.s(1)?,
        rig.light,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(66)?, rig.canvas.y(25)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(84)?),
        rig.canvas.s(1)?,
        rig.light,
    )
}

fn coffee(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    rect(
        scene,
        Rect::new(
            rig.canvas.x(27)?,
            rig.canvas.y(42)?,
            rig.canvas.x(65)?,
            rig.canvas.y(76)?,
        ),
        rig.primary,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(65)?, rig.canvas.y(49)?),
        Point::new(rig.canvas.x(78)?, rig.canvas.y(52)?),
        rig.canvas.s(7)?,
        rig.primary,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(78)?, rig.canvas.y(52)?),
        Point::new(rig.canvas.x(66)?, rig.canvas.y(67)?),
        rig.canvas.s(7)?,
        rig.primary,
    )?;
    for x in [39, 52] {
        line(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(35)?),
            Point::new(rig.canvas.x(x + 3)?, rig.canvas.y(19)?),
            rig.canvas.s(2)?,
            rig.light,
        )?;
    }
    Ok(())
}

fn shield(scene: &mut Scene, rig: FamilyRig) -> Result<(), AvatarError> {
    polygon(
        scene,
        rig,
        &[(50, 86), (27, 68), (24, 29), (50, 20), (76, 29), (73, 68)],
        rig.primary,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(50)?, rig.canvas.y(28)?),
        Point::new(rig.canvas.x(50)?, rig.canvas.y(75)?),
        rig.canvas.s(3)?,
        rig.light,
    )?;
    line(
        scene,
        Point::new(rig.canvas.x(34)?, rig.canvas.y(48)?),
        Point::new(rig.canvas.x(66)?, rig.canvas.y(48)?),
        rig.canvas.s(3)?,
        rig.light,
    )
}
