/// Built-in avatar family.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarKind {
    /// Cat face.
    #[default]
    Cat,
    /// Dog face.
    Dog,
    /// Robot head.
    Robot,
    /// Fox face.
    Fox,
    /// Alien face.
    Alien,
    /// Monster face.
    Monster,
    /// Ghost.
    Ghost,
    /// Slime creature.
    Slime,
    /// Bird.
    Bird,
    /// Wizard.
    Wizard,
    /// Skull.
    Skull,
    /// Paw print.
    Paws,
    /// Ringed planet.
    Planet,
    /// Rocket.
    Rocket,
    /// Mushroom.
    Mushroom,
    /// Cactus.
    Cactus,
    /// Frog face.
    Frog,
    /// Panda face.
    Panda,
    /// Cupcake.
    Cupcake,
    /// Pizza slice.
    Pizza,
    /// Ice cream cone.
    Icecream,
    /// Octopus.
    Octopus,
    /// Knight helmet.
    Knight,
    /// Bear face.
    Bear,
    /// Penguin.
    Penguin,
    /// Dragon.
    Dragon,
    /// Ninja.
    Ninja,
    /// Astronaut.
    Astronaut,
    /// Diamond.
    Diamond,
    /// Coffee cup.
    CoffeeCup,
    /// Shield.
    Shield,
}

impl AvatarKind {
    /// Complete catalog in frozen 1.x identifier order.
    pub const ALL: [Self; 31] = [
        Self::Cat,
        Self::Dog,
        Self::Robot,
        Self::Fox,
        Self::Alien,
        Self::Monster,
        Self::Ghost,
        Self::Slime,
        Self::Bird,
        Self::Wizard,
        Self::Skull,
        Self::Paws,
        Self::Planet,
        Self::Rocket,
        Self::Mushroom,
        Self::Cactus,
        Self::Frog,
        Self::Panda,
        Self::Cupcake,
        Self::Pizza,
        Self::Icecream,
        Self::Octopus,
        Self::Knight,
        Self::Bear,
        Self::Penguin,
        Self::Dragon,
        Self::Ninja,
        Self::Astronaut,
        Self::Diamond,
        Self::CoffeeCup,
        Self::Shield,
    ];

    /// Selects a family with the frozen catalog order.
    pub fn from_byte(value: u8) -> Self {
        Self::ALL
            .iter()
            .copied()
            .nth(usize::from(value) % Self::ALL.len())
            .unwrap_or_default()
    }

    /// Returns the frozen 1.x catalog identifier.
    pub const fn catalog_id(self) -> u16 {
        self as u16
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Robot => "robot",
            Self::Fox => "fox",
            Self::Alien => "alien",
            Self::Monster => "monster",
            Self::Ghost => "ghost",
            Self::Slime => "slime",
            Self::Bird => "bird",
            Self::Wizard => "wizard",
            Self::Skull => "skull",
            Self::Paws => "paws",
            Self::Planet => "planet",
            Self::Rocket => "rocket",
            Self::Mushroom => "mushroom",
            Self::Cactus => "cactus",
            Self::Frog => "frog",
            Self::Panda => "panda",
            Self::Cupcake => "cupcake",
            Self::Pizza => "pizza",
            Self::Icecream => "icecream",
            Self::Octopus => "octopus",
            Self::Knight => "knight",
            Self::Bear => "bear",
            Self::Penguin => "penguin",
            Self::Dragon => "dragon",
            Self::Ninja => "ninja",
            Self::Astronaut => "astronaut",
            Self::Diamond => "diamond",
            Self::CoffeeCup => "coffee-cup",
            Self::Shield => "shield",
        }
    }

    /// Returns this family's alpha.4 capability declaration.
    pub const fn capabilities(self) -> AvatarFamilyCapabilities {
        AvatarFamilyCapabilities {
            face_anchors: !matches!(
                self,
                Self::Paws
                    | Self::Planet
                    | Self::Rocket
                    | Self::Mushroom
                    | Self::Cactus
                    | Self::Cupcake
                    | Self::Pizza
                    | Self::Icecream
                    | Self::Diamond
                    | Self::CoffeeCup
                    | Self::Shield
            ),
        }
    }
}

/// Built-in canvas background.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarBackground {
    /// Identity-derived family-aware gradient.
    #[default]
    Themed,
    /// Opaque white.
    White,
    /// Opaque black.
    Black,
    /// Opaque charcoal.
    Dark,
    /// Opaque off-white.
    Light,
    /// Transparent canvas.
    Transparent,
    /// Dotted pattern.
    PolkaDot,
    /// Diagonal stripe pattern.
    Striped,
    /// Checkerboard pattern.
    Checkerboard,
    /// Grid pattern.
    Grid,
    /// Warm vertical gradient.
    Sunrise,
    /// Cool vertical gradient.
    Ocean,
    /// Dark deterministic star field.
    Starry,
}

