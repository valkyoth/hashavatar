use super::*;

mod keys;

pub use self::keys::*;

/// Frozen identifier for a built-in trait catalog.
///
/// The legacy catalog preserves the exact automatic-selection order shipped by
/// Hashavatar 1.x. Future catalogs must receive a new identifier instead of
/// changing this catalog in place.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CatalogVersion(&'static str);

impl CatalogVersion {
    /// The catalog used by all Hashavatar 1.x rendering APIs.
    pub const LEGACY_V1: Self = Self("hashavatar-catalog-v1");

    /// The default catalog for this crate release.
    pub const CURRENT: Self = Self::LEGACY_V1;

    pub const fn as_str(self) -> &'static str {
        self.0
    }

    /// Derives a style with the frozen catalog mapping.
    pub fn derive_style(self, identity: &AvatarIdentity) -> AvatarStyleOptions {
        let _ = self;
        AvatarStyleOptions::new(
            select_weighted(
                LEGACY_AVATAR_KINDS,
                identity.byte(AVATAR_STYLE_KIND_BYTE),
                AvatarKind::Cat,
            ),
            select_weighted(
                LEGACY_AVATAR_BACKGROUNDS,
                identity.byte(AVATAR_STYLE_BACKGROUND_BYTE),
                AvatarBackground::Themed,
            ),
            select_weighted(
                LEGACY_AVATAR_ACCESSORIES,
                identity.byte(AVATAR_STYLE_ACCESSORY_BYTE),
                AvatarAccessory::None,
            ),
            select_weighted(
                LEGACY_AVATAR_COLORS,
                identity.byte(AVATAR_STYLE_COLOR_BYTE),
                AvatarColor::Default,
            ),
            select_weighted(
                LEGACY_AVATAR_EXPRESSIONS,
                identity.byte(AVATAR_STYLE_EXPRESSION_BYTE),
                AvatarExpression::Default,
            ),
            select_weighted(
                LEGACY_AVATAR_SHAPES,
                identity.byte(AVATAR_STYLE_SHAPE_BYTE),
                AvatarShape::Square,
            ),
        )
    }
}

