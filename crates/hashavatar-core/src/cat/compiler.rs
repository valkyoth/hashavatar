use super::CatTraitVector;
use crate::{
    AvatarError,
    fixed::Fixed,
    geometry::{FillRule, Path, Point, Rect},
    paint::{Color, Paint},
    scene::{Clip, Command, Scene, Stroke},
};

pub(super) fn compile_scene(
    width: u32,
    height: u32,
    traits: CatTraitVector,
) -> Result<Scene, AvatarError> {
    let width = Fixed::from_integer(i32::try_from(width).map_err(|_| AvatarError::NumericRange)?)?;
    let height =
        Fixed::from_integer(i32::try_from(height).map_err(|_| AvatarError::NumericRange)?)?;
    let center_x = scale(width, 50, 100)?;
    let center_y = scale(height, 54, 100)?.checked_add(vary(height, 0, 4, traits.head_drop)?)?;
    let head_rx = scale(width, 27, 100)?.checked_add(vary(width, 0, 5, traits.head_width)?)?;
    let head_ry = scale(height, 23, 100)?.checked_add(vary(height, 0, 5, traits.head_height)?)?;
    let ear_half = scale(width, 11, 100)?.checked_add(vary(width, 0, 3, traits.ear_width)?)?;
    let ear_height = scale(height, 18, 100)?.checked_add(vary(height, 0, 5, traits.ear_height)?)?;
    let eye_offset = scale(width, 12, 100)?.checked_add(vary(width, 0, 3, traits.eye_spacing)?)?;
    let eye_rx = scale(width, 4, 100)?.checked_add(vary(width, 0, 2, traits.eye_size)?)?;
    let eye_ry = scale(height, 6, 100)?.checked_add(vary(height, 0, 2, traits.eye_size)?)?;
    let eye_y = center_y.checked_sub(scale(head_ry, 18, 100)?)?;
    let ear_base_y = center_y.checked_sub(scale(head_ry, 72, 100)?)?;
    let ear_tip_y = ear_base_y.checked_sub(ear_height)?;
    let left_ear_x = center_x.checked_sub(scale(head_rx, 58, 100)?)?;
    let right_ear_x = center_x.checked_add(scale(head_rx, 58, 100)?)?;

    let background = themed_color(traits.background_hue, 36, 84, 72);
    let background_end = themed_color(traits.background_hue.rotate_left(5), 22, 72, 41);
    let accent = themed_color(traits.accent_hue, 48, 94, 86);
    let fur = themed_color(traits.fur_hue, 54, 210, 188);
    let muzzle = themed_color(traits.muzzle_hue, 168, 246, 222);
    let iris = themed_color(traits.eye_hue, 36, 224, 170);
    let ink = Color::rgb(24, 27, 32);

    let mut scene = Scene::new(
        u32::try_from(width.floor()?).map_err(|_| AvatarError::NumericRange)?,
        u32::try_from(height.floor()?).map_err(|_| AvatarError::NumericRange)?,
    )?;
    scene.push(Command::Fill(Paint::LinearGradient {
        start: Point::new(Fixed::ZERO, Fixed::ZERO),
        end: Point::new(Fixed::ZERO, height),
        start_color: background,
        end_color: background_end,
    }))?;
    scene.push(Command::Rect {
        rect: Rect::new(Fixed::ZERO, scale(height, 88, 100)?, width, height),
        paint: Paint::solid(Color::rgba(255, 255, 255, 36)),
    })?;
    scene.push(Command::PushClip(Clip::Rect(Rect::new(
        Fixed::ZERO,
        Fixed::ZERO,
        scale(width, 95, 100)?,
        height,
    ))))?;
    scene.push(Command::PushOpacity(196))?;
    scene.push(Command::Ellipse {
        center: Point::new(scale(width, 82, 100)?, scale(height, 18, 100)?),
        radius_x: scale(width, 18, 100)?,
        radius_y: scale(height, 18, 100)?,
        paint: Paint::solid(accent),
    })?;
    scene.push(Command::PopOpacity)?;
    scene.push(Command::PopClip)?;
    let geometry = HeadGeometry {
        center_x,
        center_y,
        head_rx,
        head_ry,
        ear_half,
        ear_tip_y,
        ear_base_y,
        left_ear_x,
        right_ear_x,
        eye_offset,
        eye_y,
        eye_rx,
        eye_ry,
    };
    push_head(&mut scene, geometry, fur, muzzle, iris, ink)?;
    push_face_details(&mut scene, width, height, geometry, ink)?;
    Ok(scene)
}

