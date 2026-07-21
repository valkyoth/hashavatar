use alloc::string::String;
use core::fmt::Write;

mod clip;
mod options;
use self::clip::write_clip;
pub use self::options::{SvgMode, SvgOptions};

use crate::{
    AvatarError,
    fixed::{Fixed, write_decimal},
    geometry::{FillRule, Path, Point, Rect},
    paint::{Color, Paint, div_255_round},
    scene::{Command, MAX_STACK_DEPTH, Scene, Stroke},
};

const SVG_CAPACITY: usize = crate::MAX_SVG_OUTPUT_BYTES;

pub(crate) fn render_scene_svg(scene: &Scene) -> Result<String, AvatarError> {
    render_scene_svg_with(scene, SvgOptions::default())
}

pub(crate) fn render_scene_svg_with(
    scene: &Scene,
    options: SvgOptions<'_>,
) -> Result<String, AvatarError> {
    let mut output = SvgBuffer::new()?;
    write_scene_svg(scene, &mut output, options)?;
    Ok(output.finish())
}

pub(crate) fn write_scene_svg(
    scene: &Scene,
    output: &mut impl Write,
    options: SvgOptions<'_>,
) -> Result<(), AvatarError> {
    let _ = scene.validate()?;
    options.validate()?;
    write_opening(output, scene, options)?;
    let mut opacity_stack = [u8::MAX; MAX_STACK_DEPTH];
    let mut opacity_depth = 0_usize;
    let mut opacity = u8::MAX;
    for (index, command) in scene.commands()?.iter().enumerate() {
        match *command {
            Command::PushOpacity(value) => {
                let slot = opacity_stack
                    .get_mut(opacity_depth)
                    .ok_or(AvatarError::InvalidScene)?;
                *slot = opacity;
                opacity_depth = opacity_depth
                    .checked_add(1)
                    .ok_or(AvatarError::NumericRange)?;
                opacity = u8::try_from(div_255_round(u32::from(opacity) * u32::from(value)))
                    .map_err(|_| AvatarError::NumericRange)?;
            }
            Command::PopOpacity => {
                opacity_depth = opacity_depth
                    .checked_sub(1)
                    .ok_or(AvatarError::InvalidScene)?;
                opacity = *opacity_stack
                    .get(opacity_depth)
                    .ok_or(AvatarError::InvalidScene)?;
            }
            _ => write_command(output, scene, options.id_prefix, index, *command, opacity)?,
        }
    }
    match options.mode {
        SvgMode::Document => write_text(output, "</svg>"),
        SvgMode::Fragment => write_text(output, "</g>"),
    }
}

fn write_opening(
    output: &mut impl Write,
    scene: &Scene,
    options: SvgOptions<'_>,
) -> Result<(), AvatarError> {
    match options.mode {
        SvgMode::Document => {
            write!(
                output,
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" role=\"img\">",
                scene.width(),
                scene.height(),
                scene.width(),
                scene.height()
            )
            .map_err(|_| AvatarError::SvgWrite)?;
            write_text(output, "<title>")?;
            write_escaped(output, options.title.ok_or(AvatarError::InvalidSvgOptions)?)?;
            write_text(output, "</title><desc>")?;
            write_escaped(
                output,
                options.description.ok_or(AvatarError::InvalidSvgOptions)?,
            )?;
            write_text(output, "</desc>")
        }
        SvgMode::Fragment => write!(output, "<g id=\"{}-scene\">", options.id_prefix)
            .map_err(|_| AvatarError::SvgWrite),
    }
}

