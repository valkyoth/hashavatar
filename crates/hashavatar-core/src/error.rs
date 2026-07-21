use core::fmt;

use crate::{AvatarAccessory, AvatarAccessorySlot, AvatarExpression};

/// Identity component rejected by a bounded constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityComponent {
    /// Raw caller identity bytes.
    Input,
    /// Tenant namespace bytes.
    Tenant,
    /// Style-version namespace bytes.
    StyleVersion,
}

impl IdentityComponent {
    const fn description(self) -> &'static str {
        match self {
            Self::Input => "identity input",
            Self::Tenant => "namespace tenant",
            Self::StyleVersion => "namespace style version",
        }
    }
}

/// Error returned by canonical avatar preparation or execution.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatError {
    /// Width or height is outside the supported range.
    UnsupportedDimensions {
        /// Rejected width.
        width: u32,
        /// Rejected height.
        height: u32,
    },
    /// One identity component exceeds its public byte limit.
    IdentityComponentTooLong {
        /// Rejected component category.
        component: IdentityComponent,
        /// Maximum accepted byte length.
        maximum: usize,
    },
    /// A bounded allocation could not be completed.
    Allocation,
    /// Fixed-point request compilation exceeded the admitted numeric range.
    NumericRange,
    /// The private scene failed validation before execution.
    InvalidScene,
    /// A caller-provided RGBA8 surface has an invalid stride or buffer length.
    InvalidSurface,
    /// SVG document or fragment options failed validation.
    InvalidSvgOptions,
    /// Writing the internally bounded SVG string failed.
    SvgWrite,
    /// A bounded accessory stack exceeded its fixed capacity.
    AccessoryCapacity {
        /// Maximum admitted accessory count.
        maximum: usize,
    },
    /// The family has no compatible anchors for this accessory.
    UnsupportedAccessory {
        /// Rejected accessory.
        accessory: AvatarAccessory,
    },
    /// Two requested accessories require the same semantic slot.
    AccessorySlotConflict {
        /// Duplicated slot.
        slot: AvatarAccessorySlot,
    },
    /// An accessory overlaps an already admitted exclusion zone.
    AccessoryCollision {
        /// Rejected accessory slot.
        slot: AvatarAccessorySlot,
    },
    /// The family has no compatible anchors for this expression.
    UnsupportedExpression {
        /// Rejected expression.
        expression: AvatarExpression,
    },
    /// The expression conflicts with an admitted face layer.
    ExpressionCollision {
        /// Rejected expression.
        expression: AvatarExpression,
    },
}

impl fmt::Display for CatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDimensions { width, height } => write!(
                formatter,
                "canonical avatar dimensions must be between 64 and 2048 pixels, got {width}x{height}"
            ),
            Self::IdentityComponentTooLong { component, maximum } => write!(
                formatter,
                "{} exceeds the maximum of {maximum} bytes",
                component.description()
            ),
            Self::Allocation => formatter.write_str("bounded avatar allocation failed"),
            Self::NumericRange => formatter.write_str("avatar geometry exceeded numeric limits"),
            Self::InvalidScene => formatter.write_str("canonical avatar scene validation failed"),
            Self::InvalidSurface => formatter.write_str("caller RGBA8 surface is invalid"),
            Self::InvalidSvgOptions => formatter.write_str("canonical SVG options are invalid"),
            Self::SvgWrite => formatter.write_str("canonical SVG construction failed"),
            Self::AccessoryCapacity { maximum } => {
                write!(
                    formatter,
                    "accessory stack exceeds the maximum of {maximum}"
                )
            }
            Self::UnsupportedAccessory { accessory } => write!(
                formatter,
                "accessory {} is unsupported by this avatar family",
                accessory.as_str()
            ),
            Self::AccessorySlotConflict { slot } => write!(
                formatter,
                "multiple accessories require the {} slot",
                slot.as_str()
            ),
            Self::AccessoryCollision { slot } => write!(
                formatter,
                "accessory in the {} slot intersects an exclusion zone",
                slot.as_str()
            ),
            Self::UnsupportedExpression { expression } => write!(
                formatter,
                "expression {} is unsupported by this avatar family",
                expression.as_str()
            ),
            Self::ExpressionCollision { expression } => write!(
                formatter,
                "expression {} conflicts with an admitted face layer",
                expression.as_str()
            ),
        }
    }
}

impl core::error::Error for CatError {}
