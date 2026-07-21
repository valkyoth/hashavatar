use super::*;
use crate::paint::Color;

#[test]
fn malformed_commands_fail_before_execution() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    scene.push(Command::Fill(Paint::solid(Color::rgb(1, 2, 3))))?;
    scene.corrupt_first_command();
    assert_eq!(scene.validate(), Err(CatError::InvalidScene));
    Ok(())
}

#[test]
fn stacks_must_balance() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    scene.push(Command::Fill(Paint::solid(Color::rgb(1, 2, 3))))?;
    scene.push(Command::PushOpacity(128))?;
    assert_eq!(scene.validate(), Err(CatError::InvalidScene));
    Ok(())
}

#[test]
fn stack_underflow_is_rejected() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    scene.push(Command::Fill(Paint::solid(Color::rgb(1, 2, 3))))?;
    scene.push(Command::PopClip)?;
    assert_eq!(scene.validate(), Err(CatError::InvalidScene));
    Ok(())
}

#[test]
fn transparent_background_is_valid_canonical_clear() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    scene.push(Command::Fill(Paint::solid(Color::TRANSPARENT)))?;
    assert!(scene.validate().is_ok());
    Ok(())
}

#[test]
fn clip_predicates_are_charged_for_every_enclosed_candidate_pixel() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    let paint = Paint::solid(Color::rgb(1, 2, 3));
    let zero = Fixed::ZERO;
    let edge = Fixed::from_integer(64)?;
    let center = Point::new(Fixed::from_integer(32)?, Fixed::from_integer(32)?);
    scene.push(Command::Fill(paint))?;
    scene.push(Command::PushClip(Clip::Ellipse {
        center,
        radius_x: Fixed::from_integer(32)?,
        radius_y: Fixed::from_integer(32)?,
    }))?;
    scene.push(Command::Rect {
        rect: Rect::new(zero, zero, edge, edge),
        paint,
    })?;
    scene.push(Command::Rect {
        rect: Rect::new(zero, zero, edge, edge),
        paint,
    })?;
    scene.push(Command::PopClip)?;
    scene.push(Command::Rect {
        rect: Rect::new(zero, zero, edge, edge),
        paint,
    })?;

    let full_canvas = 64_u64 * 64;
    assert_eq!(
        scene.validate()?.estimated_pixel_tests(),
        full_canvas + (full_canvas * 2) * 2 + full_canvas
    );
    Ok(())
}

#[test]
fn path_clip_edges_are_charged_for_each_enclosed_command() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    let paint = Paint::solid(Color::rgb(1, 2, 3));
    let zero = Fixed::ZERO;
    let edge = Fixed::from_integer(64)?;
    let mut builder = Path::builder(Point::new(zero, zero))?;
    builder.line_to(Point::new(edge, zero))?;
    builder.line_to(Point::new(edge, edge))?;
    let path_index = scene.push_path(builder.finish(true)?)?;

    scene.push(Command::Fill(paint))?;
    scene.push(Command::PushClip(Clip::Path {
        path_index,
        fill_rule: FillRule::EvenOdd,
    }))?;
    for _ in 0..2 {
        scene.push(Command::Rect {
            rect: Rect::new(zero, zero, edge, edge),
            paint,
        })?;
    }
    scene.push(Command::PopClip)?;

    let full_canvas = 64_u64 * 64;
    let command_and_three_edge_tests = full_canvas * 4;
    assert_eq!(
        scene.validate()?.estimated_pixel_tests(),
        full_canvas + command_and_three_edge_tests * 2
    );
    Ok(())
}
