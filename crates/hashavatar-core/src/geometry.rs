use crate::{AvatarError, fixed::Fixed};

pub(crate) const MAX_PATHS: usize = 8;
pub(crate) const MAX_PATH_POINTS: usize = 48;
const CURVE_STEPS: u32 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: Fixed,
    pub(crate) y: Fixed,
}

impl Point {
    pub(crate) const ZERO: Self = Self {
        x: Fixed::ZERO,
        y: Fixed::ZERO,
    };

    pub(crate) const fn new(x: Fixed, y: Fixed) -> Self {
        Self { x, y }
    }

    fn lerp(self, other: Self, unit: u16) -> Result<Self, AvatarError> {
        Ok(Self::new(
            Fixed::lerp(self.x, other.x, unit)?,
            Fixed::lerp(self.y, other.y, unit)?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Rect {
    pub(crate) left: Fixed,
    pub(crate) top: Fixed,
    pub(crate) right: Fixed,
    pub(crate) bottom: Fixed,
}

impl Rect {
    pub(crate) const fn new(left: Fixed, top: Fixed, right: Fixed, bottom: Fixed) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub(crate) fn is_valid(self) -> bool {
        self.left < self.right && self.top < self.bottom
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FillRule {
    EvenOdd,
    NonZero,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Path {
    points: [Point; MAX_PATH_POINTS],
    count: usize,
    closed: bool,
}

impl Path {
    pub(crate) const EMPTY: Self = Self {
        points: [Point::ZERO; MAX_PATH_POINTS],
        count: 0,
        closed: false,
    };

    pub(crate) fn builder(start: Point) -> Result<PathBuilder, AvatarError> {
        let mut path = Self::EMPTY;
        path.push(start)?;
        Ok(PathBuilder { path })
    }

    pub(crate) fn points(&self) -> Result<&[Point], AvatarError> {
        self.points
            .get(..self.count)
            .ok_or(AvatarError::InvalidScene)
    }

    pub(crate) const fn point_count(&self) -> usize {
        self.count
    }

    pub(crate) const fn is_closed(&self) -> bool {
        self.closed
    }

    fn push(&mut self, point: Point) -> Result<(), AvatarError> {
        let slot = self
            .points
            .get_mut(self.count)
            .ok_or(AvatarError::InvalidScene)?;
        *slot = point;
        self.count = self.count.checked_add(1).ok_or(AvatarError::NumericRange)?;
        Ok(())
    }
}

pub(crate) struct PathBuilder {
    path: Path,
}

impl PathBuilder {
    pub(crate) fn line_to(&mut self, end: Point) -> Result<(), AvatarError> {
        self.path.push(end)
    }

    pub(crate) fn quad_to(&mut self, control: Point, end: Point) -> Result<(), AvatarError> {
        let start = *self
            .path
            .points()?
            .last()
            .ok_or(AvatarError::InvalidScene)?;
        for step in 1..=CURVE_STEPS {
            let unit = curve_unit(step)?;
            let first = start.lerp(control, unit)?;
            let second = control.lerp(end, unit)?;
            self.path.push(first.lerp(second, unit)?)?;
        }
        Ok(())
    }

    pub(crate) fn cubic_to(
        &mut self,
        first_control: Point,
        second_control: Point,
        end: Point,
    ) -> Result<(), AvatarError> {
        let start = *self
            .path
            .points()?
            .last()
            .ok_or(AvatarError::InvalidScene)?;
        for step in 1..=CURVE_STEPS {
            let unit = curve_unit(step)?;
            let a = start.lerp(first_control, unit)?;
            let b = first_control.lerp(second_control, unit)?;
            let c = second_control.lerp(end, unit)?;
            let d = a.lerp(b, unit)?;
            let e = b.lerp(c, unit)?;
            self.path.push(d.lerp(e, unit)?)?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self, closed: bool) -> Result<Path, AvatarError> {
        if self.path.count < 2 || (closed && self.path.count < 3) {
            return Err(AvatarError::InvalidScene);
        }
        self.path.closed = closed;
        Ok(self.path)
    }
}

fn curve_unit(step: u32) -> Result<u16, AvatarError> {
    let value = step
        .checked_mul(u32::from(u16::MAX))
        .and_then(|number| number.checked_div(CURVE_STEPS))
        .ok_or(AvatarError::NumericRange)?;
    u16::try_from(value).map_err(|_| AvatarError::NumericRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_lowering_has_a_fixed_bound_and_endpoint() -> Result<(), AvatarError> {
        let zero = Fixed::ZERO;
        let ten = Fixed::from_integer(10)?;
        let mut builder = Path::builder(Point::new(zero, zero))?;
        builder.quad_to(Point::new(ten, zero), Point::new(ten, ten))?;
        let path = builder.finish(false)?;
        assert_eq!(path.point_count(), 9);
        assert_eq!(path.points()?.last(), Some(&Point::new(ten, ten)));
        Ok(())
    }
}
