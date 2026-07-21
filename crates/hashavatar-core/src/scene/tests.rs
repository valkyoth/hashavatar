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
fn transparent_background_is_rejected() -> Result<(), CatError> {
    let mut scene = Scene::new(64, 64)?;
    scene.push(Command::Fill(Paint::solid(Color::rgba(1, 2, 3, 0))))?;
    assert_eq!(scene.validate(), Err(CatError::InvalidScene));
    Ok(())
}