impl Default for CatalogVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl std::fmt::Display for CatalogVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Frozen identifier for the renderer behavior that determines avatar pixels.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderContractId(&'static str);

impl RenderContractId {
    /// Existing Hashavatar 1.x family geometry and compositing behavior.
    pub const LEGACY_V1: Self = Self("hashavatar-render-v1");

    /// The default render contract for this crate release.
    pub const CURRENT: Self = Self::LEGACY_V1;

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Default for RenderContractId {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl std::fmt::Display for RenderContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable catalog metadata for one built-in value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogEntry<T> {
    id: u16,
    weight: u16,
    value: T,
}

impl<T: Copy> CatalogEntry<T> {
    pub const fn new(id: u16, weight: u16, value: T) -> Self {
        Self { id, weight, value }
    }

    pub const fn id(self) -> u16 {
        self.id
    }

    pub const fn weight(self) -> u16 {
        self.weight
    }

    pub const fn value(self) -> T {
        self.value
    }
}

macro_rules! catalog {
    ($name:ident, $ty:ty, [$($id:literal => $value:path),+ $(,)?]) => {
        pub const $name: &[CatalogEntry<$ty>] = &[
            $(CatalogEntry::new($id, 1, $value),)+
        ];
    };
}

catalog!(LEGACY_AVATAR_KINDS, AvatarKind, [
    0 => AvatarKind::Cat, 1 => AvatarKind::Dog, 2 => AvatarKind::Robot,
    3 => AvatarKind::Fox, 4 => AvatarKind::Alien, 5 => AvatarKind::Monster,
    6 => AvatarKind::Ghost, 7 => AvatarKind::Slime, 8 => AvatarKind::Bird,
    9 => AvatarKind::Wizard, 10 => AvatarKind::Skull, 11 => AvatarKind::Paws,
    12 => AvatarKind::Planet, 13 => AvatarKind::Rocket, 14 => AvatarKind::Mushroom,
    15 => AvatarKind::Cactus, 16 => AvatarKind::Frog, 17 => AvatarKind::Panda,
    18 => AvatarKind::Cupcake, 19 => AvatarKind::Pizza, 20 => AvatarKind::Icecream,
    21 => AvatarKind::Octopus, 22 => AvatarKind::Knight, 23 => AvatarKind::Bear,
    24 => AvatarKind::Penguin, 25 => AvatarKind::Dragon, 26 => AvatarKind::Ninja,
    27 => AvatarKind::Astronaut, 28 => AvatarKind::Diamond,
    29 => AvatarKind::CoffeeCup, 30 => AvatarKind::Shield,
]);

catalog!(LEGACY_AVATAR_BACKGROUNDS, AvatarBackground, [
    0 => AvatarBackground::Themed, 1 => AvatarBackground::White,
    2 => AvatarBackground::Black, 3 => AvatarBackground::Dark,
    4 => AvatarBackground::Light, 5 => AvatarBackground::Transparent,
    6 => AvatarBackground::PolkaDot, 7 => AvatarBackground::Striped,
    8 => AvatarBackground::Checkerboard, 9 => AvatarBackground::Grid,
    10 => AvatarBackground::Sunrise, 11 => AvatarBackground::Ocean,
    12 => AvatarBackground::Starry,
]);

catalog!(LEGACY_AVATAR_ACCESSORIES, AvatarAccessory, [
    0 => AvatarAccessory::None, 1 => AvatarAccessory::Glasses,
    2 => AvatarAccessory::Hat, 3 => AvatarAccessory::Headphones,
    4 => AvatarAccessory::Crown, 5 => AvatarAccessory::Bowtie,
    6 => AvatarAccessory::Eyepatch, 7 => AvatarAccessory::Scarf,
    8 => AvatarAccessory::Halo, 9 => AvatarAccessory::Horns,
]);

catalog!(LEGACY_AVATAR_COLORS, AvatarColor, [
    0 => AvatarColor::Default, 1 => AvatarColor::NeonMint,
    2 => AvatarColor::PastelPink, 3 => AvatarColor::Crimson,
    4 => AvatarColor::Gold, 5 => AvatarColor::DeepSeaBlue,
]);

catalog!(LEGACY_AVATAR_EXPRESSIONS, AvatarExpression, [
    0 => AvatarExpression::Default, 1 => AvatarExpression::Happy,
    2 => AvatarExpression::Grumpy, 3 => AvatarExpression::Surprised,
    4 => AvatarExpression::Sleepy, 5 => AvatarExpression::Winking,
    6 => AvatarExpression::Cool, 7 => AvatarExpression::Crying,
]);

catalog!(LEGACY_AVATAR_SHAPES, AvatarShape, [
    0 => AvatarShape::Square, 1 => AvatarShape::Circle,
    2 => AvatarShape::Squircle, 3 => AvatarShape::Hexagon,
    4 => AvatarShape::Octagon,
]);

fn select_weighted<T: Copy>(catalog: &[CatalogEntry<T>], value: u8, fallback: T) -> T {
    let total_weight = catalog
        .iter()
        .fold(0_u16, |total, entry| total.saturating_add(entry.weight));
    debug_assert!(total_weight > 0, "built-in catalog must not be empty");
    let mut selected = u16::from(value) % total_weight.max(1);

    for entry in catalog {
        if selected < entry.weight {
            return entry.value;
        }
        selected -= entry.weight;
    }

    fallback
}

macro_rules! impl_stable_catalog_metadata {
    ($ty:ty, $catalog:ident) => {
        impl $ty {
            /// Returns this value's frozen ID in the legacy 1.x catalog.
            pub fn legacy_catalog_id(self) -> u16 {
                $catalog
                    .iter()
                    .find(|entry| entry.value == self)
                    .map(|entry| entry.id)
                    .unwrap_or(u16::MAX)
            }

            /// Returns this value's frozen automatic-selection weight.
            pub fn legacy_catalog_weight(self) -> u16 {
                $catalog
                    .iter()
                    .find(|entry| entry.value == self)
                    .map(|entry| entry.weight)
                    .unwrap_or(0)
            }
        }
    };
}

impl_stable_catalog_metadata!(AvatarKind, LEGACY_AVATAR_KINDS);
impl_stable_catalog_metadata!(AvatarBackground, LEGACY_AVATAR_BACKGROUNDS);
impl_stable_catalog_metadata!(AvatarAccessory, LEGACY_AVATAR_ACCESSORIES);
impl_stable_catalog_metadata!(AvatarColor, LEGACY_AVATAR_COLORS);
impl_stable_catalog_metadata!(AvatarExpression, LEGACY_AVATAR_EXPRESSIONS);
impl_stable_catalog_metadata!(AvatarShape, LEGACY_AVATAR_SHAPES);

/// Style layer that can be rejected by strict family validation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvatarStyleLayer {
    Accessory,
    Expression,
}

impl AvatarStyleLayer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accessory => "accessory",
            Self::Expression => "expression",
        }
    }
}