fn write_command(
    output: &mut impl Write,
    scene: &Scene,
    prefix: &str,
    index: usize,
    command: Command,
    opacity: u8,
) -> Result<(), AvatarError> {
    match command {
        Command::Empty => Err(AvatarError::InvalidScene),
        Command::Fill(paint) => {
            let paint = paint.with_opacity(opacity);
            write_paint_definition(output, prefix, index, "fill", paint)?;
            write!(
                output,
                "<rect width=\"{}\" height=\"{}\"",
                scene.width(),
                scene.height()
            )
            .map_err(|_| AvatarError::SvgWrite)?;
            write_paint_attribute(output, prefix, index, "fill", "fill", paint)?;
            write_text(output, "/>")
        }
        Command::Rect { rect, paint } => {
            let paint = paint.with_opacity(opacity);
            write_paint_definition(output, prefix, index, "fill", paint)?;
            write_text(output, "<rect x=\"")?;
            write_number(output, rect.left)?;
            write_text(output, "\" y=\"")?;
            write_number(output, rect.top)?;
            write_text(output, "\" width=\"")?;
            write_number(output, rect.right.checked_sub(rect.left)?)?;
            write_text(output, "\" height=\"")?;
            write_number(output, rect.bottom.checked_sub(rect.top)?)?;
            write_text(output, "\"")?;
            write_paint_attribute(output, prefix, index, "fill", "fill", paint)?;
            write_text(output, "/>")
        }
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            paint,
        } => {
            let paint = paint.with_opacity(opacity);
            write_paint_definition(output, prefix, index, "fill", paint)?;
            write_text(output, "<ellipse cx=\"")?;
            write_number(output, center.x)?;
            write_text(output, "\" cy=\"")?;
            write_number(output, center.y)?;
            write_text(output, "\" rx=\"")?;
            write_number(output, radius_x)?;
            write_text(output, "\" ry=\"")?;
            write_number(output, radius_y)?;
            write_text(output, "\"")?;
            write_paint_attribute(output, prefix, index, "fill", "fill", paint)?;
            write_text(output, "/>")
        }
        Command::Triangle { points, paint } => {
            let paint = paint.with_opacity(opacity);
            write_paint_definition(output, prefix, index, "fill", paint)?;
            write_text(output, "<polygon points=\"")?;
            write_points(output, &points)?;
            write_text(output, "\"")?;
            write_paint_attribute(output, prefix, index, "fill", "fill", paint)?;
            write_text(output, "/>")
        }
        Command::Line { start, end, stroke } => {
            let stroke = Stroke {
                paint: stroke.paint.with_opacity(opacity),
                ..stroke
            };
            write_paint_definition(output, prefix, index, "stroke", stroke.paint)?;
            write_text(output, "<line x1=\"")?;
            write_number(output, start.x)?;
            write_text(output, "\" y1=\"")?;
            write_number(output, start.y)?;
            write_text(output, "\" x2=\"")?;
            write_number(output, end.x)?;
            write_text(output, "\" y2=\"")?;
            write_number(output, end.y)?;
            write_text(output, "\"")?;
            write_stroke(output, prefix, index, stroke)?;
            write_text(output, "/>")
        }
        Command::Path {
            path_index,
            fill_rule,
            fill,
            stroke,
        } => write_path(
            output,
            scene.path(path_index)?,
            prefix,
            index,
            fill_rule,
            fill.map(|paint| paint.with_opacity(opacity)),
            stroke.map(|stroke| Stroke {
                paint: stroke.paint.with_opacity(opacity),
                ..stroke
            }),
        ),
        Command::PushClip(clip) => write_clip(output, scene, prefix, index, clip),
        Command::PopClip => write_text(output, "</g>"),
        Command::PushOpacity(_) | Command::PopOpacity => Err(AvatarError::InvalidScene),
    }
}

fn write_path(
    output: &mut impl Write,
    path: &Path,
    prefix: &str,
    index: usize,
    fill_rule: FillRule,
    fill: Option<Paint>,
    stroke: Option<Stroke>,
) -> Result<(), AvatarError> {
    if let Some(paint) = fill {
        write_paint_definition(output, prefix, index, "fill", paint)?;
    }
    if let Some(stroke) = stroke {
        write_paint_definition(output, prefix, index, "stroke", stroke.paint)?;
    }
    write_text(output, "<path d=\"")?;
    write_path_data(output, path)?;
    write_text(output, "\"")?;
    match fill {
        Some(paint) => write_paint_attribute(output, prefix, index, "fill", "fill", paint)?,
        None => write_text(output, " fill=\"none\"")?,
    }
    write!(output, " fill-rule=\"{}\"", fill_rule_name(fill_rule))
        .map_err(|_| AvatarError::SvgWrite)?;
    if let Some(stroke) = stroke {
        write_stroke(output, prefix, index, stroke)?;
    }
    write_text(output, "/>")
}

pub(super) fn write_path_data(output: &mut impl Write, path: &Path) -> Result<(), AvatarError> {
    let mut points = path.points()?.iter();
    let first = points.next().ok_or(AvatarError::InvalidScene)?;
    write_text(output, "M")?;
    write_point(output, *first)?;
    for point in points {
        write_text(output, " L")?;
        write_point(output, *point)?;
    }
    if path.is_closed() {
        write_text(output, " Z")?;
    }
    Ok(())
}

pub(super) const fn fill_rule_name(fill_rule: FillRule) -> &'static str {
    match fill_rule {
        FillRule::EvenOdd => "evenodd",
        FillRule::NonZero => "nonzero",
    }
}

fn write_stroke(
    output: &mut impl Write,
    prefix: &str,
    index: usize,
    stroke: Stroke,
) -> Result<(), AvatarError> {
    write_paint_attribute(output, prefix, index, "stroke", "stroke", stroke.paint)?;
    write_text(output, " stroke-width=\"")?;
    write_number(output, stroke.width)?;
    write_text(
        output,
        "\" stroke-linecap=\"round\" stroke-linejoin=\"round\"",
    )
}

