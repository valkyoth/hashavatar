use alloc::string::String;
use core::fmt::Write;

use crate::{
    CatError,
    fixed::{Fixed, write_decimal},
    scene::{Color, Command, Point, Scene},
};

const SVG_CAPACITY: usize = 8 * 1024;

pub(crate) fn render_scene_svg(scene: &Scene) -> Result<String, CatError> {
    let _ = scene.validate()?;
    let mut output = SvgBuffer::new()?;
    write!(
        output,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\"><title>Hashavatar Cat</title><desc>Deterministic procedural Cat avatar</desc>",
        scene.width(),
        scene.height(),
        scene.width(),
        scene.height()
    )
    .map_err(|_| CatError::SvgWrite)?;

    for command in scene.commands()? {
        match *command {
            Command::Empty => return Err(CatError::InvalidScene),
            Command::Fill(color) => {
                write!(
                    output,
                    "<rect width=\"{}\" height=\"{}\" fill=\"{}\"/>",
                    scene.width(),
                    scene.height(),
                    HexColor(color)
                )
                .map_err(|_| CatError::SvgWrite)?;
            }
            Command::Ellipse {
                center,
                radius_x,
                radius_y,
                color,
            } => write_ellipse(&mut output, center, radius_x, radius_y, color)?,
            Command::Triangle { points, color } => {
                write_triangle(&mut output, points, color)?;
            }
        }
    }
    output.push_str("</svg>")?;
    Ok(output.finish())
}

fn write_ellipse(
    output: &mut SvgBuffer,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    color: Color,
) -> Result<(), CatError> {
    output.push_str("<ellipse cx=\"")?;
    write_number(output, center.x)?;
    output.push_str("\" cy=\"")?;
    write_number(output, center.y)?;
    output.push_str("\" rx=\"")?;
    write_number(output, radius_x)?;
    output.push_str("\" ry=\"")?;
    write_number(output, radius_y)?;
    write!(output, "\" fill=\"{}\"/>", HexColor(color)).map_err(|_| CatError::SvgWrite)
}

fn write_triangle(
    output: &mut SvgBuffer,
    points: [Point; 3],
    color: Color,
) -> Result<(), CatError> {
    output.push_str("<polygon points=\"")?;
    for (index, point) in points.iter().enumerate() {
        if index > 0 {
            output.push(' ')?;
        }
        write_number(output, point.x)?;
        output.push(',')?;
        write_number(output, point.y)?;
    }
    write!(output, "\" fill=\"{}\"/>", HexColor(color)).map_err(|_| CatError::SvgWrite)
}

fn write_number(output: &mut SvgBuffer, value: Fixed) -> Result<(), CatError> {
    write_decimal(output, value).map_err(|_| CatError::SvgWrite)
}

struct SvgBuffer {
    inner: String,
}

impl SvgBuffer {
    fn new() -> Result<Self, CatError> {
        let mut inner = String::new();
        inner
            .try_reserve_exact(SVG_CAPACITY)
            .map_err(|_| CatError::Allocation)?;
        Ok(Self { inner })
    }

    fn push_str(&mut self, value: &str) -> Result<(), CatError> {
        self.write_str(value).map_err(|_| CatError::SvgWrite)
    }

    fn push(&mut self, value: char) -> Result<(), CatError> {
        self.write_char(value).map_err(|_| CatError::SvgWrite)
    }

    fn finish(self) -> String {
        self.inner
    }
}

impl Write for SvgBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let length = self
            .inner
            .len()
            .checked_add(value.len())
            .ok_or(core::fmt::Error)?;
        if length > SVG_CAPACITY {
            return Err(core::fmt::Error);
        }
        self.inner.push_str(value);
        Ok(())
    }
}

struct HexColor(Color);

impl core::fmt::Display for HexColor {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "#{:02x}{:02x}{:02x}",
            self.0.red, self.0.green, self.0.blue
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::CatRequest;

    use super::{SVG_CAPACITY, SvgBuffer};

    #[test]
    fn canonical_svg_is_well_formed_and_matches_scene_count() -> Result<(), crate::CatError> {
        let request = CatRequest::new(128, 128, 0, b"svg-fixture")?;
        let prepared = request.prepare()?;
        let svg = prepared.render_svg()?;
        let document = roxmltree::Document::parse(&svg);
        assert!(document.is_ok());
        if let Ok(document) = document {
            let drawable_count = document
                .descendants()
                .filter(|node| matches!(node.tag_name().name(), "rect" | "ellipse" | "polygon"))
                .count();
            assert_eq!(drawable_count, prepared.scene_report().command_count());
        }
        Ok(())
    }

    #[test]
    fn svg_buffer_fails_closed_at_its_document_bound() -> Result<(), crate::CatError> {
        let mut buffer = SvgBuffer::new()?;
        let oversized = String::from("x").repeat(SVG_CAPACITY + 1);
        assert_eq!(buffer.push_str(&oversized), Err(crate::CatError::SvgWrite));
        Ok(())
    }
}
