use crate::{
    CatError,
    fixed::Fixed,
    geometry::{Point, Rect},
    paint::Color,
    scene::Scene,
};

#[derive(Clone, Copy)]
pub(super) struct Canvas {
    pub(super) width: Fixed,
    pub(super) height: Fixed,
    pub(super) center: Point,
    pub(super) minimum: Fixed,
}

impl Canvas {
    pub(super) fn new(scene: &Scene) -> Result<Self, CatError> {
        let width =
            Fixed::from_integer(i32::try_from(scene.width()).map_err(|_| CatError::NumericRange)?)?;
        let height = Fixed::from_integer(
            i32::try_from(scene.height()).map_err(|_| CatError::NumericRange)?,
        )?;
        Ok(Self {
            width,
            height,
            center: Point::new(scale(width, 1, 2)?, scale(height, 1, 2)?),
            minimum: width.min(height),
        })
    }

    pub(super) fn x(self, percent: i32) -> Result<Fixed, CatError> {
        scale(self.width, percent, 100)
    }

    pub(super) fn y(self, percent: i32) -> Result<Fixed, CatError> {
        scale(self.height, percent, 100)
    }

    pub(super) fn s(self, percent: i32) -> Result<Fixed, CatError> {
        scale(self.minimum, percent, 100)
    }

    pub(super) const fn rect(self) -> Rect {
        Rect::new(Fixed::ZERO, Fixed::ZERO, self.width, self.height)
    }
}

pub(super) fn scale(value: Fixed, numerator: i32, denominator: i32) -> Result<Fixed, CatError> {
    value.checked_mul(Fixed::from_ratio(numerator, denominator)?)
}

pub(super) fn vary(
    value: Fixed,
    minimum: i32,
    maximum: i32,
    sample: u16,
) -> Result<Fixed, CatError> {
    Fixed::lerp(
        scale(value, minimum, 100)?,
        scale(value, maximum, 100)?,
        sample,
    )
}

pub(super) fn themed_color(sample: u16, floor: u8, ceiling: u8, phase: u8) -> Color {
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
