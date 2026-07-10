use super::*;

fn parse_ascii_label<T: Copy>(
    input: &str,
    labels: &[(&str, T)],
    error: &'static str,
) -> Result<T, &'static str> {
    let input = input.trim();
    labels
        .iter()
        .find_map(|(label, value)| input.eq_ignore_ascii_case(label).then_some(*value))
        .ok_or(error)
}

/// Trait for renderers that can draw reusable avatar styles onto an image buffer.
pub trait AvatarRenderer {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError>;
}

/// Export formats for encoded avatar assets.
///
/// `WebP` is the default because it is the more modern distribution format and
/// is usually smaller than PNG for generated avatar art.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarOutputFormat {
    /// Lossless WebP output.
    #[default]
    WebP,
    /// Optional lossless PNG output.
    #[cfg(feature = "png")]
    Png,
    /// Optional JPEG output with transparent pixels composited over white.
    #[cfg(feature = "jpeg")]
    Jpeg,
    /// Optional GIF output.
    ///
    /// # Warning
    ///
    /// GIF encoding performs 256-color quantization inside the `image` crate.
    /// Those internal quantization buffers are not accessible to `hashavatar`
    /// and are not sanitized by this crate. For high-assurance deployments,
    /// prefer `AvatarOutputFormat::WebP` or PNG output.
    #[cfg(feature = "gif")]
    Gif,
}

impl AvatarOutputFormat {
    pub const ALL: &'static [Self] = &[
        Self::WebP,
        #[cfg(feature = "png")]
        Self::Png,
        #[cfg(feature = "jpeg")]
        Self::Jpeg,
        #[cfg(feature = "gif")]
        Self::Gif,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            #[cfg(feature = "png")]
            Self::Png => "png",
            #[cfg(feature = "jpeg")]
            Self::Jpeg => "jpg",
            #[cfg(feature = "gif")]
            Self::Gif => "gif",
        }
    }
}

impl FromStr for AvatarOutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("webp", Self::WebP),
                #[cfg(feature = "png")]
                ("png", Self::Png),
                #[cfg(feature = "jpeg")]
                ("jpg", Self::Jpeg),
                #[cfg(feature = "jpeg")]
                ("jpeg", Self::Jpeg),
                #[cfg(feature = "gif")]
                ("gif", Self::Gif),
            ],
            "unsupported avatar output format",
        )
    }
}

impl std::fmt::Display for AvatarOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Avatar family selection.
///
/// Values can be round-tripped through [`AvatarKind::as_str`] and
/// [`FromStr`]. `Icecream` also accepts `ice-cream` and `ice_cream` when
/// parsing user input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarKind {
    /// Cat face avatar.
    #[default]
    Cat,
    /// Dog face avatar.
    Dog,
    /// Robot head avatar.
    Robot,
    /// Fox face avatar.
    Fox,
    /// Alien face avatar.
    Alien,
    /// Monster face avatar.
    Monster,
    /// Ghost avatar.
    Ghost,
    /// Slime creature avatar.
    Slime,
    /// Bird avatar.
    Bird,
    /// Wizard avatar.
    Wizard,
    /// Skull avatar.
    Skull,
    /// Paw-print avatar.
    Paws,
    /// Ringed planet avatar.
    Planet,
    /// Rocket avatar.
    Rocket,
    /// Mushroom avatar.
    Mushroom,
    /// Cactus avatar.
    Cactus,
    /// Frog face avatar.
    Frog,
    /// Panda face avatar.
    Panda,
    /// Cupcake avatar.
    Cupcake,
    /// Pizza slice avatar.
    Pizza,
    /// Ice cream cone avatar.
    Icecream,
    /// Octopus avatar.
    Octopus,
    /// Knight helmet avatar.
    Knight,
    /// Bear face avatar.
    Bear,
    /// Penguin avatar.
    Penguin,
    /// Dragon avatar.
    Dragon,
    /// Ninja avatar.
    Ninja,
    /// Astronaut avatar.
    Astronaut,
    /// Diamond avatar.
    Diamond,
    /// Coffee cup avatar.
    CoffeeCup,
    /// Shield avatar.
    Shield,
}

