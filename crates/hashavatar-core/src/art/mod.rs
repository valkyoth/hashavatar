mod background;
mod families;
mod frame;
mod layers;
pub(crate) mod util;

use crate::{AvatarTraitVector, AvatarZBand, CatError, LayoutReport, ResolvedStyle, scene::Scene};

pub(crate) fn compile_avatar_scene(
    width: u32,
    height: u32,
    style: ResolvedStyle,
    layout: &LayoutReport,
    traits: AvatarTraitVector,
) -> Result<Scene, CatError> {
    let mut scene = Scene::new(width, height)?;
    let frame = frame::begin(&mut scene, style.shape())?;
    background::compile(&mut scene, style, traits)?;
    layers::compile_accessories(&mut scene, style, layout, AvatarZBand::BehindSubject)?;
    families::compile(&mut scene, style.kind(), traits, style.color_roles())?;
    layers::compile_expression(&mut scene, style, layout)?;
    layers::compile_accessories(&mut scene, style, layout, AvatarZBand::Foreground)?;
    frame::finish(&mut scene, frame)?;
    Ok(scene)
}
