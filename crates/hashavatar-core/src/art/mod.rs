mod background;
mod families;
mod frame;
mod util;

use crate::{AvatarStyle, AvatarTraitVector, CatError, scene::Scene};

pub(crate) fn compile_avatar_scene(
    width: u32,
    height: u32,
    style: AvatarStyle,
    traits: AvatarTraitVector,
) -> Result<Scene, CatError> {
    let mut scene = Scene::new(width, height)?;
    let frame = frame::begin(&mut scene, style.shape())?;
    background::compile(&mut scene, style.background(), traits)?;
    families::compile(&mut scene, style.kind(), traits)?;
    frame::finish(&mut scene, frame)?;
    Ok(scene)
}
