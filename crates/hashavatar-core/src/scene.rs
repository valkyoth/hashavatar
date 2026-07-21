use crate::{CatError, MAX_DIMENSION, MIN_DIMENSION, RGBA8_BYTES_PER_PIXEL, fixed::Fixed};

pub(crate) const MAX_SCENE_COMMANDS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Color {
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
    pub(crate) alpha: u8,
}

impl Color {
    pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: u8::MAX,
        }
    }

    pub(crate) const fn rgba(self) -> [u8; RGBA8_BYTES_PER_PIXEL] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: Fixed,
    pub(crate) y: Fixed,
}

impl Point {
    pub(crate) const fn new(x: Fixed, y: Fixed) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Empty,
    Fill(Color),
    Ellipse {
        center: Point,
        radius_x: Fixed,
        radius_y: Fixed,
        color: Color,
    },
    Triangle {
        points: [Point; 3],
        color: Color,
    },
}

#[derive(Debug)]
pub(crate) struct Scene {
    width: u32,
    height: u32,
    commands: [Command; MAX_SCENE_COMMANDS],
    command_count: usize,
}

impl Scene {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, CatError> {
        validate_dimensions(width, height)?;
        Ok(Self {
            width,
            height,
            commands: [Command::Empty; MAX_SCENE_COMMANDS],
            command_count: 0,
        })
    }

    pub(crate) fn push(&mut self, command: Command) -> Result<(), CatError> {
        let slot = self
            .commands
            .get_mut(self.command_count)
            .ok_or(CatError::InvalidScene)?;
        *slot = command;
        self.command_count = self
            .command_count
            .checked_add(1)
            .ok_or(CatError::NumericRange)?;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<SceneReport, CatError> {
        validate_dimensions(self.width, self.height)?;
        if self.command_count == 0 || self.command_count > MAX_SCENE_COMMANDS {
            return Err(CatError::InvalidScene);
        }
        let commands = self.commands()?;
        if !matches!(commands.first(), Some(Command::Fill(_)))
            || commands
                .iter()
                .skip(1)
                .any(|command| matches!(command, Command::Fill(_)))
        {
            return Err(CatError::InvalidScene);
        }

        let coordinate_limit = Fixed::from_integer(
            i32::try_from(MAX_DIMENSION)
                .map_err(|_| CatError::NumericRange)?
                .checked_mul(2)
                .ok_or(CatError::NumericRange)?,
        )?;
        let coordinate_minimum = Fixed::ZERO.checked_sub(coordinate_limit)?;
        let mut estimated_pixel_tests = 0_u64;
        for command in commands {
            validate_command(*command, coordinate_minimum, coordinate_limit)?;
            estimated_pixel_tests = estimated_pixel_tests
                .checked_add(command_work(*command, self.width, self.height)?)
                .ok_or(CatError::NumericRange)?;
        }

        let rgba_bytes = usize::try_from(self.width)
            .map_err(|_| CatError::NumericRange)?
            .checked_mul(usize::try_from(self.height).map_err(|_| CatError::NumericRange)?)
            .and_then(|pixels| pixels.checked_mul(RGBA8_BYTES_PER_PIXEL))
            .ok_or(CatError::NumericRange)?;
        Ok(SceneReport {
            command_count: self.command_count,
            estimated_pixel_tests,
            rgba_bytes,
        })
    }

    pub(crate) const fn width(&self) -> u32 {
        self.width
    }

    pub(crate) const fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn commands(&self) -> Result<&[Command], CatError> {
        self.commands
            .get(..self.command_count)
            .ok_or(CatError::InvalidScene)
    }

    #[cfg(test)]
    pub(crate) fn corrupt_first_command(&mut self) {
        if let Some(command) = self.commands.first_mut() {
            *command = Command::Empty;
        }
    }
}

/// Public bounded-work summary for one validated private scene.
#[must_use = "use the scene report to enforce application resource policy"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneReport {
    command_count: usize,
    estimated_pixel_tests: u64,
    rgba_bytes: usize,
}

impl SceneReport {
    /// Returns the number of validated scene commands.
    pub const fn command_count(self) -> usize {
        self.command_count
    }

    /// Returns a conservative count of pixel containment tests.
    pub const fn estimated_pixel_tests(self) -> u64 {
        self.estimated_pixel_tests
    }

    /// Returns the exact tightly packed external RGBA8 byte count.
    pub const fn rgba_bytes(self) -> usize {
        self.rgba_bytes
    }
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), CatError> {
    if (MIN_DIMENSION..=MAX_DIMENSION).contains(&width)
        && (MIN_DIMENSION..=MAX_DIMENSION).contains(&height)
    {
        Ok(())
    } else {
        Err(CatError::UnsupportedDimensions { width, height })
    }
}

