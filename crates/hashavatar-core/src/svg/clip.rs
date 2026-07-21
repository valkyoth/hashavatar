use core::fmt::Write;

use super::{fill_rule_name, write_number, write_path_data, write_rect_values, write_text};
use crate::{
    CatError,
    scene::{Clip, Scene},
};

pub(super) fn write_clip(
    output: &mut impl Write,
    scene: &Scene,
    prefix: &str,
    index: usize,
    clip: Clip,
) -> Result<(), CatError> {
    write!(output, "<defs><clipPath id=\"{prefix}-clip-{index}\">")
        .map_err(|_| CatError::SvgWrite)?;
    match clip {
        Clip::Rect(rect) => {
            write_text(output, "<rect x=\"")?;
            write_rect_values(output, rect)?;
            write_text(output, "\"/>")?;
        }
        Clip::Ellipse {
            center,
            radius_x,
            radius_y,
        } => {
            write_text(output, "<ellipse cx=\"")?;
            write_number(output, center.x)?;
            write_text(output, "\" cy=\"")?;
            write_number(output, center.y)?;
            write_text(output, "\" rx=\"")?;
            write_number(output, radius_x)?;
            write_text(output, "\" ry=\"")?;
            write_number(output, radius_y)?;
            write_text(output, "\"/>")?;
        }
        Clip::Path {
            path_index,
            fill_rule,
        } => {
            write_text(output, "<path d=\"")?;
            write_path_data(output, scene.path(path_index)?)?;
            write!(output, "\" clip-rule=\"{}\"/>", fill_rule_name(fill_rule))
                .map_err(|_| CatError::SvgWrite)?;
        }
    }
    write!(
        output,
        "</clipPath></defs><g clip-path=\"url(#{prefix}-clip-{index})\">"
    )
    .map_err(|_| CatError::SvgWrite)
}
