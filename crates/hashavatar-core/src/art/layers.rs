mod accessories;
mod common;
mod expressions;

use crate::{AvatarError, AvatarZBand, LayoutReport, ResolvedStyle, scene::Scene};

pub(super) fn compile_accessories(
    scene: &mut Scene,
    style: ResolvedStyle,
    layout: &LayoutReport,
    band: AvatarZBand,
) -> Result<(), AvatarError> {
    let Some(anchors) = layout.anchors() else {
        return Ok(());
    };
    for decision in layout.accessory_decisions() {
        if decision.z_band() == band
            && let Some(accessory) = decision.effective()
        {
            accessories::compile(
                scene,
                anchors,
                style.color_roles(),
                accessory,
                decision.vertical_adjustment_basis_points(),
            )?;
        }
    }
    Ok(())
}

pub(super) fn compile_expression(
    scene: &mut Scene,
    style: ResolvedStyle,
    layout: &LayoutReport,
) -> Result<(), AvatarError> {
    let Some(anchors) = layout.anchors() else {
        return Ok(());
    };
    expressions::compile(
        scene,
        style.kind(),
        anchors,
        style.color_roles(),
        style.expression(),
    )
}