fn validate_command(command: Command, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    match command {
        Command::Empty => Err(CatError::InvalidScene),
        Command::Fill(color) => validate_color(color),
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            color,
        } => {
            validate_point(center, minimum, maximum)?;
            if radius_x <= Fixed::ZERO || radius_y <= Fixed::ZERO {
                return Err(CatError::InvalidScene);
            }
            validate_point(
                Point::new(
                    center.x.checked_sub(radius_x)?,
                    center.y.checked_sub(radius_y)?,
                ),
                minimum,
                maximum,
            )?;
            validate_point(
                Point::new(
                    center.x.checked_add(radius_x)?,
                    center.y.checked_add(radius_y)?,
                ),
                minimum,
                maximum,
            )?;
            validate_color(color)
        }
        Command::Triangle { points, color } => {
            for point in points {
                validate_point(point, minimum, maximum)?;
            }
            if triangle_area(points) == 0 {
                return Err(CatError::InvalidScene);
            }
            validate_color(color)
        }
    }
}

fn validate_color(color: Color) -> Result<(), CatError> {
    if color.alpha == u8::MAX {
        Ok(())
    } else {
        Err(CatError::InvalidScene)
    }
}

fn validate_point(point: Point, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    if point.x >= minimum && point.x <= maximum && point.y >= minimum && point.y <= maximum {
        Ok(())
    } else {
        Err(CatError::InvalidScene)
    }
}

pub(crate) fn triangle_area(points: [Point; 3]) -> i128 {
    let Some(a) = points.first() else {
        return 0;
    };
    let Some(b) = points.get(1) else {
        return 0;
    };
    let Some(c) = points.get(2) else {
        return 0;
    };
    let ab_x = i128::from(b.x.raw()) - i128::from(a.x.raw());
    let ab_y = i128::from(b.y.raw()) - i128::from(a.y.raw());
    let ac_x = i128::from(c.x.raw()) - i128::from(a.x.raw());
    let ac_y = i128::from(c.y.raw()) - i128::from(a.y.raw());
    ab_x * ac_y - ab_y * ac_x
}

fn command_work(command: Command, width: u32, height: u32) -> Result<u64, CatError> {
    match command {
        Command::Empty => Err(CatError::InvalidScene),
        Command::Fill(_) => u64::from(width)
            .checked_mul(u64::from(height))
            .ok_or(CatError::NumericRange),
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            ..
        } => bounds_work(
            center.x.checked_sub(radius_x)?,
            center.y.checked_sub(radius_y)?,
            center.x.checked_add(radius_x)?,
            center.y.checked_add(radius_y)?,
            width,
            height,
        ),
        Command::Triangle { points, .. } => {
            let mut min_x = points.first().ok_or(CatError::InvalidScene)?.x;
            let mut max_x = min_x;
            let mut min_y = points.first().ok_or(CatError::InvalidScene)?.y;
            let mut max_y = min_y;
            for point in points.iter().skip(1) {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
                min_y = min_y.min(point.y);
                max_y = max_y.max(point.y);
            }
            bounds_work(min_x, min_y, max_x, max_y, width, height)
        }
    }
}

fn bounds_work(
    min_x: Fixed,
    min_y: Fixed,
    max_x: Fixed,
    max_y: Fixed,
    width: u32,
    height: u32,
) -> Result<u64, CatError> {
    let width_bound = i32::try_from(width).map_err(|_| CatError::NumericRange)?;
    let height_bound = i32::try_from(height).map_err(|_| CatError::NumericRange)?;
    let from_x = min_x.floor()?.clamp(0, width_bound);
    let to_x = max_x.ceil()?.clamp(0, width_bound);
    let from_y = min_y.floor()?.clamp(0, height_bound);
    let to_y = max_y.ceil()?.clamp(0, height_bound);
    let span_x = u64::try_from(to_x.saturating_sub(from_x)).map_err(|_| CatError::NumericRange)?;
    let span_y = u64::try_from(to_y.saturating_sub(from_y)).map_err(|_| CatError::NumericRange)?;
    span_x.checked_mul(span_y).ok_or(CatError::NumericRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_commands_fail_before_execution() {
        let mut scene = match Scene::new(64, 64) {
            Ok(scene) => scene,
            Err(error) => {
                assert_eq!(error, CatError::InvalidScene);
                return;
            }
        };
        assert_eq!(scene.push(Command::Fill(Color::rgb(1, 2, 3))), Ok(()));
        scene.corrupt_first_command();
        assert_eq!(scene.validate(), Err(CatError::InvalidScene));
    }

    #[test]
    fn transparent_scene_paint_is_not_admitted_in_alpha_one() {
        let command = Command::Fill(Color {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 0,
        });
        let limit = Fixed::from_integer(4096);
        assert!(limit.is_ok());
        if let Ok(limit) = limit {
            assert_eq!(
                validate_command(command, Fixed::ZERO, limit),
                Err(CatError::InvalidScene)
            );
        }
    }

    #[test]
    fn background_fill_must_be_unique_and_first() -> Result<(), CatError> {
        let mut scene = Scene::new(64, 64)?;
        scene.push(Command::Fill(Color::rgb(1, 2, 3)))?;
        scene.push(Command::Fill(Color::rgb(4, 5, 6)))?;
        assert_eq!(scene.validate(), Err(CatError::InvalidScene));
        Ok(())
    }
}
