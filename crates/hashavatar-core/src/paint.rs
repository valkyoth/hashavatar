use crate::{AvatarError, geometry::Point};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Color {
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
    pub(crate) alpha: u8,
}

impl Color {
    pub(crate) const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, u8::MAX)
    }

    pub(crate) const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub(crate) const fn channels(self) -> [u8; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    pub(crate) fn with_opacity(self, opacity: u8) -> Self {
        Self {
            alpha: u8::try_from(div_255_round(u32::from(self.alpha) * u32::from(opacity)))
                .unwrap_or(u8::MAX),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Paint {
    Solid(Color),
    LinearGradient {
        start: Point,
        end: Point,
        start_color: Color,
        end_color: Color,
    },
}

impl Paint {
    pub(crate) const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub(crate) fn validate(self) -> Result<(), AvatarError> {
        match self {
            Self::Solid(_) => Ok(()),
            Self::LinearGradient { start, end, .. } if start != end => Ok(()),
            Self::LinearGradient { .. } => Err(AvatarError::InvalidScene),
        }
    }

    pub(crate) fn sample(self, point: Point) -> Result<Color, AvatarError> {
        match self {
            Self::Solid(color) => Ok(color),
            Self::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            } => {
                let dx = i128::from(end.x.raw()) - i128::from(start.x.raw());
                let dy = i128::from(end.y.raw()) - i128::from(start.y.raw());
                let px = i128::from(point.x.raw()) - i128::from(start.x.raw());
                let py = i128::from(point.y.raw()) - i128::from(start.y.raw());
                let denominator = dx
                    .checked_mul(dx)
                    .and_then(|value| value.checked_add(dy.checked_mul(dy)?))
                    .ok_or(AvatarError::NumericRange)?;
                if denominator <= 0 {
                    return Err(AvatarError::InvalidScene);
                }
                let numerator = px
                    .checked_mul(dx)
                    .and_then(|value| value.checked_add(py.checked_mul(dy)?))
                    .ok_or(AvatarError::NumericRange)?
                    .clamp(0, denominator);
                Ok(interpolate_color(
                    start_color,
                    end_color,
                    numerator,
                    denominator,
                ))
            }
        }
    }

    pub(crate) fn with_opacity(self, opacity: u8) -> Self {
        match self {
            Self::Solid(color) => Self::Solid(color.with_opacity(opacity)),
            Self::LinearGradient {
                start,
                end,
                start_color,
                end_color,
            } => Self::LinearGradient {
                start,
                end,
                start_color: start_color.with_opacity(opacity),
                end_color: end_color.with_opacity(opacity),
            },
        }
    }
}

fn interpolate_color(first: Color, second: Color, part: i128, whole: i128) -> Color {
    let channel = |a: u8, b: u8| {
        let a = i128::from(a);
        let delta = i128::from(b) - a;
        let adjustment = if delta < 0 { -(whole / 2) } else { whole / 2 };
        let value = a + (delta * part + adjustment) / whole;
        u8::try_from(value.clamp(0, i128::from(u8::MAX))).unwrap_or_default()
    };
    Color::rgba(
        channel(first.red, second.red),
        channel(first.green, second.green),
        channel(first.blue, second.blue),
        channel(first.alpha, second.alpha),
    )
}

pub(crate) fn source_over(destination: [u8; 4], source: Color) -> [u8; 4] {
    let source_alpha = u32::from(source.alpha);
    if source_alpha == 0 {
        return if destination[3] == 0 {
            Color::TRANSPARENT.channels()
        } else {
            destination
        };
    }
    if source_alpha == u32::from(u8::MAX) {
        return source.channels();
    }
    let destination_alpha = u32::from(destination[3]);
    let inverse = u32::from(u8::MAX) - source_alpha;
    let output_alpha = source_alpha + div_255_round(destination_alpha * inverse);
    if output_alpha == 0 {
        return Color::TRANSPARENT.channels();
    }
    let channel = |source_channel: u8, destination_channel: u8| {
        let source_premultiplied = u32::from(source_channel) * source_alpha;
        let destination_premultiplied = u32::from(destination_channel) * destination_alpha;
        let combined = source_premultiplied + div_255_round(destination_premultiplied * inverse);
        u8::try_from((combined + output_alpha / 2) / output_alpha).unwrap_or(u8::MAX)
    };
    [
        channel(source.red, destination[0]),
        channel(source.green, destination[1]),
        channel(source.blue, destination[2]),
        u8::try_from(output_alpha).unwrap_or(u8::MAX),
    ]
}

pub(crate) const fn div_255_round(value: u32) -> u32 {
    value.saturating_add(127) / 255
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_and_opaque_compositing_have_canonical_edges() {
        let destination = [10, 20, 30, 40];
        assert_eq!(source_over(destination, Color::TRANSPARENT), destination);
        assert_eq!(
            source_over(destination, Color::rgb(1, 2, 3)),
            [1, 2, 3, 255]
        );
    }

    #[test]
    fn half_alpha_source_over_is_exact() {
        assert_eq!(
            source_over([0, 0, 255, 255], Color::rgba(255, 0, 0, 128)),
            [128, 0, 127, 255]
        );
    }
}
