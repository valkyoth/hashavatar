use crate::{AvatarTraitVector, art::util::themed_color};

/// Stable RGB color exposed by resolved palette roles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarRgb {
    channels: [u8; 3],
}

impl AvatarRgb {
    /// Creates an RGB value.
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            channels: [red, green, blue],
        }
    }

    /// Returns `[red, green, blue]` channels.
    pub const fn channels(self) -> [u8; 3] {
        self.channels
    }
}

/// Named integer-only color palette.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvatarPalette {
    /// Identity-derived family colors.
    #[default]
    Default,
    /// Bright mint roles.
    NeonMint,
    /// Soft pink roles.
    PastelPink,
    /// Deep red roles.
    Crimson,
    /// Warm gold roles.
    Gold,
    /// Blue-green roles.
    DeepSeaBlue,
}

impl AvatarPalette {
    /// Complete palette catalog in stable identifier order.
    pub const ALL: [Self; 6] = [
        Self::Default,
        Self::NeonMint,
        Self::PastelPink,
        Self::Crimson,
        Self::Gold,
        Self::DeepSeaBlue,
    ];

    /// Selects a palette deterministically from a trait sample.
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
            Self::NeonMint => "neon-mint",
            Self::PastelPink => "pastel-pink",
            Self::Crimson => "crimson",
            Self::Gold => "gold",
            Self::DeepSeaBlue => "deep-sea-blue",
        }
    }
}

/// Resolved semantic colors consumed by family and layer compilers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarColorRoles {
    primary: AvatarRgb,
    secondary: AvatarRgb,
    accent: AvatarRgb,
    light: AvatarRgb,
    ink: AvatarRgb,
}

impl AvatarColorRoles {
    /// Returns the primary subject color.
    pub const fn primary(self) -> AvatarRgb {
        self.primary
    }
    /// Returns the secondary subject color.
    pub const fn secondary(self) -> AvatarRgb {
        self.secondary
    }
    /// Returns the accent and accessory color.
    pub const fn accent(self) -> AvatarRgb {
        self.accent
    }
    /// Returns the light contrast color.
    pub const fn light(self) -> AvatarRgb {
        self.light
    }
    /// Returns the dark line and detail color.
    pub const fn ink(self) -> AvatarRgb {
        self.ink
    }
}

pub(crate) fn resolve_color_roles(
    palette: AvatarPalette,
    traits: AvatarTraitVector,
) -> AvatarColorRoles {
    let (primary, secondary, accent) = match palette {
        AvatarPalette::Default => (
            from_private(themed_color(traits.primary_hue(), 72, 224, 5)),
            from_private(themed_color(traits.secondary_hue(), 96, 238, 9)),
            from_private(themed_color(traits.accent_hue(), 74, 232, 12)),
        ),
        AvatarPalette::NeonMint => (
            AvatarRgb::new(27, 235, 179),
            AvatarRgb::new(100, 246, 210),
            AvatarRgb::new(10, 145, 118),
        ),
        AvatarPalette::PastelPink => (
            AvatarRgb::new(246, 160, 196),
            AvatarRgb::new(252, 205, 224),
            AvatarRgb::new(190, 82, 132),
        ),
        AvatarPalette::Crimson => (
            AvatarRgb::new(190, 18, 60),
            AvatarRgb::new(244, 99, 128),
            AvatarRgb::new(120, 10, 40),
        ),
        AvatarPalette::Gold => (
            AvatarRgb::new(234, 179, 8),
            AvatarRgb::new(253, 224, 71),
            AvatarRgb::new(161, 98, 7),
        ),
        AvatarPalette::DeepSeaBlue => (
            AvatarRgb::new(14, 116, 144),
            AvatarRgb::new(45, 212, 191),
            AvatarRgb::new(15, 62, 92),
        ),
    };
    AvatarColorRoles {
        primary,
        secondary,
        accent,
        light: AvatarRgb::new(244, 247, 243),
        ink: AvatarRgb::new(25, 29, 36),
    }
}

fn from_private(color: crate::paint::Color) -> AvatarRgb {
    AvatarRgb::new(color.red, color.green, color.blue)
}
