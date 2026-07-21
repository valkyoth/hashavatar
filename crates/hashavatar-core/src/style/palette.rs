use crate::{AvatarKind, AvatarTraitVector};

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
    kind: AvatarKind,
) -> AvatarColorRoles {
    let (primary, secondary, accent) = match palette {
        AvatarPalette::Default => default_family_colors(kind, traits),
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
        light: family_light(kind),
        ink: family_ink(kind),
    }
}

fn default_family_colors(
    kind: AvatarKind,
    traits: AvatarTraitVector,
) -> (AvatarRgb, AvatarRgb, AvatarRgb) {
    let colors = match kind {
        AvatarKind::Cat => ([194, 139, 82], [239, 215, 184], [219, 143, 151]),
        AvatarKind::Dog => ([187, 132, 75], [117, 73, 45], [219, 151, 62]),
        AvatarKind::Robot => ([180, 195, 203], [79, 99, 113], [154, 205, 67]),
        AvatarKind::Fox => ([225, 111, 42], [248, 234, 205], [119, 67, 37]),
        AvatarKind::Alien => ([132, 205, 127], [84, 145, 88], [178, 223, 112]),
        AvatarKind::Monster => ([119, 179, 91], [70, 115, 77], [219, 153, 77]),
        AvatarKind::Ghost => ([238, 243, 246], [176, 190, 201], [124, 151, 174]),
        AvatarKind::Slime => ([111, 188, 80], [67, 132, 62], [189, 220, 86]),
        AvatarKind::Bird => ([92, 157, 193], [56, 111, 153], [232, 158, 54]),
        AvatarKind::Wizard => ([73, 64, 133], [222, 181, 148], [232, 175, 55]),
        AvatarKind::Skull => ([226, 222, 205], [177, 171, 151], [116, 108, 91]),
        AvatarKind::Paws => ([173, 126, 91], [220, 157, 164], [190, 96, 113]),
        AvatarKind::Planet => ([75, 133, 190], [223, 181, 94], [82, 176, 159]),
        AvatarKind::Rocket => ([213, 220, 224], [193, 68, 70], [239, 145, 51]),
        AvatarKind::Mushroom => ([199, 65, 69], [239, 221, 180], [247, 240, 218]),
        AvatarKind::Cactus => ([66, 145, 82], [43, 107, 65], [220, 94, 153]),
        AvatarKind::Frog => ([99, 178, 80], [58, 125, 65], [225, 142, 157]),
        AvatarKind::Panda => ([42, 47, 50], [239, 238, 229], [112, 162, 112]),
        AvatarKind::Cupcake => ([93, 151, 174], [234, 157, 186], [207, 57, 89]),
        AvatarKind::Pizza => ([201, 126, 64], [239, 199, 94], [194, 57, 50]),
        AvatarKind::Icecream => ([235, 151, 178], [205, 161, 98], [120, 190, 173]),
        AvatarKind::Octopus => ([165, 102, 181], [105, 65, 128], [231, 151, 174]),
        AvatarKind::Knight => ([161, 172, 181], [103, 116, 127], [183, 59, 67]),
        AvatarKind::Bear => ([139, 91, 58], [209, 169, 119], [188, 119, 71]),
        AvatarKind::Penguin => ([37, 47, 56], [238, 241, 235], [231, 151, 55]),
        AvatarKind::Dragon => ([71, 151, 83], [167, 195, 105], [224, 151, 64]),
        AvatarKind::Ninja => ([191, 55, 67], [217, 171, 137], [70, 98, 151]),
        AvatarKind::Astronaut => ([78, 151, 181], [217, 224, 228], [221, 90, 67]),
        AvatarKind::Diamond => ([63, 175, 201], [139, 224, 232], [46, 115, 164]),
        AvatarKind::CoffeeCup => ([174, 111, 70], [91, 57, 42], [217, 161, 91]),
        AvatarKind::Shield => ([119, 143, 166], [184, 198, 208], [83, 171, 61]),
    };
    (
        varied(colors.0, traits.primary_hue(), 12),
        varied(colors.1, traits.secondary_hue(), 8),
        varied(colors.2, traits.accent_hue(), 8),
    )
}

fn varied(channels: [u8; 3], sample: u16, range: i16) -> AvatarRgb {
    let offset =
        i32::from(sample) * i32::from(range * 2 + 1) / i32::from(u16::MAX) - i32::from(range);
    let adjust =
        |channel: u8| u8::try_from((i32::from(channel) + offset).clamp(0, 255)).unwrap_or(channel);
    AvatarRgb::new(
        adjust(channels[0]),
        adjust(channels[1]),
        adjust(channels[2]),
    )
}

const fn family_light(kind: AvatarKind) -> AvatarRgb {
    match kind {
        AvatarKind::Wizard => AvatarRgb::new(232, 229, 218),
        AvatarKind::Skull => AvatarRgb::new(230, 226, 210),
        AvatarKind::Panda | AvatarKind::Penguin => AvatarRgb::new(244, 244, 237),
        _ => AvatarRgb::new(248, 248, 241),
    }
}

const fn family_ink(kind: AvatarKind) -> AvatarRgb {
    match kind {
        AvatarKind::Cat | AvatarKind::Dog | AvatarKind::Bear => AvatarRgb::new(57, 42, 31),
        AvatarKind::Fox => AvatarRgb::new(48, 37, 29),
        AvatarKind::Alien => AvatarRgb::new(38, 24, 50),
        AvatarKind::Ghost => AvatarRgb::new(48, 56, 74),
        AvatarKind::Slime | AvatarKind::Frog => AvatarRgb::new(35, 72, 35),
        AvatarKind::Dragon => AvatarRgb::new(24, 48, 34),
        _ => AvatarRgb::new(25, 29, 36),
    }
}