impl AvatarKind {
    pub const ALL: &'static [Self] = &[
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

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

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

    /// Returns whether this family has face anchors for accessories and
    /// expressions.
    ///
    /// Families without face anchors still support canvas-level color and
    /// frame-shape layers. Accessory and expression choices are accepted but
    /// skipped deterministically for those families.
    pub const fn supports_face_layers(self) -> bool {
        !matches!(
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
        )
    }
}

impl FromStr for AvatarKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("cat", Self::Cat),
                ("dog", Self::Dog),
                ("robot", Self::Robot),
                ("fox", Self::Fox),
                ("alien", Self::Alien),
                ("monster", Self::Monster),
                ("ghost", Self::Ghost),
                ("slime", Self::Slime),
                ("bird", Self::Bird),
                ("wizard", Self::Wizard),
                ("skull", Self::Skull),
                ("paws", Self::Paws),
                ("planet", Self::Planet),
                ("rocket", Self::Rocket),
                ("mushroom", Self::Mushroom),
                ("cactus", Self::Cactus),
                ("frog", Self::Frog),
                ("panda", Self::Panda),
                ("cupcake", Self::Cupcake),
                ("pizza", Self::Pizza),
                ("icecream", Self::Icecream),
                ("ice-cream", Self::Icecream),
                ("ice_cream", Self::Icecream),
                ("octopus", Self::Octopus),
                ("knight", Self::Knight),
                ("bear", Self::Bear),
                ("penguin", Self::Penguin),
                ("dragon", Self::Dragon),
                ("ninja", Self::Ninja),
                ("astronaut", Self::Astronaut),
                ("diamond", Self::Diamond),
                ("coffee-cup", Self::CoffeeCup),
                ("coffee_cup", Self::CoffeeCup),
                ("coffeecup", Self::CoffeeCup),
                ("shield", Self::Shield),
            ],
            "unsupported avatar kind",
        )
    }
}

impl std::fmt::Display for AvatarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Canvas background mode for raster and SVG avatar output.
///
/// `Themed` is identity and family aware. The fixed modes are useful for
/// predictable compositing, while `Transparent` leaves the SVG background out
/// and uses a fully transparent raster canvas.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarBackground {
    /// Identity-derived background chosen by the selected avatar family.
    #[default]
    Themed,
    /// Pure white background.
    White,
    /// Pure black background.
    Black,
    /// Charcoal background, useful for dark UI previews.
    Dark,
    /// Subtle off-white background.
    Light,
    /// Fully transparent background.
    Transparent,
    /// Light dotted pattern.
    PolkaDot,
    /// Subtle diagonal stripe pattern.
    Striped,
    /// Small checkerboard pattern.
    Checkerboard,
    /// Fine grid pattern.
    Grid,
    /// Warm sunrise gradient.
    Sunrise,
    /// Cool ocean gradient.
    Ocean,
    /// Dark star-field background.
    Starry,
}

impl AvatarBackground {
    pub const ALL: &'static [Self] = &[
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

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

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

impl FromStr for AvatarBackground {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("themed", Self::Themed),
                ("white", Self::White),
                ("black", Self::Black),
                ("dark", Self::Dark),
                ("light", Self::Light),
                ("transparent", Self::Transparent),
                ("polka-dot", Self::PolkaDot),
                ("polka_dot", Self::PolkaDot),
                ("polkadot", Self::PolkaDot),
                ("striped", Self::Striped),
                ("stripes", Self::Striped),
                ("checkerboard", Self::Checkerboard),
                ("checker-board", Self::Checkerboard),
                ("checker_board", Self::Checkerboard),
                ("grid", Self::Grid),
                ("sunrise", Self::Sunrise),
                ("ocean", Self::Ocean),
                ("starry", Self::Starry),
                ("stars", Self::Starry),
            ],
            "unsupported avatar background",
        )
    }
}

impl std::fmt::Display for AvatarBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Optional avatar accessory layer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarAccessory {
    /// No accessory layer.
    #[default]
    None,
    /// Simple glasses overlay.
    Glasses,
    /// Hat overlay.
    Hat,
    /// Headphones overlay.
    Headphones,
    /// Crown overlay.
    Crown,
    /// Bowtie overlay.
    Bowtie,
    /// Eyepatch overlay.
    Eyepatch,
    /// Scarf overlay.
    Scarf,
    /// Halo overlay.
    Halo,
    /// Horn overlay.
    Horns,
}

