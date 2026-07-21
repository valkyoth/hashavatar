use crate::{
    CatError, MAX_DIMENSION, MIN_DIMENSION, RGBA8_BYTES_PER_PIXEL,
    fixed::Fixed,
    geometry::{FillRule, MAX_PATHS, Path, Point, Rect},
    paint::Paint,
};

pub(crate) const MAX_SCENE_COMMANDS: usize = 64;
pub(crate) const MAX_STACK_DEPTH: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Stroke {
    pub(crate) width: Fixed,
    pub(crate) paint: Paint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Command {
    Empty,
    Fill(Paint),
    Rect {
        rect: Rect,
        paint: Paint,
    },
    Ellipse {
        center: Point,
        radius_x: Fixed,
        radius_y: Fixed,
        paint: Paint,
    },
    Triangle {
        points: [Point; 3],
        paint: Paint,
    },
    Line {
        start: Point,
        end: Point,
        stroke: Stroke,
    },
    Path {
        path_index: u8,
        fill_rule: FillRule,
        fill: Option<Paint>,
        stroke: Option<Stroke>,
    },
    PushClip(Rect),
    PopClip,
    PushOpacity(u8),
    PopOpacity,
}

#[derive(Debug)]
pub(crate) struct Scene {
    width: u32,
    height: u32,
    commands: [Command; MAX_SCENE_COMMANDS],
    command_count: usize,
    paths: [Path; MAX_PATHS],
    path_count: usize,
}

impl Scene {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, CatError> {
        validate_dimensions(width, height)?;
        Ok(Self {
            width,
            height,
            commands: [Command::Empty; MAX_SCENE_COMMANDS],
            command_count: 0,
            paths: [Path::EMPTY; MAX_PATHS],
            path_count: 0,
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

    pub(crate) fn push_path(&mut self, path: Path) -> Result<u8, CatError> {
        let index = self.path_count;
        let slot = self.paths.get_mut(index).ok_or(CatError::InvalidScene)?;
        *slot = path;
        self.path_count = self
            .path_count
            .checked_add(1)
            .ok_or(CatError::NumericRange)?;
        u8::try_from(index).map_err(|_| CatError::NumericRange)
    }

    pub(crate) fn validate(&self) -> Result<SceneReport, CatError> {
        validate_dimensions(self.width, self.height)?;
        if self.command_count == 0 || self.command_count > MAX_SCENE_COMMANDS {
            return Err(CatError::InvalidScene);
        }
        let commands = self.commands()?;
        if !matches!(commands.first(), Some(Command::Fill(paint)) if paint.is_opaque())
            || commands
                .iter()
                .skip(1)
                .any(|command| matches!(command, Command::Fill(_)))
        {
            return Err(CatError::InvalidScene);
        }

        let limit = coordinate_limit()?;
        let minimum = Fixed::ZERO.checked_sub(limit)?;
        let mut clip_depth = 0_usize;
        let mut opacity_depth = 0_usize;
        let mut maximum_clip_depth = 0_usize;
        let mut maximum_opacity_depth = 0_usize;
        let mut estimated_pixel_tests = 0_u64;
        for command in commands {
            validate_command(*command, self, minimum, limit)?;
            match command {
                Command::PushClip(_) => {
                    clip_depth = push_depth(clip_depth)?;
                    maximum_clip_depth = maximum_clip_depth.max(clip_depth);
                }
                Command::PopClip => clip_depth = pop_depth(clip_depth)?,
                Command::PushOpacity(_) => {
                    opacity_depth = push_depth(opacity_depth)?;
                    maximum_opacity_depth = maximum_opacity_depth.max(opacity_depth);
                }
                Command::PopOpacity => opacity_depth = pop_depth(opacity_depth)?,
                _ => {}
            }
            estimated_pixel_tests = estimated_pixel_tests
                .checked_add(command_work(*command, self)?)
                .ok_or(CatError::NumericRange)?;
        }
        if clip_depth != 0 || opacity_depth != 0 {
            return Err(CatError::InvalidScene);
        }

        Ok(SceneReport {
            command_count: self.command_count,
            path_count: self.path_count,
            path_point_count: self
                .paths()?
                .iter()
                .try_fold(0_usize, |total, path| total.checked_add(path.point_count()))
                .ok_or(CatError::NumericRange)?,
            maximum_clip_depth,
            maximum_opacity_depth,
            estimated_pixel_tests,
            rgba_bytes: rgba_len(self.width, self.height)?,
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

    pub(crate) fn paths(&self) -> Result<&[Path], CatError> {
        self.paths
            .get(..self.path_count)
            .ok_or(CatError::InvalidScene)
    }

    pub(crate) fn path(&self, index: u8) -> Result<&Path, CatError> {
        self.paths()?
            .get(usize::from(index))
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
    path_count: usize,
    path_point_count: usize,
    maximum_clip_depth: usize,
    maximum_opacity_depth: usize,
    estimated_pixel_tests: u64,
    rgba_bytes: usize,
}

impl SceneReport {
    /// Returns the number of validated scene commands.
    pub const fn command_count(self) -> usize {
        self.command_count
    }

    /// Returns the number of validated bounded paths.
    pub const fn path_count(self) -> usize {
        self.path_count
    }

    /// Returns the total number of lowered points across all paths.
    pub const fn path_point_count(self) -> usize {
        self.path_point_count
    }

    /// Returns the maximum nested rectangular clip depth.
    pub const fn maximum_clip_depth(self) -> usize {
        self.maximum_clip_depth
    }

    /// Returns the maximum nested opacity-group depth.
    pub const fn maximum_opacity_depth(self) -> usize {
        self.maximum_opacity_depth
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

pub(crate) fn validate_dimensions(width: u32, height: u32) -> Result<(), CatError> {
    if (MIN_DIMENSION..=MAX_DIMENSION).contains(&width)
        && (MIN_DIMENSION..=MAX_DIMENSION).contains(&height)
    {
        Ok(())
    } else {
        Err(CatError::UnsupportedDimensions { width, height })
    }
}

fn validate_command(
    command: Command,
    scene: &Scene,
    minimum: Fixed,
    maximum: Fixed,
) -> Result<(), CatError> {
    match command {
        Command::Empty | Command::PopClip | Command::PopOpacity => Ok(()),
        Command::Fill(paint) => validate_paint(paint, minimum, maximum),
        Command::Rect { rect, paint } => {
            validate_rect(rect, minimum, maximum)?;
            validate_paint(paint, minimum, maximum)
        }
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            paint,
        } => {
            if radius_x <= Fixed::ZERO || radius_y <= Fixed::ZERO {
                return Err(CatError::InvalidScene);
            }
            validate_rect(
                Rect::new(
                    center.x.checked_sub(radius_x)?,
                    center.y.checked_sub(radius_y)?,
                    center.x.checked_add(radius_x)?,
                    center.y.checked_add(radius_y)?,
                ),
                minimum,
                maximum,
            )?;
            validate_paint(paint, minimum, maximum)
        }
        Command::Triangle { points, paint } => {
            for point in points {
                validate_point(point, minimum, maximum)?;
            }
            if triangle_area(points) == 0 {
                return Err(CatError::InvalidScene);
            }
            validate_paint(paint, minimum, maximum)
        }
        Command::Line { start, end, stroke } => {
            if start == end {
                return Err(CatError::InvalidScene);
            }
            validate_point(start, minimum, maximum)?;
            validate_point(end, minimum, maximum)?;
            validate_stroke(stroke, minimum, maximum)
        }
        Command::Path {
            path_index,
            fill,
            stroke,
            ..
        } => {
            let path = scene.path(path_index)?;
            if fill.is_none() && stroke.is_none() {
                return Err(CatError::InvalidScene);
            }
            if fill.is_some() && !path.is_closed() {
                return Err(CatError::InvalidScene);
            }
            for point in path.points()? {
                validate_point(*point, minimum, maximum)?;
            }
            if let Some(paint) = fill {
                validate_paint(paint, minimum, maximum)?;
            }
            if let Some(stroke) = stroke {
                validate_stroke(stroke, minimum, maximum)?;
            }
            Ok(())
        }
        Command::PushClip(rect) => validate_rect(rect, minimum, maximum),
        Command::PushOpacity(_) => Ok(()),
    }
}

fn validate_paint(paint: Paint, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    paint.validate()?;
    if let Paint::LinearGradient { start, end, .. } = paint {
        validate_point(start, minimum, maximum)?;
        validate_point(end, minimum, maximum)?;
    }
    Ok(())
}

fn validate_stroke(stroke: Stroke, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    if stroke.width <= Fixed::ZERO || stroke.width > maximum {
        return Err(CatError::InvalidScene);
    }
    validate_paint(stroke.paint, minimum, maximum)
}

fn validate_rect(rect: Rect, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    if !rect.is_valid() {
        return Err(CatError::InvalidScene);
    }
    validate_point(Point::new(rect.left, rect.top), minimum, maximum)?;
    validate_point(Point::new(rect.right, rect.bottom), minimum, maximum)
}

fn validate_point(point: Point, minimum: Fixed, maximum: Fixed) -> Result<(), CatError> {
    if point.x >= minimum && point.x <= maximum && point.y >= minimum && point.y <= maximum {
        Ok(())
    } else {
        Err(CatError::InvalidScene)
    }
}

fn push_depth(depth: usize) -> Result<usize, CatError> {
    let next = depth.checked_add(1).ok_or(CatError::NumericRange)?;
    if next > MAX_STACK_DEPTH {
        Err(CatError::InvalidScene)
    } else {
        Ok(next)
    }
}

fn pop_depth(depth: usize) -> Result<usize, CatError> {
    depth.checked_sub(1).ok_or(CatError::InvalidScene)
}

pub(crate) fn triangle_area(points: [Point; 3]) -> i128 {
    let Some(a) = points.first() else { return 0 };
    let Some(b) = points.get(1) else { return 0 };
    let Some(c) = points.get(2) else { return 0 };
    let ab_x = i128::from(b.x.raw()) - i128::from(a.x.raw());
    let ab_y = i128::from(b.y.raw()) - i128::from(a.y.raw());
    let ac_x = i128::from(c.x.raw()) - i128::from(a.x.raw());
    let ac_y = i128::from(c.y.raw()) - i128::from(a.y.raw());
    ab_x * ac_y - ab_y * ac_x
}

fn command_work(command: Command, scene: &Scene) -> Result<u64, CatError> {
    match command {
        Command::Empty => Err(CatError::InvalidScene),
        Command::Fill(_) => full_work(scene),
        Command::Rect { rect, .. } | Command::PushClip(rect) => rect_work(rect, scene),
        Command::Ellipse {
            center,
            radius_x,
            radius_y,
            ..
        } => rect_work(
            Rect::new(
                center.x.checked_sub(radius_x)?,
                center.y.checked_sub(radius_y)?,
                center.x.checked_add(radius_x)?,
                center.y.checked_add(radius_y)?,
            ),
            scene,
        ),
        Command::Triangle { points, .. } => points_work(&points, scene),
        Command::Line { .. } => full_work(scene),
        Command::Path { path_index, .. } => {
            let path = scene.path(path_index)?;
            full_work(scene)?
                .checked_mul(
                    u64::try_from(path.point_count())
                        .map_err(|_| CatError::NumericRange)?
                        .saturating_add(1),
                )
                .ok_or(CatError::NumericRange)
        }
        Command::PopClip | Command::PushOpacity(_) | Command::PopOpacity => Ok(0),
    }
}

fn full_work(scene: &Scene) -> Result<u64, CatError> {
    u64::from(scene.width)
        .checked_mul(u64::from(scene.height))
        .ok_or(CatError::NumericRange)
}

fn points_work(points: &[Point], scene: &Scene) -> Result<u64, CatError> {
    let first = points.first().ok_or(CatError::InvalidScene)?;
    let mut rect = Rect::new(first.x, first.y, first.x, first.y);
    for point in points.iter().skip(1) {
        rect.left = rect.left.min(point.x);
        rect.top = rect.top.min(point.y);
        rect.right = rect.right.max(point.x);
        rect.bottom = rect.bottom.max(point.y);
    }
    if rect.left == rect.right {
        rect.right = rect.right.checked_add(Fixed::from_integer(1)?)?;
    }
    if rect.top == rect.bottom {
        rect.bottom = rect.bottom.checked_add(Fixed::from_integer(1)?)?;
    }
    rect_work(rect, scene)
}

fn rect_work(rect: Rect, scene: &Scene) -> Result<u64, CatError> {
    let width = i32::try_from(scene.width).map_err(|_| CatError::NumericRange)?;
    let height = i32::try_from(scene.height).map_err(|_| CatError::NumericRange)?;
    let span_x = rect
        .right
        .ceil()?
        .clamp(0, width)
        .saturating_sub(rect.left.floor()?.clamp(0, width));
    let span_y = rect
        .bottom
        .ceil()?
        .clamp(0, height)
        .saturating_sub(rect.top.floor()?.clamp(0, height));
    u64::try_from(span_x)
        .ok()
        .and_then(|x| u64::try_from(span_y).ok().and_then(|y| x.checked_mul(y)))
        .ok_or(CatError::NumericRange)
}

fn coordinate_limit() -> Result<Fixed, CatError> {
    Fixed::from_integer(
        i32::try_from(MAX_DIMENSION)
            .map_err(|_| CatError::NumericRange)?
            .checked_mul(2)
            .ok_or(CatError::NumericRange)?,
    )
}

pub(crate) fn rgba_len(width: u32, height: u32) -> Result<usize, CatError> {
    usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(usize::try_from(height).ok()?))
        .and_then(|value| value.checked_mul(RGBA8_BYTES_PER_PIXEL))
        .ok_or(CatError::NumericRange)
}

#[cfg(test)]
mod tests;