fn write_paint_definition(
    output: &mut impl Write,
    prefix: &str,
    index: usize,
    role: &str,
    paint: Paint,
) -> Result<(), AvatarError> {
    let Paint::LinearGradient {
        start,
        end,
        start_color,
        end_color,
    } = paint
    else {
        return Ok(());
    };
    write!(output, "<defs><linearGradient id=\"{prefix}-{role}-{index}\" gradientUnits=\"userSpaceOnUse\" x1=\"")
        .map_err(|_| AvatarError::SvgWrite)?;
    write_point_axis(output, start, true)?;
    write_text(output, "\" x2=\"")?;
    write_point_axis(output, end, true)?;
    write_text(output, "\" y1=\"")?;
    write_point_axis(output, start, false)?;
    write_text(output, "\" y2=\"")?;
    write_point_axis(output, end, false)?;
    write_text(output, "\"><stop offset=\"0\"")?;
    write_color_attribute(output, "stop-color", start_color)?;
    write_text(output, "/><stop offset=\"1\"")?;
    write_color_attribute(output, "stop-color", end_color)?;
    write_text(output, "/></linearGradient></defs>")
}

fn write_paint_attribute(
    output: &mut impl Write,
    prefix: &str,
    index: usize,
    role: &str,
    attribute: &str,
    paint: Paint,
) -> Result<(), AvatarError> {
    match paint {
        Paint::Solid(color) => write_color_attribute(output, attribute, color),
        Paint::LinearGradient { .. } => {
            write!(output, " {attribute}=\"url(#{prefix}-{role}-{index})\"")
                .map_err(|_| AvatarError::SvgWrite)
        }
    }
}

fn write_color_attribute(
    output: &mut impl Write,
    attribute: &str,
    color: Color,
) -> Result<(), AvatarError> {
    write!(
        output,
        " {attribute}=\"#{:02x}{:02x}{:02x}\"",
        color.red, color.green, color.blue
    )
    .map_err(|_| AvatarError::SvgWrite)?;
    if color.alpha != u8::MAX {
        let opacity_attribute = match attribute {
            "stop-color" => "stop-opacity",
            "stroke" => "stroke-opacity",
            _ => "fill-opacity",
        };
        write!(output, " {opacity_attribute}=\"").map_err(|_| AvatarError::SvgWrite)?;
        write_opacity(output, color.alpha)?;
        write_text(output, "\"")?;
    }
    Ok(())
}

fn write_opacity(output: &mut impl Write, opacity: u8) -> Result<(), AvatarError> {
    if opacity == 0 {
        return write_text(output, "0");
    }
    if opacity == u8::MAX {
        return write_text(output, "1");
    }
    let scale = 10_000_000_000_000_000_u64;
    let mut fraction = u64::from(opacity)
        .checked_mul(scale)
        .and_then(|value| value.checked_add(127))
        .and_then(|value| value.checked_div(255))
        .ok_or(AvatarError::NumericRange)?;
    let mut digits = 16_usize;
    while fraction % 10 == 0 {
        fraction /= 10;
        digits = digits.checked_sub(1).ok_or(AvatarError::NumericRange)?;
    }
    write!(output, "0.{fraction:0digits$}").map_err(|_| AvatarError::SvgWrite)
}

pub(super) fn write_rect_values(output: &mut impl Write, rect: Rect) -> Result<(), AvatarError> {
    write_number(output, rect.left)?;
    write_text(output, "\" y=\"")?;
    write_number(output, rect.top)?;
    write_text(output, "\" width=\"")?;
    write_number(output, rect.right.checked_sub(rect.left)?)?;
    write_text(output, "\" height=\"")?;
    write_number(output, rect.bottom.checked_sub(rect.top)?)
}

fn write_points(output: &mut impl Write, points: &[Point]) -> Result<(), AvatarError> {
    for (index, point) in points.iter().enumerate() {
        if index > 0 {
            write_text(output, " ")?;
        }
        write_point(output, *point)?;
    }
    Ok(())
}

fn write_point(output: &mut impl Write, point: Point) -> Result<(), AvatarError> {
    write_number(output, point.x)?;
    write_text(output, ",")?;
    write_number(output, point.y)
}

fn write_point_axis(
    output: &mut impl Write,
    point: Point,
    x_axis: bool,
) -> Result<(), AvatarError> {
    write_number(output, if x_axis { point.x } else { point.y })
}

pub(super) fn write_number(output: &mut impl Write, value: Fixed) -> Result<(), AvatarError> {
    write_decimal(output, value).map_err(|_| AvatarError::SvgWrite)
}

pub(super) fn write_text(output: &mut impl Write, value: &str) -> Result<(), AvatarError> {
    output.write_str(value).map_err(|_| AvatarError::SvgWrite)
}

fn write_escaped(output: &mut impl Write, value: &str) -> Result<(), AvatarError> {
    for character in value.chars() {
        write_text(
            output,
            match character {
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                '"' => "&quot;",
                '\'' => "&apos;",
                _ => {
                    output
                        .write_char(character)
                        .map_err(|_| AvatarError::SvgWrite)?;
                    continue;
                }
            },
        )?;
    }
    Ok(())
}

struct SvgBuffer {
    inner: String,
}

impl SvgBuffer {
    fn new() -> Result<Self, AvatarError> {
        let mut inner = String::new();
        inner
            .try_reserve_exact(SVG_CAPACITY)
            .map_err(|_| AvatarError::Allocation)?;
        Ok(Self { inner })
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

#[cfg(test)]
mod tests;