impl std::fmt::Display for AvatarStyleLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Declared style capabilities for one built-in avatar family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarFamilyCapabilities {
    face_layers: bool,
}

impl AvatarFamilyCapabilities {
    const fn new(face_layers: bool) -> Self {
        Self { face_layers }
    }

    pub const fn supports_accessories(self) -> bool {
        self.face_layers
    }

    pub const fn supports_expressions(self) -> bool {
        self.face_layers
    }

    pub const fn supports_backgrounds(self) -> bool {
        true
    }

    pub const fn supports_colors(self) -> bool {
        true
    }

    pub const fn supports_shapes(self) -> bool {
        true
    }

    pub const fn supports_accessory(self, accessory: AvatarAccessory) -> bool {
        matches!(accessory, AvatarAccessory::None) || self.supports_accessories()
    }

    pub const fn supports_expression(self, expression: AvatarExpression) -> bool {
        matches!(expression, AvatarExpression::Default) || self.supports_expressions()
    }
}

/// One entry in the frozen family capability manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarFamilyCapabilityEntry {
    kind: AvatarKind,
    capabilities: AvatarFamilyCapabilities,
}

impl AvatarFamilyCapabilityEntry {
    const fn new(kind: AvatarKind) -> Self {
        Self {
            kind,
            capabilities: AvatarFamilyCapabilities::new(kind.supports_face_layers()),
        }
    }

    pub const fn kind(self) -> AvatarKind {
        self.kind
    }

    pub const fn capabilities(self) -> AvatarFamilyCapabilities {
        self.capabilities
    }
}

pub const LEGACY_FAMILY_CAPABILITIES: &[AvatarFamilyCapabilityEntry] = &[
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

impl AvatarKind {
    pub const fn capabilities(self) -> AvatarFamilyCapabilities {
        AvatarFamilyCapabilities::new(self.supports_face_layers())
    }
}

/// Error returned when an explicit style requests a layer a family cannot use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarStyleValidationError {
    kind: AvatarKind,
    layer: AvatarStyleLayer,
    selection: &'static str,
}

impl AvatarStyleValidationError {
    pub const fn kind(self) -> AvatarKind {
        self.kind
    }

    pub const fn layer(self) -> AvatarStyleLayer {
        self.layer
    }

    pub const fn selection(self) -> &'static str {
        self.selection
    }
}

impl std::fmt::Display for AvatarStyleValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "avatar family {} does not support {} layer {}",
            self.kind, self.layer, self.selection
        )
    }
}

impl std::error::Error for AvatarStyleValidationError {}

impl AvatarStyleOptions {
    /// Returns the effective style used by the selected family.
    ///
    /// Legacy rendering skips accessories and expressions for families without
    /// face anchors. Canonicalization maps those ignored selections to their
    /// neutral values so identical pixels also receive identical asset keys.
    pub const fn canonicalized_for_family(mut self) -> Self {
        let capabilities = self.kind.capabilities();
        if !capabilities.supports_accessories() {
            self.accessory = AvatarAccessory::None;
        }
        if !capabilities.supports_expressions() {
            self.expression = AvatarExpression::Default;
        }
        self
    }

    /// Rejects explicit layers that the selected family would skip.
    ///
    /// Existing render APIs intentionally keep the 1.x compatibility behavior
    /// of skipping unsupported face layers. Call this method, or use
    /// [`AvatarBuilder::strict_style_validation`], when explicit user choices
    /// must fail closed instead.
    pub const fn validate_strict(self) -> Result<(), AvatarStyleValidationError> {
        let capabilities = self.kind.capabilities();
        if !capabilities.supports_accessory(self.accessory) {
            return Err(AvatarStyleValidationError {
                kind: self.kind,
                layer: AvatarStyleLayer::Accessory,
                selection: self.accessory.as_str(),
            });
        }
        if !capabilities.supports_expression(self.expression) {
            return Err(AvatarStyleValidationError {
                kind: self.kind,
                layer: AvatarStyleLayer::Expression,
                selection: self.expression.as_str(),
            });
        }
        Ok(())
    }
}