#[derive(Clone, Copy)]
struct HeadGeometry {
    center_x: Fixed,
    center_y: Fixed,
    head_rx: Fixed,
    head_ry: Fixed,
    ear_half: Fixed,
    ear_tip_y: Fixed,
    ear_base_y: Fixed,
    left_ear_x: Fixed,
    right_ear_x: Fixed,
    eye_offset: Fixed,
    eye_y: Fixed,
    eye_rx: Fixed,
    eye_ry: Fixed,
}

fn push_head(
    scene: &mut Scene,
    geometry: HeadGeometry,
    fur: Color,
    muzzle: Color,
    iris: Color,
    ink: Color,
) -> Result<(), AvatarError> {
    for ear_x in [geometry.left_ear_x, geometry.right_ear_x] {
        scene.push(Command::Triangle {
            points: [
                Point::new(ear_x.checked_sub(geometry.ear_half)?, geometry.ear_base_y),
                Point::new(ear_x, geometry.ear_tip_y),
                Point::new(ear_x.checked_add(geometry.ear_half)?, geometry.ear_base_y),
            ],
            paint: Paint::solid(fur),
        })?;
    }
    scene.push(Command::Ellipse {
        center: Point::new(geometry.center_x, geometry.center_y),
        radius_x: geometry.head_rx,
        radius_y: geometry.head_ry,
        paint: Paint::solid(fur),
    })?;
    scene.push(Command::Ellipse {
        center: Point::new(
            geometry.center_x,
            geometry
                .center_y
                .checked_add(scale(geometry.head_ry, 35, 100)?)?,
        ),
        radius_x: scale(geometry.head_rx, 38, 100)?,
        radius_y: scale(geometry.head_ry, 28, 100)?,
        paint: Paint::solid(muzzle),
    })?;
    for direction in [-1_i32, 1_i32] {
        let offset = if direction < 0 {
            Fixed::ZERO.checked_sub(geometry.eye_offset)?
        } else {
            geometry.eye_offset
        };
        let eye_x = geometry.center_x.checked_add(offset)?;
        for (radius_x, radius_y, color) in [
            (geometry.eye_rx, geometry.eye_ry, Color::rgb(248, 250, 252)),
            (
                scale(geometry.eye_rx, 48, 100)?,
                scale(geometry.eye_ry, 62, 100)?,
                iris,
            ),
            (
                scale(geometry.eye_rx, 18, 100)?,
                scale(geometry.eye_ry, 44, 100)?,
                ink,
            ),
        ] {
            scene.push(Command::Ellipse {
                center: Point::new(eye_x, geometry.eye_y),
                radius_x,
                radius_y,
                paint: Paint::solid(color),
            })?;
        }
    }
    Ok(())
}