impl AvatarBackground {
    /// Complete background catalog in frozen 1.x identifier order.
    pub const ALL: [Self; 13] = [
        Self::Themed,
        Self::White,
        Self::Black,
        Self::Dark,
        Self::Light,
        Self::Transparent,
        Self::PolkaDot,
        Self::Striped,
        Self::Checkerboard,
        Self::Grid,
        Self::Sunrise,
        Self::Ocean,
        Self::Starry,
    ];

    /// Selects a background with the frozen catalog order.
    pub fn from_byte(value: u8) -> Self {
        Self::ALL
            .iter()
            .copied()
            .nth(usize::from(value) % Self::ALL.len())
            .unwrap_or_default()
    }

    /// Returns the frozen 1.x catalog identifier.
    pub const fn catalog_id(self) -> u16 {
        self as u16
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Themed => "themed",
            Self::White => "white",
            Self::Black => "black",
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Transparent => "transparent",
            Self::PolkaDot => "polka-dot",
            Self::Striped => "striped",
            Self::Checkerboard => "checkerboard",
            Self::Grid => "grid",
            Self::Sunrise => "sunrise",
            Self::Ocean => "ocean",
            Self::Starry => "starry",
        }
    }
}

/// Built-in canvas frame shape.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarShape {
    /// Full square canvas.
    #[default]
    Square,
    /// Circular frame.
    Circle,
    /// Rounded square frame.
    Squircle,
    /// Hexagonal frame.
    Hexagon,
    /// Octagonal frame.
    Octagon,
}

impl AvatarShape {
    /// Complete frame catalog in frozen 1.x identifier order.
    pub const ALL: [Self; 5] = [
        Self::Square,
        Self::Circle,
        Self::Squircle,
        Self::Hexagon,
        Self::Octagon,
    ];

    /// Selects a frame shape with the frozen catalog order.
    pub fn from_byte(value: u8) -> Self {
        Self::ALL
            .iter()
            .copied()
            .nth(usize::from(value) % Self::ALL.len())
            .unwrap_or_default()
    }

    /// Returns the frozen 1.x catalog identifier.
    pub const fn catalog_id(self) -> u16 {
        self as u16
    }

    /// Returns the canonical ASCII label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Circle => "circle",
            Self::Squircle => "squircle",
            Self::Hexagon => "hexagon",
            Self::Octagon => "octagon",
        }
    }
}

/// Declared alpha.4 capabilities for one family.
#[must_use = "inspect family capabilities before requesting later style layers"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarFamilyCapabilities {
    face_anchors: bool,
}

impl AvatarFamilyCapabilities {
    /// Returns whether the family exposes face anchors for alpha.4 layers.
    pub const fn has_face_anchors(self) -> bool {
        self.face_anchors
    }

    /// Returns whether this family admits the requested accessory slot.
    pub const fn supports_accessory_slot(self, slot: crate::AvatarAccessorySlot) -> bool {
        self.face_anchors
            && matches!(
                slot,
                crate::AvatarAccessorySlot::Aura
                    | crate::AvatarAccessorySlot::Headwear
                    | crate::AvatarAccessorySlot::Earwear
                    | crate::AvatarAccessorySlot::Facewear
                    | crate::AvatarAccessorySlot::Eyewear
                    | crate::AvatarAccessorySlot::Neckwear
            )
    }

    /// Returns whether this family admits expression overlays.
    pub const fn supports_expressions(self) -> bool {
        self.face_anchors
    }

    /// Every family admits all built-in integer palettes.
    pub const fn supports_palettes(self) -> bool {
        true
    }

    /// Every built-in family supports every built-in background.
    pub const fn supports_backgrounds(self) -> bool {
        true
    }

    /// Every built-in family supports every built-in frame shape.
    pub const fn supports_shapes(self) -> bool {
        true
    }
}

/// One immutable family capability-manifest entry.
#[must_use = "capability entries describe built-in family behavior"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarFamilyCapabilityEntry {
    kind: AvatarKind,
    capabilities: AvatarFamilyCapabilities,
}

impl AvatarFamilyCapabilityEntry {
    const fn new(kind: AvatarKind) -> Self {
        Self {
            kind,
            capabilities: kind.capabilities(),
        }
    }

    /// Returns the family.
    pub const fn kind(self) -> AvatarKind {
        self.kind
    }

    /// Returns the declared capabilities.
    pub const fn capabilities(self) -> AvatarFamilyCapabilities {
        self.capabilities
    }
}

/// Complete immutable alpha.4 family capability manifest.
pub const AVATAR_FAMILY_CAPABILITIES: [AvatarFamilyCapabilityEntry; 31] = [
    AvatarFamilyCapabilityEntry::new(AvatarKind::Cat),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Dog),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Robot),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Fox),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Alien),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Monster),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Ghost),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Slime),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Bird),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Wizard),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Skull),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Paws),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Planet),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Rocket),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Mushroom),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Cactus),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Frog),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Panda),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Cupcake),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Pizza),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Icecream),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Octopus),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Knight),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Bear),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Penguin),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Dragon),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Ninja),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Astronaut),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Diamond),
    AvatarFamilyCapabilityEntry::new(AvatarKind::CoffeeCup),
    AvatarFamilyCapabilityEntry::new(AvatarKind::Shield),
];
