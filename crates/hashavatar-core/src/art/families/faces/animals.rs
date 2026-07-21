mod domestic;
mod fantasy;
mod wild;

use crate::{
    AvatarError, AvatarKind,
    geometry::{FillRule, Path, Point},
    paint::Paint,
    scene::{Command, Scene, Stroke},
};

use super::super::common::{FamilyRig, ellipse};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    rig: FamilyRig,
) -> Result<(), AvatarError> {
    match kind {
        AvatarKind::Cat | AvatarKind::Dog => domestic::compile(scene, kind, rig),
        AvatarKind::Fox | AvatarKind::Panda | AvatarKind::Bear => wild::compile(scene, kind, rig),
        AvatarKind::Frog | AvatarKind::Dragon => fantasy::compile(scene, kind, rig),
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

fn split_smile(
    scene: &mut Scene,
    rig: FamilyRig,
    center_x: i32,
    y: i32,
) -> Result<(), AvatarError> {
    if !rig.draws_default_mouth() {
        return Ok(());
    }
    for direction in [-1, 1] {
        let end_x = center_x + direction * 8;
        let control_x = center_x + direction * 4;
        let mut path = Path::builder(Point::new(rig.canvas.x(center_x)?, rig.canvas.y(y)?))?;
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