impl AvatarAccessory {
    pub const ALL: &'static [Self] = &[
        Self::None,
        Self::Glasses,
        Self::Hat,
        Self::Headphones,
        Self::Crown,
        Self::Bowtie,
        Self::Eyepatch,
        Self::Scarf,
        Self::Halo,
        Self::Horns,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Glasses => "glasses",
            Self::Hat => "hat",
            Self::Headphones => "headphones",
            Self::Crown => "crown",
            Self::Bowtie => "bowtie",
            Self::Eyepatch => "eyepatch",
            Self::Scarf => "scarf",
            Self::Halo => "halo",
            Self::Horns => "horns",
        }
    }
}

impl FromStr for AvatarAccessory {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("none", Self::None),
                ("glasses", Self::Glasses),
                ("hat", Self::Hat),
                ("headphones", Self::Headphones),
                ("crown", Self::Crown),
                ("bowtie", Self::Bowtie),
                ("bow-tie", Self::Bowtie),
                ("bow_tie", Self::Bowtie),
                ("eyepatch", Self::Eyepatch),
                ("eye-patch", Self::Eyepatch),
                ("eye_patch", Self::Eyepatch),
                ("scarf", Self::Scarf),
                ("halo", Self::Halo),
                ("horns", Self::Horns),
            ],
            "unsupported avatar accessory",
        )
    }
}

impl std::fmt::Display for AvatarAccessory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Optional avatar accent color palette.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarColor {
    /// Family default colors.
    #[default]
    Default,
    /// Bright mint accent.
    NeonMint,
    /// Soft pink accent.
    PastelPink,
    /// Deep red accent.
    Crimson,
    /// Warm gold accent.
    Gold,
    /// Blue-green accent.
    DeepSeaBlue,
}

impl AvatarColor {
    pub const ALL: &'static [Self] = &[
        Self::Default,
        Self::NeonMint,
        Self::PastelPink,
        Self::Crimson,
        Self::Gold,
        Self::DeepSeaBlue,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

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

impl FromStr for AvatarColor {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("default", Self::Default),
                ("neon-mint", Self::NeonMint),
                ("neon_mint", Self::NeonMint),
                ("neonmint", Self::NeonMint),
                ("pastel-pink", Self::PastelPink),
                ("pastel_pink", Self::PastelPink),
                ("pastelpink", Self::PastelPink),
                ("crimson", Self::Crimson),
                ("gold", Self::Gold),
                ("deep-sea-blue", Self::DeepSeaBlue),
                ("deep_sea_blue", Self::DeepSeaBlue),
                ("deepseablue", Self::DeepSeaBlue),
            ],
            "unsupported avatar color",
        )
    }
}

impl std::fmt::Display for AvatarColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Optional avatar expression layer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarExpression {
    /// Family default expression.
    #[default]
    Default,
    /// Happy expression overlay.
    Happy,
    /// Grumpy expression overlay.
    Grumpy,
    /// Surprised expression overlay.
    Surprised,
    /// Sleepy expression overlay.
    Sleepy,
    /// Winking expression overlay.
    Winking,
    /// Cool expression overlay.
    Cool,
    /// Crying expression overlay.
    Crying,
}

impl AvatarExpression {
    pub const ALL: &'static [Self] = &[
        Self::Default,
        Self::Happy,
        Self::Grumpy,
        Self::Surprised,
        Self::Sleepy,
        Self::Winking,
        Self::Cool,
        Self::Crying,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

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

impl FromStr for AvatarExpression {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("default", Self::Default),
                ("happy", Self::Happy),
                ("grumpy", Self::Grumpy),
                ("surprised", Self::Surprised),
                ("sleepy", Self::Sleepy),
                ("winking", Self::Winking),
                ("wink", Self::Winking),
                ("cool", Self::Cool),
                ("crying", Self::Crying),
            ],
            "unsupported avatar expression",
        )
    }
}