fn push_face_details(
    scene: &mut Scene,
    width: Fixed,
    height: Fixed,
    geometry: HeadGeometry,
    ink: Color,
) -> Result<(), AvatarError> {
    let center_x = geometry.center_x;
    let nose_y = geometry
        .center_y
        .checked_add(scale(geometry.head_ry, 31, 100)?)?;
    let mut nose = Path::builder(Point::new(
        center_x.checked_sub(scale(width, 3, 100)?)?,
        nose_y,
    ))?;
    nose.line_to(Point::new(
        center_x.checked_add(scale(width, 3, 100)?)?,
        nose_y,
    ))?;
    nose.line_to(Point::new(
        center_x,
        nose_y.checked_add(scale(height, 3, 100)?)?,
    ))?;
    let nose_index = scene.push_path(nose.finish(true)?)?;
    scene.push(Command::Path {
        path_index: nose_index,
        fill_rule: FillRule::NonZero,
        fill: Some(Paint::solid(ink)),
        stroke: None,
    })?;

    let mouth_y = nose_y.checked_add(scale(height, 4, 100)?)?;
    let mouth_half = scale(width, 8, 100)?;
    let mouth_low = mouth_y.checked_add(scale(height, 5, 100)?)?;
    let mut mouth = Path::builder(Point::new(center_x.checked_sub(mouth_half)?, mouth_y))?;
    mouth.quad_to(
        Point::new(
            center_x.checked_sub(scale(mouth_half, 35, 100)?)?,
            mouth_low,
        ),
        Point::new(center_x, mouth_low),
    )?;
    mouth.cubic_to(
        Point::new(
            center_x.checked_add(scale(mouth_half, 25, 100)?)?,
            mouth_low,
        ),
        Point::new(
            center_x.checked_add(scale(mouth_half, 70, 100)?)?,
            mouth_low,
        ),
        Point::new(center_x.checked_add(mouth_half)?, mouth_y),
    )?;
    let mouth_index = scene.push_path(mouth.finish(false)?)?;
    scene.push(Command::Path {
        path_index: mouth_index,
        fill_rule: FillRule::EvenOdd,
        fill: None,
        stroke: Some(Stroke {
            width: scale(width.min(height), 1, 100)?,
            paint: Paint::solid(ink),
        }),
    })?;

    let whisker_width = scale(width.min(height), 1, 160)?;
    for direction in [-1_i32, 1_i32] {
        let horizontal = signed_unit(direction)?;
        let inner_x =
            center_x.checked_add(scale(geometry.head_rx, 30, 100)?.checked_mul(horizontal)?)?;
        let outer_x =
            center_x.checked_add(scale(geometry.head_rx, 86, 100)?.checked_mul(horizontal)?)?;
        for vertical_direction in [-1_i32, 1_i32] {
            let vertical = signed_unit(vertical_direction)?;
            scene.push(Command::Line {
                start: Point::new(
                    inner_x,
                    mouth_y.checked_add(scale(height, 2, 100)?.checked_mul(vertical)?)?,
                ),
                end: Point::new(
                    outer_x,
                    mouth_y.checked_add(scale(height, 5, 100)?.checked_mul(vertical)?)?,
                ),
                stroke: Stroke {
                    width: whisker_width,
                    paint: Paint::solid(ink),
                },
            })?;
        }
    }
    Ok(())
}

fn signed_unit(direction: i32) -> Result<Fixed, AvatarError> {
    if direction < 0 {
        Fixed::ZERO.checked_sub(Fixed::from_integer(1)?)
    } else {
        Fixed::from_integer(1)
    }
}

fn scale(value: Fixed, numerator: i32, denominator: i32) -> Result<Fixed, AvatarError> {
    value.checked_mul(Fixed::from_ratio(numerator, denominator)?)
}

fn vary(value: Fixed, minimum: i32, maximum: i32, sample: u16) -> Result<Fixed, AvatarError> {
    Fixed::lerp(
        scale(value, minimum, 100)?,
        scale(value, maximum, 100)?,
        sample,
    )
}

fn themed_color(sample: u16, floor: u8, ceiling: u8, phase: u8) -> Color {
    let span = u16::from(ceiling.saturating_sub(floor));
    let channel = |shift: u16| {
        let mixed = sample.rotate_left(u32::from(shift % 16));
        let scaled = u32::from(mixed)
            .saturating_mul(u32::from(span))
            .checked_div(u32::from(u16::MAX))
            .unwrap_or(0);
        floor.saturating_add(u8::try_from(scaled).unwrap_or(u8::MAX))
    };
    Color::rgb(channel(0), channel(u16::from(phase % 13)), channel(11))
}
