use super::util::{Canvas, themed_color};
use crate::{
    AvatarBackground, AvatarError, AvatarPalette, AvatarTraitVector, ResolvedStyle,
    geometry::Point,
    paint::{Color, Paint},
    scene::{Command, Scene, Stroke},
};

pub(super) fn compile(
    scene: &mut Scene,
    style: ResolvedStyle,
    traits: AvatarTraitVector,
) -> Result<(), AvatarError> {
    let canvas = Canvas::new(scene)?;
    match style.background() {
        AvatarBackground::Transparent => Ok(()),
        AvatarBackground::Themed if matches!(style.palette(), AvatarPalette::Default) => {
            push_gradient(
                scene,
                canvas,
                themed_color(traits.primary_hue(), 52, 224, 7),
                themed_color(traits.secondary_hue(), 28, 172, 11),
            )
        }
        AvatarBackground::Themed => push_gradient(
            scene,
            canvas,
            super::util::role_color(style.color_roles().secondary()),
            super::util::role_color(style.color_roles().primary()),
        ),
        AvatarBackground::Sunrise => push_gradient(
            scene,
            canvas,
            Color::rgb(255, 247, 212),
            Color::rgb(255, 154, 92),
        ),
        AvatarBackground::Ocean => push_gradient(
            scene,
            canvas,
            Color::rgb(214, 248, 252),
            Color::rgb(42, 126, 176),
        ),
        AvatarBackground::White => push_solid(scene, canvas, Color::rgb(255, 255, 255)),
        AvatarBackground::Black => push_solid(scene, canvas, Color::rgb(0, 0, 0)),
        AvatarBackground::Dark => push_solid(scene, canvas, Color::rgb(28, 32, 39)),
        AvatarBackground::Light => push_solid(scene, canvas, Color::rgb(247, 249, 246)),
        AvatarBackground::PolkaDot => push_polka(scene, canvas, traits),
        AvatarBackground::Striped => push_stripes(scene, canvas, traits),
        AvatarBackground::Checkerboard => push_checkerboard(scene, canvas),
        AvatarBackground::Grid => push_grid(scene, canvas),
        AvatarBackground::Starry => push_stars(scene, canvas, traits.pattern_seed()),
    }
}

fn push_solid(scene: &mut Scene, canvas: Canvas, color: Color) -> Result<(), AvatarError> {
    scene.push(Command::Rect {
        rect: canvas.rect(),
        paint: Paint::solid(color),
    })
}

fn push_gradient(
    scene: &mut Scene,
    canvas: Canvas,
    top: Color,
    bottom: Color,
) -> Result<(), AvatarError> {
    scene.push(Command::Rect {
        rect: canvas.rect(),
        paint: Paint::LinearGradient {
            start: Point::new(crate::fixed::Fixed::ZERO, crate::fixed::Fixed::ZERO),
            end: Point::new(crate::fixed::Fixed::ZERO, canvas.height),
            start_color: top,
            end_color: bottom,
        },
    })
}

fn push_polka(
    scene: &mut Scene,
    canvas: Canvas,
    traits: AvatarTraitVector,
) -> Result<(), AvatarError> {
    push_solid(scene, canvas, Color::rgb(248, 250, 247))?;
    let dot = themed_color(traits.accent_hue(), 80, 210, 5).with_opacity(82);
    for row in 0_i32..5 {
        for column in 0_i32..5 {
            scene.push(Command::Ellipse {
                center: Point::new(canvas.x(column * 20 + 10)?, canvas.y(row * 20 + 10)?),
                radius_x: canvas.s(2)?,
                radius_y: canvas.s(2)?,
                paint: Paint::solid(dot),
            })?;
        }
    }
    Ok(())
}

fn push_stripes(
    scene: &mut Scene,
    canvas: Canvas,
    traits: AvatarTraitVector,
) -> Result<(), AvatarError> {
    push_solid(scene, canvas, Color::rgb(248, 250, 247))?;
    let stroke = Stroke {
        width: canvas.s(6)?,
        paint: Paint::solid(themed_color(traits.accent_hue(), 90, 190, 9).with_opacity(58)),
    };
    for index in -1_i32..7 {
        scene.push(Command::Line {
            start: Point::new(canvas.x(index * 20)?, crate::fixed::Fixed::ZERO),
            end: Point::new(canvas.x(index * 20 + 55)?, canvas.height),
            stroke,
        })?;
    }
    Ok(())
}

fn push_checkerboard(scene: &mut Scene, canvas: Canvas) -> Result<(), AvatarError> {
    push_solid(scene, canvas, Color::rgb(248, 250, 247))?;
    for row in 0_i32..6 {
        for column in 0_i32..6 {
            if (row + column) % 2 == 0 {
                scene.push(Command::Rect {
                    rect: crate::geometry::Rect::new(
                        canvas.x(column * 17)?,
                        canvas.y(row * 17)?,
                        canvas.x((column + 1) * 17)?,
                        canvas.y((row + 1) * 17)?,
                    ),
                    paint: Paint::solid(Color::rgb(226, 232, 226)),
                })?;
            }
        }
    }
    Ok(())
}

fn push_grid(scene: &mut Scene, canvas: Canvas) -> Result<(), AvatarError> {
    push_solid(scene, canvas, Color::rgb(248, 250, 247))?;
    let stroke = Stroke {
        width: canvas.s(1)?,
        paint: Paint::solid(Color::rgb(214, 222, 216)),
    };
    for index in 1_i32..6 {
        let x = canvas.x(index * 17)?;
        let y = canvas.y(index * 17)?;
        scene.push(Command::Line {
            start: Point::new(x, crate::fixed::Fixed::ZERO),
            end: Point::new(x, canvas.height),
            stroke,
        })?;
        scene.push(Command::Line {
            start: Point::new(crate::fixed::Fixed::ZERO, y),
            end: Point::new(canvas.width, y),
            stroke,
        })?;
    }
    Ok(())
}

fn push_stars(scene: &mut Scene, canvas: Canvas, seed: u16) -> Result<(), AvatarError> {
    push_solid(scene, canvas, Color::rgb(17, 24, 39))?;
    let mut state = u32::from(seed) ^ 0x9e37_79b9;
    for index in 0_u32..16 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let x = i32::try_from(5 + state % 91).map_err(|_| AvatarError::NumericRange)?;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let y = i32::try_from(5 + state % 91).map_err(|_| AvatarError::NumericRange)?;
        let radius = if index % 5 == 0 { 2 } else { 1 };
        scene.push(Command::Ellipse {
            center: Point::new(canvas.x(x)?, canvas.y(y)?),
            radius_x: canvas.s(radius)?,
            radius_y: canvas.s(radius)?,
            paint: Paint::solid(Color::rgba(255, 255, 255, 190)),
        })?;
    }
    Ok(())
}