impl std::fmt::Display for AvatarExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Optional frame shape for the generated avatar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarShape {
    /// Default square canvas.
    #[default]
    Square,
    /// Circular frame.
    Circle,
    /// Rounded rectangle frame.
    Squircle,
    /// Hexagonal frame.
    Hexagon,
    /// Octagonal frame.
    Octagon,
}

impl AvatarShape {
    pub const ALL: &'static [Self] = &[
        Self::Square,
        Self::Circle,
        Self::Squircle,
        Self::Hexagon,
        Self::Octagon,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

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

impl FromStr for AvatarShape {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ascii_label(
            s,
            &[
                ("square", Self::Square),
                ("circle", Self::Circle),
                ("squircle", Self::Squircle),
                ("hexagon", Self::Hexagon),
                ("octagon", Self::Octagon),
            ],
            "unsupported avatar shape",
        )
    }
}

impl std::fmt::Display for AvatarShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
macro_rules! impl_serde_string_enum {
    ($ty:ty) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <&str as serde::Deserialize>::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarOutputFormat);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarKind);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarBackground);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarAccessory);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarColor);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarExpression);
#[cfg(feature = "serde")]
impl_serde_string_enum!(AvatarShape);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AvatarOptions {
    pub kind: AvatarKind,
    pub background: AvatarBackground,
}

impl AvatarOptions {
    pub const fn new(kind: AvatarKind, background: AvatarBackground) -> Self {
        Self { kind, background }
    }
}

/// Full avatar style selection including the baseline kind/background and
/// optional visual layers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AvatarStyleOptions {
    pub kind: AvatarKind,
    pub background: AvatarBackground,
    pub accessory: AvatarAccessory,
    pub color: AvatarColor,
    pub expression: AvatarExpression,
    pub shape: AvatarShape,
}

impl AvatarStyleOptions {
    pub const fn new(
        kind: AvatarKind,
        background: AvatarBackground,
        accessory: AvatarAccessory,
        color: AvatarColor,
        expression: AvatarExpression,
        shape: AvatarShape,
    ) -> Self {
        Self {
            kind,
            background,
            accessory,
            color,
            expression,
            shape,
        }
    }

    pub const fn from_options(options: AvatarOptions) -> Self {
        Self {
            kind: options.kind,
            background: options.background,
            accessory: AvatarAccessory::None,
            color: AvatarColor::Default,
            expression: AvatarExpression::Default,
            shape: AvatarShape::Square,
        }
    }

    pub fn from_identity(identity: &AvatarIdentity) -> Self {
        Self {
            kind: AvatarKind::from_byte(identity.byte(AVATAR_STYLE_KIND_BYTE)),
            background: AvatarBackground::from_byte(identity.byte(AVATAR_STYLE_BACKGROUND_BYTE)),
            accessory: AvatarAccessory::from_byte(identity.byte(AVATAR_STYLE_ACCESSORY_BYTE)),
            color: AvatarColor::from_byte(identity.byte(AVATAR_STYLE_COLOR_BYTE)),
            expression: AvatarExpression::from_byte(identity.byte(AVATAR_STYLE_EXPRESSION_BYTE)),
            shape: AvatarShape::from_byte(identity.byte(AVATAR_STYLE_SHAPE_BYTE)),
        }
    }

    pub const fn legacy_options(self) -> AvatarOptions {
        AvatarOptions::new(self.kind, self.background)
    }

    pub fn summary(self) -> String {
        self.to_string()
    }

    pub(crate) const fn has_extra_layers(self) -> bool {
        !matches!(self.accessory, AvatarAccessory::None)
            || !matches!(self.color, AvatarColor::Default)
            || !matches!(self.expression, AvatarExpression::Default)
            || !matches!(self.shape, AvatarShape::Square)
    }
}

impl From<AvatarOptions> for AvatarStyleOptions {
    fn from(options: AvatarOptions) -> Self {
        Self::from_options(options)
    }
}

impl From<AvatarStyleOptions> for AvatarOptions {
    fn from(options: AvatarStyleOptions) -> Self {
        options.legacy_options()
    }
}

impl std::fmt::Display for AvatarStyleOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} / {} / {} / {} / {} / {}",
            self.kind, self.background, self.accessory, self.color, self.expression, self.shape
        )
    }
}
