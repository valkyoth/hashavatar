use core::fmt::{self, Write};

use crate::CatError;

pub(crate) const FRACTION_BITS: u32 = 16;
const ONE: i64 = 1_i64 << FRACTION_BITS;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Fixed(i32);

impl Fixed {
    pub(crate) const ZERO: Self = Self(0);

    pub(crate) fn from_integer(value: i32) -> Result<Self, CatError> {
        let raw = i64::from(value)
            .checked_mul(ONE)
            .ok_or(CatError::NumericRange)?;
        Self::from_raw_i64(raw)
    }

    pub(crate) fn from_ratio(numerator: i32, denominator: i32) -> Result<Self, CatError> {
        if denominator == 0 {
            return Err(CatError::NumericRange);
        }
        let raw = i64::from(numerator)
            .checked_mul(ONE)
            .ok_or(CatError::NumericRange)?
            .checked_div(i64::from(denominator))
            .ok_or(CatError::NumericRange)?;
        Self::from_raw_i64(raw)
    }

    pub(crate) fn from_unit_u16(value: u16) -> Result<Self, CatError> {
        let raw = i64::from(value)
            .checked_mul(ONE)
            .ok_or(CatError::NumericRange)?
            .checked_div(i64::from(u16::MAX))
            .ok_or(CatError::NumericRange)?;
        Self::from_raw_i64(raw)
    }

    pub(crate) fn pixel_center(value: u32) -> Result<Self, CatError> {
        let doubled = i64::from(value)
            .checked_mul(2)
            .and_then(|number| number.checked_add(1))
            .ok_or(CatError::NumericRange)?;
        let raw = doubled
            .checked_mul(ONE)
            .and_then(|number| number.checked_div(2))
            .ok_or(CatError::NumericRange)?;
        Self::from_raw_i64(raw)
    }

    pub(crate) fn checked_add(self, other: Self) -> Result<Self, CatError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(CatError::NumericRange)
    }

    pub(crate) fn checked_sub(self, other: Self) -> Result<Self, CatError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(CatError::NumericRange)
    }

    pub(crate) fn checked_mul(self, other: Self) -> Result<Self, CatError> {
        let product = i64::from(self.0)
            .checked_mul(i64::from(other.0))
            .ok_or(CatError::NumericRange)?;
        let biased = if product < 0 {
            product.checked_sub(ONE / 2)
        } else {
            product.checked_add(ONE / 2)
        }
        .ok_or(CatError::NumericRange)?;
        let rounded = biased.checked_div(ONE).ok_or(CatError::NumericRange)?;
        Self::from_raw_i64(rounded)
    }

    pub(crate) fn lerp(minimum: Self, maximum: Self, unit: u16) -> Result<Self, CatError> {
        let span = maximum.checked_sub(minimum)?;
        minimum.checked_add(span.checked_mul(Self::from_unit_u16(unit)?)?)
    }

    pub(crate) const fn raw(self) -> i32 {
        self.0
    }

    pub(crate) fn floor(self) -> Result<i32, CatError> {
        i32::try_from(i64::from(self.0).div_euclid(ONE)).map_err(|_| CatError::NumericRange)
    }

    pub(crate) fn ceil(self) -> Result<i32, CatError> {
        let adjusted = i64::from(self.0)
            .checked_add(ONE - 1)
            .ok_or(CatError::NumericRange)?;
        i32::try_from(adjusted.div_euclid(ONE)).map_err(|_| CatError::NumericRange)
    }

    fn from_raw_i64(raw: i64) -> Result<Self, CatError> {
        i32::try_from(raw)
            .map(Self)
            .map_err(|_| CatError::NumericRange)
    }
}

pub(crate) fn write_decimal(output: &mut impl Write, value: Fixed) -> fmt::Result {
    let raw = i64::from(value.raw());
    let negative = raw < 0;
    let magnitude = raw.unsigned_abs();
    let one = u64::try_from(ONE).map_err(|_| fmt::Error)?;
    let whole = magnitude / one;
    let remainder = magnitude % one;
    if negative {
        output.write_char('-')?;
    }
    write!(output, "{whole}")?;
    if remainder == 0 {
        return Ok(());
    }

    let scale = 10_000_000_000_000_000_u128;
    let fraction = u128::from(remainder)
        .checked_mul(scale)
        .ok_or(fmt::Error)?
        .checked_div(u128::from(one))
        .ok_or(fmt::Error)?;
    let mut digits = alloc::format!("{fraction:016}");
    while digits.ends_with('0') {
        let _ = digits.pop();
    }
    output.write_char('.')?;
    output.write_str(&digits)
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::*;

    #[test]
    fn decimal_format_is_exact_for_q16_values() -> Result<(), CatError> {
        let value = Fixed::from_ratio(1, 8)?;
        let mut output = String::new();
        assert_eq!(write_decimal(&mut output, value), Ok(()));
        assert_eq!(output, "0.125");
        Ok(())
    }

    #[test]
    fn checked_math_rejects_out_of_range_values() {
        assert_eq!(Fixed::from_integer(i32::MAX), Err(CatError::NumericRange));
        assert_eq!(Fixed::from_ratio(1, 0), Err(CatError::NumericRange));
    }
}
