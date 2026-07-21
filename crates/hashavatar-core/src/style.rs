mod accessory;
mod palette;

pub(crate) use self::palette::resolve_color_roles;
pub use self::{
    accessory::{AccessoryStack, AvatarAccessory, AvatarAccessorySlot, MAX_ACCESSORY_LAYERS},
    palette::{AvatarColorRoles, AvatarPalette, AvatarRgb},
};

use crate::{AvatarBackground, AvatarError, AvatarKind, AvatarShape};

/// Optional expression composed over a family with face anchors.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvatarExpression {
    /// Preserve the family's default expression.
    #[default]
    Default,
    /// Happy mouth overlay.
    Happy,
    /// Grumpy mouth overlay.
    Grumpy,
    /// Surprised mouth overlay.
    Surprised,
    /// Closed sleepy eyes.
    Sleepy,
    /// One closed eye.
    Winking,
    /// Dark eyewear expression.
    Cool,
    /// Grumpy mouth and tear overlay.
    Crying,
}

impl AvatarExpression {
    /// Complete expression catalog in stable identifier order.
    pub const ALL: [Self; 8] = [
        Self::Default,
        Self::Happy,
        Self::Grumpy,
        Self::Surprised,
        Self::Sleepy,
        Self::Winking,
        Self::Cool,
        Self::Crying,
    ];

    /// Selects an expression deterministically from a trait sample.
    pub fn from_sample(value: u16) -> Self {
        Self::ALL
            .iter()
            .copied()
            .nth(usize::from(value) % Self::ALL.len())
            .unwrap_or_default()
    }

    /// Returns the stable catalog identifier.
    pub const fn catalog_id(self) -> u8 {
        self as u8
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Happy => "happy",
            Self::Grumpy => "grumpy",
            Self::Surprised => "surprised",
            Self::Sleepy => "sleepy",
            Self::Winking => "winking",
            Self::Cool => "cool",
            Self::Crying => "crying",
        }
    }
}

/// Compatibility behavior used while resolving requested visual layers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StyleResolutionPolicy {
    /// Reject unsupported, conflicting, or duplicate-slot layers.
    #[default]
    Strict,
    /// Apply the frozen deterministic fallback policy and report every change.
    AutomaticFallback,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum LayerSelection {
    #[default]
    Explicit,
    Automatic,
}

/// Complete requested avatar style, including bounded layered composition.
#[must_use = "pass the style to AvatarRequest"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarStyle {
    kind: AvatarKind,
    background: AvatarBackground,
    shape: AvatarShape,
    palette: AvatarPalette,
    expression: AvatarExpression,
    accessories: AccessoryStack,
    policy: StyleResolutionPolicy,
    selection: LayerSelection,
}

impl AvatarStyle {
    /// Creates a strict explicit style with no optional layers.
    pub const fn new(kind: AvatarKind, background: AvatarBackground, shape: AvatarShape) -> Self {
        Self {
            kind,
            background,
            shape,
            palette: AvatarPalette::Default,
            expression: AvatarExpression::Default,
            accessories: AccessoryStack::new(),
            policy: StyleResolutionPolicy::Strict,
            selection: LayerSelection::Explicit,
        }
    }

    /// Creates a style whose palette, expression, and accessories are derived
    /// from labeled identity traits and resolved with frozen fallback rules.
    pub const fn automatic(
        kind: AvatarKind,
        background: AvatarBackground,
        shape: AvatarShape,
    ) -> Self {
        Self {
            policy: StyleResolutionPolicy::AutomaticFallback,
            selection: LayerSelection::Automatic,
            ..Self::new(kind, background, shape)
        }
    }

    /// Sets an explicit palette and disables automatic layer selection.
    pub const fn with_palette(mut self, palette: AvatarPalette) -> Self {
        self.palette = palette;
        self.selection = LayerSelection::Explicit;
        self
    }

    /// Sets an explicit expression and disables automatic layer selection.
    pub const fn with_expression(mut self, expression: AvatarExpression) -> Self {
        self.expression = expression;
        self.selection = LayerSelection::Explicit;
        self
    }

    /// Replaces the bounded accessory stack and disables automatic selection.
    pub const fn with_accessories(mut self, accessories: AccessoryStack) -> Self {
        self.accessories = accessories;
        self.selection = LayerSelection::Explicit;
        self
    }

    /// Adds one explicit accessory to the bounded stack.
    pub fn with_accessory(mut self, accessory: AvatarAccessory) -> Result<Self, AvatarError> {
        self.accessories.try_push(accessory)?;
        self.selection = LayerSelection::Explicit;
        Ok(self)
    }

    /// Selects strict rejection or deterministic automatic fallback.
    pub const fn with_resolution_policy(mut self, policy: StyleResolutionPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Returns the family.
    pub const fn kind(self) -> AvatarKind {
        self.kind
    }

    /// Returns the background.
    pub const fn background(self) -> AvatarBackground {
        self.background
    }

    /// Returns the frame shape.
    pub const fn shape(self) -> AvatarShape {
        self.shape
    }

    /// Returns the requested explicit palette.
    pub const fn palette(self) -> AvatarPalette {
        self.palette
    }

    /// Returns the requested explicit expression.
    pub const fn expression(self) -> AvatarExpression {
        self.expression
    }

    /// Returns the requested bounded accessory stack.
    pub const fn accessories(self) -> AccessoryStack {
        self.accessories
    }

    /// Returns the requested resolution policy.
    pub const fn resolution_policy(self) -> StyleResolutionPolicy {
        self.policy
    }

    pub(crate) const fn selection(self) -> LayerSelection {
        self.selection
    }
}

impl Default for AvatarStyle {
    fn default() -> Self {
        Self::new(
            AvatarKind::Cat,
            AvatarBackground::Themed,
            AvatarShape::Square,
        )
    }
}
