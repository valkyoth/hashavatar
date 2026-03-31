//! Procedural, asset-free avatar generation driven by stable identity hashes.
//!
//! The crate produces deterministic avatar images from an input identifier
//! without shipping image packs, sprites, or third-party artwork. All visual
//! output is drawn from code using geometric primitives.
//!
//! Typical usage:
//! ```no_run
//! use hashavatar::{
//!     AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
//!     encode_avatar_for_id,
//! };
//!
//! let bytes = encode_avatar_for_id(
//!     AvatarSpec::new(256, 256, 0),
//!     "alice@example.com",
//!     AvatarOutputFormat::WebP,
//!     AvatarOptions {
//!         kind: AvatarKind::Robot,
//!         background: AvatarBackground::White,
//!     },
//! )?;
//! # Ok::<(), image::ImageError>(())
//! ```

use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;
use std::str::FromStr;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::{ExtendedColorType, ImageBuffer, ImageEncoder, ImageResult, Rgba, RgbaImage};
use imageproc::drawing::{
    draw_antialiased_line_segment_mut, draw_filled_circle_mut, draw_filled_ellipse_mut,
    draw_filled_rect_mut, draw_hollow_circle_mut, draw_line_segment_mut, draw_polygon_mut,
};
use imageproc::pixelops::interpolate;
use imageproc::point::Point;
use imageproc::rect::Rect;
use palette::{FromColor, Hsl, Srgb};
use rand::{Rng, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha512};

/// Rendering contract version for deterministic avatars.
///
/// Within a major crate release, the goal is to keep visuals stable for the
/// same `(namespace, id, kind, background, size)` tuple unless a documented bug
/// fix requires a targeted change.
pub const AVATAR_STYLE_VERSION: u32 = 2;

/// RGBA color helper for concise shape drawing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }
}

impl From<Color> for Rgba<u8> {
    fn from(value: Color) -> Self {
        Rgba(value.0)
    }
}

/// Input parameters for a generated avatar image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpec {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
}

impl AvatarSpec {
    pub const fn new(width: u32, height: u32, seed: u64) -> Self {
        Self {
            width,
            height,
            seed,
        }
    }
}

impl Default for AvatarSpec {
    fn default() -> Self {
        Self::new(256, 256, 1)
    }
}

/// A stable avatar identity derived from a SHA-512 digest.
///
/// This is intended for Robohash-style uniqueness: the same input always maps
/// to the same visual genome, while different inputs produce different shape
/// and palette parameters with negligible collision risk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvatarIdentity {
    digest: [u8; 64],
}

impl AvatarIdentity {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Self {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(namespace: AvatarNamespace<'_>, input: T) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(b"hashavatar");
        hasher.update([0]);
        hasher.update(namespace.tenant.as_bytes());
        hasher.update([0]);
        hasher.update(namespace.style_version.as_bytes());
        hasher.update([0]);
        hasher.update(input.as_ref());
        let digest: [u8; 64] = hasher.finalize().into();
        Self { digest }
    }

    pub const fn as_digest(&self) -> &[u8; 64] {
        &self.digest
    }

    pub fn seed(&self) -> u64 {
        let mut seed = [0u8; 8];
        seed.copy_from_slice(&self.digest[..8]);
        u64::from_le_bytes(seed)
    }

    fn byte(&self, index: usize) -> u8 {
        self.digest[index]
    }

    fn unit_f32(&self, index: usize) -> f32 {
        self.byte(index) as f32 / 255.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarNamespace<'a> {
    pub tenant: &'a str,
    pub style_version: &'a str,
}

impl<'a> AvatarNamespace<'a> {
    pub const fn new(tenant: &'a str, style_version: &'a str) -> Self {
        Self {
            tenant,
            style_version,
        }
    }
}

impl Default for AvatarNamespace<'_> {
    fn default() -> Self {
        Self::new("public", "v2")
    }
}

/// Trait for renderers that can draw reusable avatar styles onto an image buffer.
pub trait AvatarRenderer {
    fn render(&self, spec: AvatarSpec) -> RgbaImage;
}

/// Export formats for encoded avatar assets.
///
/// `WebP` is the default because it is the more modern distribution format and
/// is usually smaller than PNG for generated avatar art.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarOutputFormat {
    #[default]
    WebP,
    Png,
}

impl AvatarOutputFormat {
    pub const ALL: [Self; 2] = [Self::WebP, Self::Png];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            Self::Png => "png",
        }
    }
}

impl FromStr for AvatarOutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "webp" => Ok(Self::WebP),
            "png" => Ok(Self::Png),
            _ => Err("unsupported avatar output format"),
        }
    }
}

impl std::fmt::Display for AvatarOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarKind {
    #[default]
    Cat,
    Dog,
    Robot,
    Fox,
    Alien,
    Monster,
    Ghost,
    Slime,
    Bird,
    Wizard,
    Skull,
    Paws,
}

impl AvatarKind {
    pub const ALL: [Self; 12] = [
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
    ];

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
        }
    }
}

impl FromStr for AvatarKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cat" => Ok(Self::Cat),
            "dog" => Ok(Self::Dog),
            "robot" => Ok(Self::Robot),
            "fox" => Ok(Self::Fox),
            "alien" => Ok(Self::Alien),
            "monster" => Ok(Self::Monster),
            "ghost" => Ok(Self::Ghost),
            "slime" => Ok(Self::Slime),
            "bird" => Ok(Self::Bird),
            "wizard" => Ok(Self::Wizard),
            "skull" => Ok(Self::Skull),
            "paws" => Ok(Self::Paws),
            _ => Err("unsupported avatar kind"),
        }
    }
}

impl std::fmt::Display for AvatarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarBackground {
    #[default]
    Themed,
    White,
}

impl AvatarBackground {
    pub const ALL: [Self; 2] = [Self::Themed, Self::White];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Themed => "themed",
            Self::White => "white",
        }
    }
}

impl FromStr for AvatarBackground {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "themed" => Ok(Self::Themed),
            "white" => Ok(Self::White),
            _ => Err("unsupported avatar background"),
        }
    }
}

impl std::fmt::Display for AvatarBackground {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

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

/// Cat-face avatar renderer built from simple geometric primitives.
///
/// The face is intentionally stylized:
/// - a rounded head ellipse defines the main silhouette
/// - two ear polygons make the head read as feline rather than circular
/// - wide-set eyes, a small triangular nose, whiskers, and a curved smile complete the expression
#[derive(Clone, Copy, Debug, Default)]
pub struct CatAvatar;

impl AvatarRenderer for CatAvatar {
    fn render(&self, spec: AvatarSpec) -> RgbaImage {
        render_cat_avatar(spec)
    }
}

/// Cat-face avatar renderer driven by a SHA-512 identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedCatAvatar {
    identity: AvatarIdentity,
}

impl HashedCatAvatar {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Self {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(namespace: AvatarNamespace<'_>, input: T) -> Self {
        Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input),
        }
    }

    pub fn identity(&self) -> &AvatarIdentity {
        &self.identity
    }
}

impl AvatarRenderer for HashedCatAvatar {
    fn render(&self, spec: AvatarSpec) -> RgbaImage {
        render_cat_avatar_for_identity(spec, &self.identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedDogAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedDogAvatar {
    pub fn new<T: AsRef<[u8]>>(input: T, background: AvatarBackground) -> Self {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Self {
        Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input),
            background,
        }
    }
}

impl AvatarRenderer for HashedDogAvatar {
    fn render(&self, spec: AvatarSpec) -> RgbaImage {
        render_dog_avatar_for_identity(spec, &self.identity, self.background)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedRobotAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedRobotAvatar {
    pub fn new<T: AsRef<[u8]>>(input: T, background: AvatarBackground) -> Self {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Self {
        Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input),
            background,
        }
    }
}

impl AvatarRenderer for HashedRobotAvatar {
    fn render(&self, spec: AvatarSpec) -> RgbaImage {
        render_robot_avatar_for_identity(spec, &self.identity, self.background)
    }
}

/// Render and encode an avatar into memory.
pub fn encode_avatar<R: AvatarRenderer>(
    renderer: &R,
    spec: AvatarSpec,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    let image = renderer.render(spec);
    encode_rgba_image(&image, format)
}

/// Render and write an avatar to disk.
pub fn export_avatar<R: AvatarRenderer, P: AsRef<Path>>(
    renderer: &R,
    spec: AvatarSpec,
    format: AvatarOutputFormat,
    path: P,
) -> ImageResult<()> {
    let image = renderer.render(spec);
    write_rgba_image(&image, format, path)
}

/// Render and encode a cat avatar into memory.
pub fn encode_cat_avatar(spec: AvatarSpec, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    encode_avatar(&CatAvatar, spec, format)
}

/// Render and write a cat avatar to disk.
pub fn export_cat_avatar<P: AsRef<Path>>(
    spec: AvatarSpec,
    format: AvatarOutputFormat,
    path: P,
) -> ImageResult<()> {
    export_avatar(&CatAvatar, spec, format, path)
}

/// Render and encode a cat avatar for a stable identity string.
pub fn encode_cat_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    let renderer = HashedCatAvatar::new(id);
    encode_avatar(&renderer, spec, format)
}

/// Render and write a cat avatar for a stable identity string.
pub fn export_cat_avatar_for_id<T: AsRef<[u8]>, P: AsRef<Path>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
    path: P,
) -> ImageResult<()> {
    let renderer = HashedCatAvatar::new(id);
    export_avatar(&renderer, spec, format, path)
}

pub fn encode_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    encode_avatar_for_namespace(spec, AvatarNamespace::default(), id, format, options)
}

pub fn encode_avatar_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    let identity = AvatarIdentity::new_with_namespace(namespace, id);
    match options.kind {
        AvatarKind::Cat => {
            let image = render_cat_avatar_for_identity_with_background(
                spec,
                &identity,
                options.background,
            );
            encode_rgba_image(&image, format)
        }
        AvatarKind::Dog => encode_rgba_image(
            &render_dog_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Robot => encode_rgba_image(
            &render_robot_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Fox => encode_rgba_image(
            &render_fox_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Alien => encode_rgba_image(
            &render_alien_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Monster => encode_rgba_image(
            &render_monster_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Ghost => encode_rgba_image(
            &render_ghost_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Slime => encode_rgba_image(
            &render_slime_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Bird => encode_rgba_image(
            &render_bird_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Wizard => encode_rgba_image(
            &render_wizard_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Skull => encode_rgba_image(
            &render_skull_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
        AvatarKind::Paws => encode_rgba_image(
            &render_paws_avatar_for_identity(spec, &identity, options.background),
            format,
        ),
    }
}

/// Render an avatar image directly without encoding it.
pub fn render_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> RgbaImage {
    render_avatar_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> RgbaImage {
    let identity = AvatarIdentity::new_with_namespace(namespace, id);
    match options.kind {
        AvatarKind::Cat => {
            render_cat_avatar_for_identity_with_background(spec, &identity, options.background)
        }
        AvatarKind::Dog => render_dog_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Robot => render_robot_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Fox => render_fox_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Alien => render_alien_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Monster => {
            render_monster_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Ghost => render_ghost_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Slime => render_slime_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Bird => render_bird_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Wizard => {
            render_wizard_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Skull => render_skull_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Paws => render_paws_avatar_for_identity(spec, &identity, options.background),
    }
}

/// Render an avatar as a compact SVG string.
pub fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> String {
    render_avatar_svg_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_svg_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> String {
    let identity = AvatarIdentity::new_with_namespace(namespace, id);
    let bg = match options.background {
        AvatarBackground::Themed => match options.kind {
            AvatarKind::Cat => hsl_to_color(28.0 + identity.unit_f32(2) * 40.0, 0.25, 0.92),
            AvatarKind::Dog => hsl_to_color(200.0 + identity.unit_f32(3) * 60.0, 0.20, 0.92),
            AvatarKind::Robot => hsl_to_color(220.0 + identity.unit_f32(4) * 50.0, 0.18, 0.93),
            AvatarKind::Fox => hsl_to_color(18.0 + identity.unit_f32(5) * 30.0, 0.28, 0.93),
            AvatarKind::Alien => hsl_to_color(260.0 + identity.unit_f32(6) * 60.0, 0.20, 0.93),
            AvatarKind::Monster => hsl_to_color(300.0 + identity.unit_f32(7) * 45.0, 0.24, 0.92),
            AvatarKind::Ghost => hsl_to_color(220.0 + identity.unit_f32(8) * 35.0, 0.18, 0.95),
            AvatarKind::Slime => hsl_to_color(120.0 + identity.unit_f32(9) * 70.0, 0.24, 0.92),
            AvatarKind::Bird => hsl_to_color(180.0 + identity.unit_f32(10) * 40.0, 0.22, 0.93),
            AvatarKind::Wizard => hsl_to_color(250.0 + identity.unit_f32(11) * 40.0, 0.24, 0.92),
            AvatarKind::Skull => hsl_to_color(210.0 + identity.unit_f32(12) * 20.0, 0.08, 0.94),
            AvatarKind::Paws => hsl_to_color(28.0 + identity.unit_f32(13) * 30.0, 0.22, 0.94),
        },
        AvatarBackground::White => Color::rgb(255, 255, 255),
    };

    let body = match options.kind {
        AvatarKind::Cat => render_cat_svg(spec, &identity),
        AvatarKind::Dog => render_dog_svg(spec, &identity),
        AvatarKind::Robot => render_robot_svg(spec, &identity),
        AvatarKind::Fox => render_fox_svg(spec, &identity),
        AvatarKind::Alien => render_alien_svg(spec, &identity),
        AvatarKind::Monster => render_monster_svg(spec, &identity),
        AvatarKind::Ghost => render_ghost_svg(spec, &identity),
        AvatarKind::Slime => render_slime_svg(spec, &identity),
        AvatarKind::Bird => render_bird_svg(spec, &identity),
        AvatarKind::Wizard => render_wizard_svg(spec, &identity),
        AvatarKind::Skull => render_skull_svg(spec, &identity),
        AvatarKind::Paws => render_paws_svg(spec, &identity),
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label} avatar"><rect width="100%" height="100%" fill="{bg}"/>{body}</svg>"#,
        w = spec.width,
        h = spec.height,
        bg = color_hex(bg),
        label = options.kind.as_str(),
        body = body,
    )
    .replace('\n', "")
    .replace("  ", "")
}

pub fn export_avatar_svg_for_id<T: AsRef<[u8]>, P: AsRef<Path>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
    path: P,
) -> std::io::Result<()> {
    std::fs::write(path, render_avatar_svg_for_id(spec, id, options))
}

pub fn export_avatar_svg_for_namespace<T: AsRef<[u8]>, P: AsRef<Path>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
    path: P,
) -> std::io::Result<()> {
    std::fs::write(path, render_avatar_svg_for_namespace(spec, namespace, id, options))
}

/// Render a cat face avatar into an RGBA image.
pub fn render_cat_avatar(spec: AvatarSpec) -> RgbaImage {
    let identity = AvatarIdentity::new(spec.seed.to_le_bytes());
    render_cat_avatar_with_identity(spec, &identity, AvatarBackground::Themed)
}

/// Render a cat face avatar from a SHA-512-backed identity.
pub fn render_cat_avatar_for_identity(spec: AvatarSpec, identity: &AvatarIdentity) -> RgbaImage {
    render_cat_avatar_with_identity(spec, identity, AvatarBackground::Themed)
}

pub fn render_cat_avatar_for_identity_with_background(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    render_cat_avatar_with_identity(spec, identity, background)
}

fn render_cat_avatar_with_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let mut rng = StdRng::seed_from_u64(identity.seed() ^ spec.seed.rotate_left(13));
    let genome = CatGenome::from_identity(identity, &mut rng);
    let palette = CatPalette::from_genome(&genome);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, palette.background).into(),
    );

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = ((height as f32) * (0.53 + genome.head_drop * 0.08)) as i32;

    let head_rx = ((width as f32) * (0.26 + genome.head_width * 0.07)) as i32;
    let head_ry = ((height as f32) * (0.22 + genome.head_height * 0.08)) as i32;
    let ear_height = ((height as f32) * (0.15 + genome.ear_height * 0.08)) as i32;
    let ear_width = ((width as f32) * (0.12 + genome.ear_width * 0.08)) as i32;

    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.accent,
        genome.accent_band_height,
        background,
    );
    draw_ear(
        &mut image,
        EarSpec::left(
            center_x,
            center_y,
            head_rx,
            head_ry,
            ear_width,
            ear_height,
            genome.ear_tilt,
        ),
        palette.head,
        palette.ear_inner,
        palette.outline,
    );
    draw_ear(
        &mut image,
        EarSpec::right(
            center_x,
            center_y,
            head_rx,
            head_ry,
            ear_width,
            ear_height,
            genome.ear_tilt,
        ),
        palette.head,
        palette.ear_inner,
        palette.outline,
    );

    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        palette.head.into(),
    );
    draw_hollow_circle_mut(
        &mut image,
        (center_x, center_y),
        head_rx.min(head_ry),
        palette.outline.into(),
    );

    let muzzle_center = (center_x, center_y + head_ry / 4);
    draw_filled_ellipse_mut(
        &mut image,
        muzzle_center,
        (head_rx as f32 * (0.40 + genome.muzzle_width * 0.18)) as i32,
        (head_ry as f32 * (0.24 + genome.muzzle_height * 0.14)) as i32,
        palette.muzzle.into(),
    );

    draw_eyes(
        &mut image, center_x, center_y, head_rx, head_ry, palette, genome,
    );
    draw_nose_and_mouth(
        &mut image, center_x, center_y, head_rx, head_ry, palette, genome,
    );
    draw_whiskers(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.outline,
        genome,
    );
    draw_cat_markings(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.marking,
        genome,
    );

    image
}

pub fn render_dog_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(0) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(1) * 0.09)) as i32;
    let ear_drop = (head_ry as f32 * (0.65 + identity.unit_f32(2) * 0.35)) as i32;
    let muzzle_rx = (head_rx as f32 * (0.30 + identity.unit_f32(3) * 0.16)) as i32;
    let muzzle_ry = (head_ry as f32 * (0.22 + identity.unit_f32(4) * 0.10)) as i32;

    let fur = hsl_to_color(
        18.0 + identity.unit_f32(5) * 45.0,
        0.40,
        0.55 + identity.unit_f32(6) * 0.18,
    );
    let accent = hsl_to_color(190.0 + identity.unit_f32(7) * 70.0, 0.28, 0.88);
    let ear = hsl_to_color(
        22.0 + identity.unit_f32(8) * 30.0,
        0.38,
        0.38 + identity.unit_f32(9) * 0.12,
    );
    let muzzle = hsl_to_color(32.0 + identity.unit_f32(10) * 12.0, 0.18, 0.90);
    let nose = Color::rgb(45, 36, 34);
    let eye = Color::rgb(36, 26, 20);
    let tongue = hsl_to_color(350.0 + identity.unit_f32(11) * 10.0, 0.70, 0.70);
    let spot = hsl_to_color(
        24.0 + identity.unit_f32(12) * 20.0,
        0.36,
        0.34 + identity.unit_f32(13) * 0.10,
    );
    let bg_fill = background_fill(background, accent);
    image.pixels_mut().for_each(|pixel| *pixel = bg_fill.into());
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, accent, 0.45, background,
    );

    draw_filled_ellipse_mut(
        &mut image,
        (center_x - head_rx / 2, center_y - head_ry / 5),
        head_rx / 3,
        ear_drop,
        ear.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + head_rx / 2, center_y - head_ry / 5),
        head_rx / 3,
        ear_drop,
        ear.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        fur.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 3),
        muzzle_rx,
        muzzle_ry,
        muzzle.into(),
    );

    if identity.byte(14) % 2 == 0 {
        draw_filled_ellipse_mut(
            &mut image,
            (center_x - head_rx / 3, center_y - head_ry / 8),
            head_rx / 4,
            head_ry / 3,
            Color::rgba(spot.0[0], spot.0[1], spot.0[2], 150).into(),
        );
    }

    let eye_y = center_y - head_ry / 6;
    let eye_offset = (head_rx as f32 * (0.28 + identity.unit_f32(15) * 0.12)) as i32;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_circle_mut(
            &mut image,
            (x, eye_y),
            (head_rx as f32 * 0.08) as i32,
            Color::rgb(255, 255, 255).into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (x, eye_y),
            (head_rx as f32 * 0.04) as i32,
            eye.into(),
        );
    }

    let nose_y = center_y + head_ry / 5;
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, nose_y),
        head_rx / 7,
        head_ry / 10,
        nose.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, nose_y as f32),
        (center_x as f32, (nose_y + head_ry / 7) as f32),
        nose.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.55,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.55,
    );

    if identity.byte(16) % 3 != 0 {
        draw_filled_ellipse_mut(
            &mut image,
            (center_x, nose_y + head_ry / 4),
            head_rx / 10,
            head_ry / 7,
            tongue.into(),
        );
    }

    image
}

pub fn render_robot_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let width = spec.width as i32;
    let height = spec.height as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let bg = hsl_to_color(
        210.0 + identity.unit_f32(0) * 70.0,
        0.18 + identity.unit_f32(1) * 0.18,
        0.92,
    );
    let accent = hsl_to_color(160.0 + identity.unit_f32(2) * 120.0, 0.48, 0.62);
    let metal = hsl_to_color(200.0 + identity.unit_f32(3) * 28.0, 0.16, 0.74);
    let trim = hsl_to_color(205.0 + identity.unit_f32(4) * 22.0, 0.18, 0.46);
    let light = hsl_to_color(50.0 + identity.unit_f32(5) * 120.0, 0.84, 0.66);
    let dark = Color::rgb(47, 60, 72);
    let bg_fill = background_fill(background, bg);
    image.pixels_mut().for_each(|pixel| *pixel = bg_fill.into());

    let head_w = (width as f32 * (0.44 + identity.unit_f32(6) * 0.12)) as i32;
    let head_h = (height as f32 * (0.34 + identity.unit_f32(7) * 0.10)) as i32;
    let head_x = center_x - head_w / 2;
    let head_y = center_y - head_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_w / 2,
        head_h / 2,
        accent,
        0.5,
        background,
    );

    draw_filled_rect_mut(
        &mut image,
        Rect::at(head_x, head_y).of_size(head_w as u32, head_h as u32),
        metal.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, head_y as f32),
        ((head_x + head_w) as f32, head_y as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, (head_y + head_h) as f32),
        ((head_x + head_w) as f32, (head_y + head_h) as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, head_y as f32),
        (head_x as f32, (head_y + head_h) as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        ((head_x + head_w) as f32, head_y as f32),
        ((head_x + head_w) as f32, (head_y + head_h) as f32),
        trim.into(),
    );

    let antenna_h = (height as f32 * 0.10) as i32;
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, (head_y - antenna_h / 2) as f32),
        (center_x as f32, head_y as f32),
        dark.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (center_x, head_y - antenna_h / 2),
        (head_w as f32 * 0.05) as i32,
        accent.into(),
    );

    let eye_y = center_y - head_h / 6;
    let eye_offset = head_w / 4;
    let eye_rx = (head_w as f32 * 0.12) as i32;
    let eye_ry = (head_h as f32 * 0.10) as i32;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_ellipse_mut(&mut image, (x, eye_y), eye_rx, eye_ry, light.into());
        if identity.byte(8) % 2 == 0 {
            draw_filled_circle_mut(
                &mut image,
                (x, eye_y),
                (eye_rx as f32 * 0.35) as i32,
                dark.into(),
            );
        }
    }

    let mouth_y = center_y + head_h / 5;
    let mouth_w = (head_w as f32 * 0.42) as i32;
    let mouth_h = (head_h as f32 * 0.12) as i32;
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - mouth_w / 2, mouth_y - mouth_h / 2)
            .of_size(mouth_w as u32, mouth_h as u32),
        dark.into(),
    );
    let teeth = 4 + (identity.byte(9) % 4) as i32;
    for idx in 1..teeth {
        let x = center_x - mouth_w / 2 + idx * mouth_w / teeth;
        draw_line_segment_mut(
            &mut image,
            (x as f32, (mouth_y - mouth_h / 2) as f32),
            (x as f32, (mouth_y + mouth_h / 2) as f32),
            metal.into(),
        );
    }

    let bolt_y = center_y;
    draw_filled_circle_mut(
        &mut image,
        (head_x + head_w / 8, bolt_y),
        (head_w as f32 * 0.035) as i32,
        trim.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (head_x + head_w - head_w / 8, bolt_y),
        (head_w as f32 * 0.035) as i32,
        trim.into(),
    );
    image
}

pub fn render_fox_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let bg = hsl_to_color(22.0 + identity.unit_f32(0) * 26.0, 0.26, 0.92);
    let orange = hsl_to_color(18.0 + identity.unit_f32(1) * 20.0, 0.76, 0.58);
    let deep_orange = hsl_to_color(16.0 + identity.unit_f32(2) * 12.0, 0.72, 0.42);
    let cream = hsl_to_color(40.0 + identity.unit_f32(3) * 10.0, 0.32, 0.93);
    let eye = Color::rgb(34, 28, 24);
    let nose = Color::rgb(55, 40, 34);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, bg).into());

    let head_rx = (width as f32 * (0.25 + identity.unit_f32(4) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(5) * 0.08)) as i32;
    let ear_h = (height as f32 * (0.16 + identity.unit_f32(6) * 0.09)) as i32;
    let ear_w = (width as f32 * (0.12 + identity.unit_f32(7) * 0.05)) as i32;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        deep_orange,
        0.35,
        background,
    );
    draw_ear(
        &mut image,
        EarSpec::left(center_x, center_y, head_rx, head_ry, ear_w, ear_h, -0.2),
        orange,
        cream,
        deep_orange,
    );
    draw_ear(
        &mut image,
        EarSpec::right(center_x, center_y, head_rx, head_ry, ear_w, ear_h, 0.2),
        orange,
        cream,
        deep_orange,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        orange.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 4),
        head_rx / 2,
        head_ry / 3,
        cream.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - head_rx / 2, center_y - head_ry / 8),
            Point::new(center_x, center_y + head_ry / 3),
            Point::new(center_x - head_rx / 8, center_y + head_ry / 2),
        ],
        cream.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x + head_rx / 2, center_y - head_ry / 8),
            Point::new(center_x, center_y + head_ry / 3),
            Point::new(center_x + head_rx / 8, center_y + head_ry / 2),
        ],
        cream.into(),
    );

    let eye_y = center_y - head_ry / 7;
    let eye_offset = head_rx / 3;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_ellipse_mut(
            &mut image,
            (x, eye_y),
            head_rx / 10,
            head_ry / 8,
            Color::rgb(255, 255, 255).into(),
        );
        draw_filled_ellipse_mut(
            &mut image,
            (x, eye_y),
            head_rx / 18,
            head_ry / 7,
            eye.into(),
        );
    }
    let nose_y = center_y + head_ry / 4;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - head_rx / 10, nose_y),
            Point::new(center_x + head_rx / 10, nose_y),
            Point::new(center_x, nose_y + head_ry / 10),
        ],
        nose.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.45,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.45,
    );
    image
}

pub fn render_alien_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let skin = hsl_to_color(
        90.0 + identity.unit_f32(0) * 80.0,
        0.45 + identity.unit_f32(1) * 0.20,
        0.68,
    );
    let shade = hsl_to_color(110.0 + identity.unit_f32(2) * 50.0, 0.38, 0.44);
    let accent = hsl_to_color(280.0 + identity.unit_f32(3) * 40.0, 0.32, 0.92);
    let eye = Color::rgb(28, 18, 38);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, accent).into());
    let head_rx = (width as f32 * (0.20 + identity.unit_f32(4) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.28 + identity.unit_f32(5) * 0.10)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, shade, 0.28, background,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        skin.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x - head_rx / 2, center_y - head_ry / 4),
        head_rx / 5,
        head_ry / 3,
        eye.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + head_rx / 2, center_y - head_ry / 4),
        head_rx / 5,
        head_ry / 3,
        eye.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (center_x, center_y + head_ry / 8),
        head_rx / 14,
        shade.into(),
    );
    if identity.byte(6) % 2 == 0 {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x - head_rx / 8) as f32,
                (center_y + head_ry / 3) as f32,
            ),
            (
                (center_x + head_rx / 8) as f32,
                (center_y + head_ry / 3) as f32,
            ),
            shade.into(),
        );
    }
    image
}

pub fn render_monster_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());

    let skin = hsl_to_color(
        identity.unit_f32(0) * 360.0,
        0.48 + identity.unit_f32(1) * 0.24,
        0.46 + identity.unit_f32(2) * 0.20,
    );
    let shade = hsl_to_color(
        identity.unit_f32(3) * 360.0,
        0.38 + identity.unit_f32(4) * 0.18,
        0.24 + identity.unit_f32(5) * 0.10,
    );
    let accent = hsl_to_color(
        20.0 + identity.unit_f32(6) * 320.0,
        0.34 + identity.unit_f32(7) * 0.26,
        0.86,
    );
    let mouth = Color::rgb(48, 18, 24);
    let eye_white = Color::rgb(252, 248, 236);
    let pupil = Color::rgb(24, 20, 28);

    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, accent).into());

    let head_rx = (width as f32 * (0.23 + identity.unit_f32(8) * 0.10)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(9) * 0.11)) as i32;
    let horn_height = (height as f32 * (0.08 + identity.unit_f32(10) * 0.10)) as i32;
    let horn_width = (width as f32 * (0.06 + identity.unit_f32(11) * 0.05)) as i32;
    let eye_count = 1 + (identity.byte(12) % 3) as usize;
    let mouth_style = identity.byte(13) % 3;
    let body_style = identity.byte(14) % 3;
    let spot_count = 3 + (identity.byte(15) % 5) as i32;
    let tentacle_count = 2 + (identity.byte(16) % 4) as i32;

    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        shade,
        0.30 + identity.unit_f32(17) * 0.25,
        background,
    );

    match body_style {
        0 => draw_filled_ellipse_mut(&mut image, (center_x, center_y), head_rx, head_ry, skin.into()),
        1 => {
            draw_filled_ellipse_mut(
                &mut image,
                (center_x, center_y + head_ry / 8),
                head_rx,
                head_ry - head_ry / 8,
                skin.into(),
            );
            draw_polygon_mut(
                &mut image,
                &[
                    Point::new(center_x - head_rx, center_y),
                    Point::new(center_x, center_y - head_ry),
                    Point::new(center_x + head_rx, center_y),
                    Point::new(center_x + head_rx / 2, center_y + head_ry),
                    Point::new(center_x - head_rx / 2, center_y + head_ry),
                ],
                skin.into(),
            );
        }
        _ => {
            draw_filled_rect_mut(
                &mut image,
                Rect::at(center_x - head_rx, center_y - head_ry).of_size(
                    (head_rx * 2) as u32,
                    (head_ry * 2) as u32,
                ),
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x - head_rx, center_y - head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x + head_rx, center_y - head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x - head_rx, center_y + head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x + head_rx, center_y + head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
        }
    }

    if identity.byte(18).is_multiple_of(2) {
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x - head_rx / 2, center_y - head_ry),
                Point::new(center_x - head_rx / 3 - horn_width, center_y - head_ry - horn_height),
                Point::new(center_x - head_rx / 8, center_y - head_ry / 2),
            ],
            shade.into(),
        );
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x + head_rx / 2, center_y - head_ry),
                Point::new(center_x + head_rx / 3 + horn_width, center_y - head_ry - horn_height),
                Point::new(center_x + head_rx / 8, center_y - head_ry / 2),
            ],
            shade.into(),
        );
    } else {
        for spike in 0..3 {
            let spike_x = center_x - head_rx / 2 + spike * head_rx / 2;
            draw_polygon_mut(
                &mut image,
                &[
                    Point::new(spike_x - horn_width / 2, center_y - head_ry / 2),
                    Point::new(spike_x, center_y - head_ry - horn_height / 2),
                    Point::new(spike_x + horn_width / 2, center_y - head_ry / 2),
                ],
                shade.into(),
            );
        }
    }

    for index in 0..spot_count {
        let x = center_x - head_rx / 2 + (index * head_rx) / spot_count;
        let y = center_y - head_ry / 3 + ((index * 37 + identity.byte(19) as i32) % head_ry.max(1));
        let radius = (head_rx as f32 * (0.05 + ((index + 1) as f32 / spot_count as f32) * 0.06)) as i32;
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            radius.max(3),
            Color::rgba(shade.0[0], shade.0[1], shade.0[2], 168).into(),
        );
    }

    let eye_y = center_y - head_ry / 5;
    let eye_rx = (head_rx as f32 * (0.10 + identity.unit_f32(20) * 0.08)) as i32;
    let eye_ry = (head_ry as f32 * (0.10 + identity.unit_f32(21) * 0.10)) as i32;
    let eye_spacing = if eye_count == 1 {
        0
    } else {
        (head_rx as f32 * 0.46 / (eye_count - 1) as f32) as i32
    };
    let eye_start = center_x - eye_spacing * ((eye_count.saturating_sub(1)) as i32) / 2;
    for index in 0..eye_count {
        let x = eye_start + eye_spacing * index as i32;
        draw_filled_ellipse_mut(&mut image, (x, eye_y), eye_rx, eye_ry, eye_white.into());
        if identity.byte(22).is_multiple_of(2) {
            draw_filled_ellipse_mut(
                &mut image,
                (x, eye_y),
                (eye_rx / 3).max(2),
                (eye_ry - 1).max(2),
                pupil.into(),
            );
        } else {
            draw_filled_circle_mut(&mut image, (x, eye_y), (eye_ry / 2).max(2), pupil.into());
        }
        draw_filled_circle_mut(
            &mut image,
            (x - eye_rx / 3, eye_y - eye_ry / 3),
            (eye_rx / 5).max(1),
            Color::rgba(255, 255, 255, 220).into(),
        );
    }

    let mouth_y = center_y + head_ry / 3;
    match mouth_style {
        0 => {
            draw_filled_ellipse_mut(
                &mut image,
                (center_x, mouth_y),
                head_rx / 3,
                head_ry / 8,
                mouth.into(),
            );
            for fang_x in [center_x - head_rx / 8, center_x + head_rx / 8] {
                draw_polygon_mut(
                    &mut image,
                    &[
                        Point::new(fang_x - head_rx / 24, mouth_y - 2),
                        Point::new(fang_x + head_rx / 24, mouth_y - 2),
                        Point::new(fang_x, mouth_y + head_ry / 5),
                    ],
                    eye_white.into(),
                );
            }
        }
        1 => {
            draw_smile_arc(
                &mut image,
                center_x - head_rx / 10,
                mouth_y,
                head_rx / 4,
                mouth,
                0.50,
            );
            draw_smile_arc(
                &mut image,
                center_x + head_rx / 10,
                mouth_y,
                head_rx / 4,
                mouth,
                0.50,
            );
            draw_line_segment_mut(
                &mut image,
                ((center_x - head_rx / 4) as f32, mouth_y as f32),
                ((center_x + head_rx / 4) as f32, mouth_y as f32),
                mouth.into(),
            );
        }
        _ => {
            draw_filled_rect_mut(
                &mut image,
                Rect::at(center_x - head_rx / 3, mouth_y - head_ry / 10).of_size(
                    (head_rx * 2 / 3) as u32,
                    (head_ry / 5).max(1) as u32,
                ),
                mouth.into(),
            );
            for tooth in 0..4 {
                let tooth_x = center_x - head_rx / 4 + tooth * head_rx / 6;
                draw_polygon_mut(
                    &mut image,
                    &[
                        Point::new(tooth_x - head_rx / 30, mouth_y - head_ry / 10),
                        Point::new(tooth_x + head_rx / 30, mouth_y - head_ry / 10),
                        Point::new(tooth_x, mouth_y + head_ry / 14),
                    ],
                    eye_white.into(),
                );
            }
        }
    }

    if identity.byte(23).is_multiple_of(2) {
        for index in 0..tentacle_count {
            let start_x = center_x - head_rx / 2 + (index * head_rx) / tentacle_count;
            let start_y = center_y + head_ry - 4;
            let end_x = start_x + ((index % 2) * 2 - 1) * head_rx / 6;
            let end_y = start_y + head_ry / 2;
            draw_antialiased_line_segment_mut(
                &mut image,
                (start_x, start_y),
                (end_x, end_y),
                shade.into(),
                interpolate,
            );
            draw_filled_circle_mut(&mut image, (end_x, end_y), (head_rx / 18).max(2), shade.into());
        }
    }

    image
}

#[derive(Clone, Copy)]
struct FaceLayout {
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
}

#[derive(Clone, Copy)]
enum CreatureEyeStyle {
    Round,
    Tall,
    Hollow,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum CreatureMouthStyle {
    Smile,
    Fang,
    Flat,
}

fn draw_creature_eyes(
    image: &mut RgbaImage,
    layout: FaceLayout,
    count: usize,
    style: CreatureEyeStyle,
    eye_white: Color,
    pupil: Color,
) {
    let spacing = if count <= 1 {
        0
    } else {
        (layout.head_rx as f32 * 0.48 / (count - 1) as f32) as i32
    };
    let start_x = layout.center_x - spacing * (count.saturating_sub(1) as i32) / 2;
    let eye_y = layout.center_y - layout.head_ry / 5;
    let eye_rx = (layout.head_rx as f32 * 0.12) as i32;
    let eye_ry = (layout.head_ry as f32 * 0.12) as i32;

    for index in 0..count {
        let x = start_x + spacing * index as i32;
        match style {
            CreatureEyeStyle::Round => {
                draw_filled_circle_mut(image, (x, eye_y), eye_rx.max(3), eye_white.into());
                draw_filled_circle_mut(image, (x, eye_y), (eye_rx / 2).max(2), pupil.into());
            }
            CreatureEyeStyle::Tall => {
                draw_filled_ellipse_mut(image, (x, eye_y), eye_rx, eye_ry + 4, eye_white.into());
                draw_filled_ellipse_mut(
                    image,
                    (x, eye_y),
                    (eye_rx / 3).max(2),
                    (eye_ry + 2).max(2),
                    pupil.into(),
                );
            }
            CreatureEyeStyle::Hollow => {
                draw_filled_ellipse_mut(image, (x, eye_y), eye_rx + 3, eye_ry + 5, pupil.into());
                draw_filled_ellipse_mut(
                    image,
                    (x, eye_y + 1),
                    (eye_rx / 2).max(2),
                    (eye_ry / 2).max(2),
                    Color::rgba(255, 255, 255, 20).into(),
                );
            }
        }
    }
}

fn draw_creature_mouth(
    image: &mut RgbaImage,
    layout: FaceLayout,
    style: CreatureMouthStyle,
    color: Color,
) {
    let mouth_y = layout.center_y + layout.head_ry / 3;
    match style {
        CreatureMouthStyle::Smile => {
            draw_smile_arc(
                image,
                layout.center_x - layout.head_rx / 10,
                mouth_y,
                layout.head_rx / 4,
                color,
                0.45,
            );
            draw_smile_arc(
                image,
                layout.center_x + layout.head_rx / 10,
                mouth_y,
                layout.head_rx / 4,
                color,
                0.45,
            );
        }
        CreatureMouthStyle::Fang => {
            draw_filled_ellipse_mut(
                image,
                (layout.center_x, mouth_y),
                layout.head_rx / 3,
                layout.head_ry / 8,
                color.into(),
            );
            for tooth_x in [layout.center_x - layout.head_rx / 7, layout.center_x + layout.head_rx / 7]
            {
                draw_polygon_mut(
                    image,
                    &[
                        Point::new(tooth_x - layout.head_rx / 26, mouth_y - 1),
                        Point::new(tooth_x + layout.head_rx / 26, mouth_y - 1),
                        Point::new(tooth_x, mouth_y + layout.head_ry / 5),
                    ],
                    Color::rgb(248, 246, 238).into(),
                );
            }
        }
        CreatureMouthStyle::Flat => {
            draw_filled_rect_mut(
                image,
                Rect::at(layout.center_x - layout.head_rx / 3, mouth_y - 3)
                    .of_size((layout.head_rx * 2 / 3) as u32, 6),
                color.into(),
            );
        }
    }
}

pub fn render_ghost_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.54) as i32,
        head_rx: (width as f32 * 0.22) as i32,
        head_ry: (height as f32 * 0.24) as i32,
    };
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, background_fill(background, hsl_to_color(220.0 + identity.unit_f32(0) * 30.0, 0.18, 0.95)).into());
    let body = hsl_to_color(200.0 + identity.unit_f32(1) * 30.0, 0.14, 0.98);
    let shade = hsl_to_color(215.0 + identity.unit_f32(2) * 18.0, 0.18, 0.78);
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        shade,
        0.28,
        background,
    );
    draw_filled_ellipse_mut(&mut image, (layout.center_x, layout.center_y), layout.head_rx, layout.head_ry, body.into());
    draw_filled_rect_mut(
        &mut image,
        Rect::at(layout.center_x - layout.head_rx, layout.center_y)
            .of_size((layout.head_rx * 2) as u32, (layout.head_ry + layout.head_ry / 2) as u32),
        body.into(),
    );
    for index in 0..4 {
        let x = layout.center_x - layout.head_rx + index * (layout.head_rx * 2 / 3);
        let radius = layout.head_rx / 4 + (index % 2) * 4;
        draw_filled_circle_mut(
            &mut image,
            (x, layout.center_y + layout.head_ry + layout.head_ry / 2),
            radius,
            body.into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        CreatureEyeStyle::Tall,
        Color::rgb(42, 48, 68),
        Color::rgb(42, 48, 68),
    );
    draw_creature_mouth(&mut image, layout, CreatureMouthStyle::Smile, shade);
    image
}

pub fn render_slime_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.58) as i32,
        head_rx: (width as f32 * 0.24) as i32,
        head_ry: (height as f32 * 0.18) as i32,
    };
    let bg = hsl_to_color(110.0 + identity.unit_f32(3) * 80.0, 0.18, 0.93);
    let slime = hsl_to_color(95.0 + identity.unit_f32(4) * 70.0, 0.52, 0.56);
    let dark = hsl_to_color(110.0 + identity.unit_f32(5) * 40.0, 0.42, 0.32);
    let mut image = ImageBuffer::from_pixel(spec.width, spec.height, background_fill(background, bg).into());
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        dark,
        0.32,
        background,
    );
    draw_filled_ellipse_mut(&mut image, (layout.center_x, layout.center_y), layout.head_rx, layout.head_ry, slime.into());
    for index in 0..3 {
        let drip_x = layout.center_x - layout.head_rx / 2 + index * layout.head_rx / 2;
        let drip_h = layout.head_ry / 2 + ((identity.byte(10 + index as usize) % 20) as i32);
        draw_filled_rect_mut(
            &mut image,
            Rect::at(drip_x, layout.center_y).of_size((layout.head_rx / 3) as u32, drip_h as u32),
            slime.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (drip_x + layout.head_rx / 6, layout.center_y + drip_h),
            layout.head_rx / 7,
            slime.into(),
        );
    }
    for bubble in 0..4 {
        draw_filled_circle_mut(
            &mut image,
            (
                layout.center_x - layout.head_rx / 2 + bubble * layout.head_rx / 3,
                layout.center_y - layout.head_ry / 3 + bubble * 9,
            ),
            (layout.head_rx / 10 + bubble).max(3),
            Color::rgba(255, 255, 255, 90).into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        if identity.byte(20).is_multiple_of(2) { 2 } else { 3 },
        CreatureEyeStyle::Round,
        Color::rgb(248, 255, 236),
        Color::rgb(32, 48, 24),
    );
    draw_creature_mouth(&mut image, layout, CreatureMouthStyle::Flat, dark);
    image
}

pub fn render_bird_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.56) as i32,
        head_rx: (width as f32 * 0.22) as i32,
        head_ry: (height as f32 * 0.22) as i32,
    };
    let bg = hsl_to_color(190.0 + identity.unit_f32(6) * 60.0, 0.18, 0.93);
    let plumage = hsl_to_color(identity.unit_f32(7) * 360.0, 0.42, 0.62);
    let wing = hsl_to_color(20.0 + identity.unit_f32(8) * 160.0, 0.32, 0.46);
    let beak = hsl_to_color(32.0 + identity.unit_f32(9) * 26.0, 0.82, 0.58);
    let mut image = ImageBuffer::from_pixel(spec.width, spec.height, background_fill(background, bg).into());
    draw_background_accent(&mut image, layout.center_x, layout.center_y, layout.head_rx, layout.head_ry, wing, 0.24, background);
    draw_filled_circle_mut(&mut image, (layout.center_x, layout.center_y), layout.head_rx, plumage.into());
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x - layout.head_rx / 2, layout.center_y + layout.head_ry / 6),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x + layout.head_rx / 2, layout.center_y + layout.head_ry / 6),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(layout.center_x + layout.head_rx / 2, layout.center_y + layout.head_ry / 6),
            Point::new(layout.center_x, layout.center_y + layout.head_ry / 3),
        ],
        beak.into(),
    );
    for feather in 0..3 {
        let fx = layout.center_x - layout.head_rx / 5 + feather * layout.head_rx / 5;
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(fx, layout.center_y - layout.head_ry),
                Point::new(fx + layout.head_rx / 10, layout.center_y - layout.head_ry - layout.head_ry / 2),
                Point::new(fx + layout.head_rx / 5, layout.center_y - layout.head_ry / 2),
            ],
            wing.into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        CreatureEyeStyle::Round,
        Color::rgb(255, 255, 255),
        Color::rgb(28, 24, 34),
    );
    image
}

pub fn render_wizard_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.60) as i32,
        head_rx: (width as f32 * 0.18) as i32,
        head_ry: (height as f32 * 0.18) as i32,
    };
    let bg = hsl_to_color(250.0 + identity.unit_f32(10) * 40.0, 0.24, 0.92);
    let hat = hsl_to_color(230.0 + identity.unit_f32(11) * 50.0, 0.42, 0.38);
    let hat_band = hsl_to_color(28.0 + identity.unit_f32(12) * 30.0, 0.74, 0.58);
    let skin = hsl_to_color(22.0 + identity.unit_f32(13) * 18.0, 0.30, 0.82);
    let beard = hsl_to_color(40.0 + identity.unit_f32(14) * 25.0, 0.10, 0.92);
    let mut image = ImageBuffer::from_pixel(spec.width, spec.height, background_fill(background, bg).into());
    draw_background_accent(&mut image, layout.center_x, layout.center_y, layout.head_rx, layout.head_ry, hat_band, 0.20, background);
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x - layout.head_rx, layout.center_y - layout.head_ry / 2),
            Point::new(layout.center_x + layout.head_rx, layout.center_y - layout.head_ry / 2),
            Point::new(layout.center_x, layout.center_y - layout.head_ry * 2),
        ],
        hat.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(layout.center_x - layout.head_rx - layout.head_rx / 2, layout.center_y - layout.head_ry / 2)
            .of_size((layout.head_rx * 3) as u32, (layout.head_ry / 3) as u32),
        hat_band.into(),
    );
    draw_filled_circle_mut(&mut image, (layout.center_x, layout.center_y), layout.head_rx, skin.into());
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x - layout.head_rx / 2, layout.center_y + layout.head_ry / 3),
            Point::new(layout.center_x + layout.head_rx / 2, layout.center_y + layout.head_ry / 3),
            Point::new(layout.center_x, layout.center_y + layout.head_ry + layout.head_ry / 2),
        ],
        beard.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        CreatureEyeStyle::Round,
        Color::rgb(255, 255, 255),
        Color::rgb(36, 30, 52),
    );
    draw_creature_mouth(&mut image, layout, CreatureMouthStyle::Smile, Color::rgb(86, 64, 58));
    draw_filled_circle_mut(
        &mut image,
        (layout.center_x + layout.head_rx / 2, layout.center_y - layout.head_ry - layout.head_ry / 2),
        (layout.head_rx / 6).max(3),
        hat_band.into(),
    );
    image
}

pub fn render_skull_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.54) as i32,
        head_rx: (width as f32 * 0.20) as i32,
        head_ry: (height as f32 * 0.20) as i32,
    };
    let bg = hsl_to_color(210.0 + identity.unit_f32(15) * 20.0, 0.08, 0.94);
    let bone = hsl_to_color(38.0 + identity.unit_f32(16) * 14.0, 0.10, 0.90);
    let crack = Color::rgb(72, 68, 62);
    let mut image = ImageBuffer::from_pixel(spec.width, spec.height, background_fill(background, bg).into());
    draw_background_accent(&mut image, layout.center_x, layout.center_y, layout.head_rx, layout.head_ry, crack, 0.16, background);
    draw_filled_ellipse_mut(&mut image, (layout.center_x, layout.center_y), layout.head_rx, layout.head_ry, bone.into());
    draw_filled_rect_mut(
        &mut image,
        Rect::at(layout.center_x - layout.head_rx / 2, layout.center_y + layout.head_ry / 2)
            .of_size(layout.head_rx as u32, (layout.head_ry / 2) as u32),
        bone.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        CreatureEyeStyle::Hollow,
        Color::rgb(44, 42, 44),
        Color::rgb(44, 42, 44),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(layout.center_x - layout.head_rx / 8, layout.center_y + layout.head_ry / 5),
            Point::new(layout.center_x + layout.head_rx / 8, layout.center_y + layout.head_ry / 5),
        ],
        crack.into(),
    );
    draw_creature_mouth(&mut image, layout, CreatureMouthStyle::Flat, crack);
    for tooth in 0..4 {
        let x = layout.center_x - layout.head_rx / 4 + tooth * layout.head_rx / 6;
        draw_line_segment_mut(
            &mut image,
            (x as f32, (layout.center_y + layout.head_ry / 2) as f32),
            (x as f32, (layout.center_y + layout.head_ry) as f32),
            crack.into(),
        );
    }
    draw_line_segment_mut(
        &mut image,
        ((layout.center_x + layout.head_rx / 4) as f32, (layout.center_y - layout.head_ry / 2) as f32),
        ((layout.center_x + layout.head_rx / 8) as f32, (layout.center_y - layout.head_ry / 8) as f32),
        crack.into(),
    );
    image
}

pub fn render_paws_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    assert!(spec.width >= 64, "width must be at least 64 pixels");
    assert!(spec.height >= 64, "height must be at least 64 pixels");

    let width = spec.width as i32;
    let height = spec.height as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let bg = hsl_to_color(24.0 + identity.unit_f32(0) * 36.0, 0.20, 0.94);
    let fur = hsl_to_color(identity.unit_f32(1) * 360.0, 0.32 + identity.unit_f32(2) * 0.18, 0.60);
    let pad = hsl_to_color(330.0 + identity.unit_f32(3) * 20.0, 0.36 + identity.unit_f32(4) * 0.18, 0.72);
    let accent = hsl_to_color(18.0 + identity.unit_f32(5) * 24.0, 0.34, 0.82);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, bg).into());

    if background == AvatarBackground::Themed {
        for stripe in 0..4 {
            let y = (height / 8) + stripe * (height / 5);
            draw_filled_rect_mut(
                &mut image,
                Rect::at(0, y).of_size(spec.width, (height / 18).max(1) as u32),
                Color::rgba(accent.0[0], accent.0[1], accent.0[2], 70).into(),
            );
        }
    }

    let primary_x = width / 2;
    let primary_y = height / 2 + height / 12;
    let palm_rx = (width as f32 * (0.14 + identity.unit_f32(6) * 0.04)) as i32;
    let palm_ry = (height as f32 * (0.16 + identity.unit_f32(7) * 0.04)) as i32;
    draw_paw_print(
        &mut image,
        primary_x,
        primary_y,
        palm_rx,
        palm_ry,
        fur,
        pad,
        identity.byte(8),
    );

    if identity.byte(9).is_multiple_of(2) {
        draw_paw_print(
            &mut image,
            width / 3,
            height / 3,
            (palm_rx as f32 * 0.82) as i32,
            (palm_ry as f32 * 0.82) as i32,
            hsl_to_color(identity.unit_f32(10) * 360.0, 0.28, 0.66),
            pad,
            identity.byte(11),
        );
    }

    if identity.byte(12) % 3 != 0 {
        draw_paw_print(
            &mut image,
            width * 2 / 3,
            height / 3 + height / 8,
            (palm_rx as f32 * 0.70) as i32,
            (palm_ry as f32 * 0.70) as i32,
            fur,
            hsl_to_color(340.0 + identity.unit_f32(13) * 12.0, 0.30, 0.80),
            identity.byte(14),
        );
    }

    image
}

fn draw_paw_print(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    palm_rx: i32,
    palm_ry: i32,
    fur: Color,
    pad: Color,
    shape_seed: u8,
) {
    let toe_offset_y = palm_ry;
    let toe_spacing = (palm_rx as f32 * (0.48 + (shape_seed as f32 / 255.0) * 0.12)) as i32;
    let toe_rx = (palm_rx as f32 * (0.26 + (shape_seed as f32 / 255.0) * 0.04)) as i32;
    let toe_ry = (palm_ry as f32 * (0.24 + ((shape_seed >> 2) as f32 / 255.0) * 0.06)) as i32;

    draw_filled_ellipse_mut(image, (center_x, center_y), palm_rx, palm_ry, fur.into());
    draw_filled_ellipse_mut(
        image,
        (center_x, center_y + palm_ry / 8),
        (palm_rx as f32 * 0.72) as i32,
        (palm_ry as f32 * 0.68) as i32,
        pad.into(),
    );

    for (index, offset) in [-3, -1, 1, 3].into_iter().enumerate() {
        let x = center_x + offset * toe_spacing / 4;
        let y = center_y - toe_offset_y + if index % 2 == 0 { 0 } else { toe_ry / 3 };
        draw_filled_ellipse_mut(image, (x, y), toe_rx, toe_ry, fur.into());
        draw_filled_ellipse_mut(
            image,
            (x, y + toe_ry / 5),
            (toe_rx as f32 * 0.68) as i32,
            (toe_ry as f32 * 0.68) as i32,
            pad.into(),
        );
    }
}

#[derive(Clone, Copy, Debug)]
struct CatPalette {
    background: Color,
    accent: Color,
    head: Color,
    ear_inner: Color,
    muzzle: Color,
    eye: Color,
    pupil: Color,
    nose: Color,
    outline: Color,
    marking: Color,
}

impl CatPalette {
    fn from_genome(genome: &CatGenome) -> Self {
        let hue = genome.base_hue;
        let head = hsl_to_color(
            hue,
            0.42 + genome.head_saturation * 0.25,
            0.55 + genome.head_lightness * 0.16,
        );
        let background = hsl_to_color(
            (hue + 180.0 + genome.background_shift * 40.0) % 360.0,
            0.25 + genome.background_sat * 0.20,
            0.90,
        );
        let accent = hsl_to_color(
            (hue + 18.0 + genome.accent_shift * 60.0) % 360.0,
            0.34 + genome.accent_sat * 0.20,
            0.80,
        );

        Self {
            background,
            accent,
            head,
            ear_inner: hsl_to_color(
                hue - 6.0,
                0.50 + genome.ear_inner_sat * 0.20,
                0.72 + genome.ear_inner_light * 0.12,
            ),
            muzzle: hsl_to_color(
                hue + 8.0,
                0.18 + genome.muzzle_sat * 0.16,
                0.84 + genome.muzzle_light * 0.10,
            ),
            eye: hsl_to_color(
                genome.eye_hue,
                0.65 + genome.eye_sat * 0.20,
                0.50 + genome.eye_light * 0.12,
            ),
            pupil: Color::rgb(28, 24, 18),
            nose: hsl_to_color(
                344.0 + genome.nose_hue * 18.0,
                0.58 + genome.nose_sat * 0.18,
                0.66 + genome.nose_light * 0.10,
            ),
            outline: Color::rgb(64, 45, 32),
            marking: hsl_to_color(
                hue + genome.marking_hue_shift * 24.0,
                0.25 + genome.marking_sat * 0.20,
                0.42 + genome.marking_light * 0.16,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CatGenome {
    base_hue: f32,
    eye_hue: f32,
    head_saturation: f32,
    head_lightness: f32,
    background_shift: f32,
    background_sat: f32,
    accent_shift: f32,
    accent_sat: f32,
    ear_inner_sat: f32,
    ear_inner_light: f32,
    muzzle_sat: f32,
    muzzle_light: f32,
    eye_sat: f32,
    eye_light: f32,
    nose_hue: f32,
    nose_sat: f32,
    nose_light: f32,
    marking_hue_shift: f32,
    marking_sat: f32,
    marking_light: f32,
    head_width: f32,
    head_height: f32,
    head_drop: f32,
    ear_width: f32,
    ear_height: f32,
    ear_tilt: f32,
    muzzle_width: f32,
    muzzle_height: f32,
    eye_spacing: f32,
    eye_width: f32,
    eye_height: f32,
    pupil_width: f32,
    whisker_len: f32,
    whisker_tilt: f32,
    smile_width: f32,
    smile_depth: f32,
    accent_band_height: f32,
    forehead_mark: f32,
    cheek_spots: f32,
    stripe_count: u8,
}

impl CatGenome {
    fn from_identity(identity: &AvatarIdentity, rng: &mut StdRng) -> Self {
        let mut noise =
            |idx: usize| (identity.unit_f32(idx) + rng.random_range(0.0..0.03)).min(1.0);
        Self {
            base_hue: 12.0 + identity.unit_f32(0) * 300.0,
            eye_hue: 45.0 + identity.unit_f32(1) * 120.0,
            head_saturation: noise(2),
            head_lightness: noise(3),
            background_shift: noise(4),
            background_sat: noise(5),
            accent_shift: noise(6),
            accent_sat: noise(7),
            ear_inner_sat: noise(8),
            ear_inner_light: noise(9),
            muzzle_sat: noise(10),
            muzzle_light: noise(11),
            eye_sat: noise(12),
            eye_light: noise(13),
            nose_hue: noise(14),
            nose_sat: noise(15),
            nose_light: noise(16),
            marking_hue_shift: identity.unit_f32(17) * 2.0 - 1.0,
            marking_sat: noise(18),
            marking_light: noise(19),
            head_width: noise(20),
            head_height: noise(21),
            head_drop: noise(22),
            ear_width: noise(23),
            ear_height: noise(24),
            ear_tilt: identity.unit_f32(25) * 2.0 - 1.0,
            muzzle_width: noise(26),
            muzzle_height: noise(27),
            eye_spacing: noise(28),
            eye_width: noise(29),
            eye_height: noise(30),
            pupil_width: noise(31),
            whisker_len: noise(32),
            whisker_tilt: identity.unit_f32(33) * 2.0 - 1.0,
            smile_width: noise(34),
            smile_depth: noise(35),
            accent_band_height: noise(36),
            forehead_mark: noise(37),
            cheek_spots: noise(38),
            stripe_count: 2 + (identity.byte(39) % 4),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct EarSpec {
    outer: [Point<i32>; 3],
    inner: [Point<i32>; 3],
}

impl EarSpec {
    fn left(
        center_x: i32,
        center_y: i32,
        head_rx: i32,
        head_ry: i32,
        ear_width: i32,
        ear_height: i32,
        ear_tilt: f32,
    ) -> Self {
        let base_x = center_x - head_rx / 2;
        let base_y = center_y - head_ry + 12;
        let tip_shift = (ear_width as f32 * 0.35 * ear_tilt) as i32;
        Self {
            outer: [
                Point::new(base_x - ear_width / 2, base_y + ear_height / 2),
                Point::new(base_x + ear_width / 3 + tip_shift, base_y - ear_height),
                Point::new(base_x + ear_width, base_y + ear_height / 3),
            ],
            inner: [
                Point::new(base_x - ear_width / 6, base_y + ear_height / 4),
                Point::new(
                    base_x + ear_width / 4 + tip_shift / 2,
                    base_y - (ear_height as f32 * 0.55) as i32,
                ),
                Point::new(
                    base_x + (ear_width as f32 * 0.6) as i32,
                    base_y + ear_height / 8,
                ),
            ],
        }
    }

    fn right(
        center_x: i32,
        center_y: i32,
        head_rx: i32,
        head_ry: i32,
        ear_width: i32,
        ear_height: i32,
        ear_tilt: f32,
    ) -> Self {
        let base_x = center_x + head_rx / 2;
        let base_y = center_y - head_ry + 12;
        let tip_shift = (ear_width as f32 * 0.35 * ear_tilt) as i32;
        Self {
            outer: [
                Point::new(base_x - ear_width, base_y + ear_height / 3),
                Point::new(base_x - ear_width / 3 - tip_shift, base_y - ear_height),
                Point::new(base_x + ear_width / 2, base_y + ear_height / 2),
            ],
            inner: [
                Point::new(
                    base_x - (ear_width as f32 * 0.6) as i32,
                    base_y + ear_height / 8,
                ),
                Point::new(
                    base_x - ear_width / 4 - tip_shift / 2,
                    base_y - (ear_height as f32 * 0.55) as i32,
                ),
                Point::new(base_x + ear_width / 6, base_y + ear_height / 4),
            ],
        }
    }
}

fn draw_background_accent(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    accent: Color,
    accent_band_height: f32,
    background: AvatarBackground,
) {
    if background == AvatarBackground::White {
        return;
    }
    let width = image.width() as i32;
    let stripe_top = center_y - head_ry - 18;
    let stripe_height = ((head_ry as f32) * (0.25 + accent_band_height * 0.45)) as i32;

    draw_filled_rect_mut(
        image,
        Rect::at(0, stripe_top.max(0)).of_size(width as u32, stripe_height.max(1) as u32),
        accent.into(),
    );
    draw_filled_circle_mut(
        image,
        (center_x + head_rx / 2, center_y - head_ry / 2),
        head_ry / 3,
        Color::rgba(accent.0[0], accent.0[1], accent.0[2], 180).into(),
    );
}

fn draw_ear(
    image: &mut RgbaImage,
    spec: EarSpec,
    outer_color: Color,
    inner_color: Color,
    outline: Color,
) {
    draw_polygon_mut(image, &spec.outer, outer_color.into());
    draw_polygon_mut(image, &spec.inner, inner_color.into());

    for edge in spec.outer.windows(2) {
        draw_antialiased_line_segment_mut(
            image,
            (edge[0].x, edge[0].y),
            (edge[1].x, edge[1].y),
            outline.into(),
            interpolate,
        );
    }
    draw_antialiased_line_segment_mut(
        image,
        (spec.outer[2].x, spec.outer[2].y),
        (spec.outer[0].x, spec.outer[0].y),
        outline.into(),
        interpolate,
    );
}

fn draw_eyes(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    palette: CatPalette,
    genome: CatGenome,
) {
    let eye_offset_x = (head_rx as f32 * (0.31 + genome.eye_spacing * 0.18)) as i32;
    let eye_y = center_y - head_ry / 6;
    let eye_rx = (head_rx as f32 * (0.12 + genome.eye_width * 0.10)) as i32;
    let eye_ry = (head_ry as f32 * (0.11 + genome.eye_height * 0.10)) as i32;
    let pupil_ry = (eye_ry as f32 * 0.90) as i32;
    let pupil_rx = ((eye_rx as f32) * (0.12 + genome.pupil_width * 0.18)) as i32;

    for eye_x in [center_x - eye_offset_x, center_x + eye_offset_x] {
        draw_filled_ellipse_mut(image, (eye_x, eye_y), eye_rx, eye_ry, palette.eye.into());
        draw_filled_ellipse_mut(
            image,
            (eye_x, eye_y),
            pupil_rx,
            pupil_ry,
            palette.pupil.into(),
        );
        draw_filled_circle_mut(
            image,
            (eye_x - eye_rx / 3, eye_y - eye_ry / 3),
            (eye_rx as f32 * 0.15) as i32,
            Color::rgba(255, 255, 255, 220).into(),
        );
    }
}

fn draw_nose_and_mouth(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    palette: CatPalette,
    genome: CatGenome,
) {
    let nose_y = center_y + head_ry / 7;
    let nose_half_width = (head_rx as f32 * (0.08 + genome.muzzle_width * 0.05)) as i32;
    let nose_height = (head_ry as f32 * (0.08 + genome.muzzle_height * 0.05)) as i32;
    let nose = [
        Point::new(center_x - nose_half_width, nose_y),
        Point::new(center_x + nose_half_width, nose_y),
        Point::new(center_x, nose_y + nose_height),
    ];
    draw_polygon_mut(image, &nose, palette.nose.into());

    let mouth_top = nose_y + nose_height;
    draw_line_segment_mut(
        image,
        (center_x as f32, mouth_top as f32),
        (center_x as f32, (mouth_top + head_ry / 8) as f32),
        palette.outline.into(),
    );

    let smile_radius = (head_rx as f32 * (0.08 + genome.smile_width * 0.10)) as i32;
    draw_smile_arc(
        image,
        center_x - smile_radius,
        mouth_top + smile_radius / 2,
        smile_radius,
        palette.outline,
        genome.smile_depth,
    );
    draw_smile_arc(
        image,
        center_x + smile_radius,
        mouth_top + smile_radius / 2,
        smile_radius,
        palette.outline,
        genome.smile_depth,
    );
}

fn draw_smile_arc(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: Color,
    smile_depth: f32,
) {
    for step in 20..=160 {
        let theta = (step as f32).to_radians();
        let x = center_x as f32 + theta.cos() * radius as f32 * 0.55;
        let y = center_y as f32 + theta.sin() * radius as f32 * (0.24 + smile_depth * 0.28);
        draw_filled_circle_mut(image, (x.round() as i32, y.round() as i32), 1, color.into());
    }
}

fn draw_whiskers(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    color: Color,
    genome: CatGenome,
) {
    let muzzle_y = center_y + head_ry / 5;
    let left_start = center_x - head_rx / 6;
    let right_start = center_x + head_rx / 6;
    let whisker_len = (head_rx as f32 * (0.58 + genome.whisker_len * 0.42)) as i32;
    let whisker_slope = (genome.whisker_tilt * 12.0) as i32;

    for offset in [-12, 0, 12] {
        draw_antialiased_line_segment_mut(
            image,
            (left_start, muzzle_y + offset),
            (
                left_start - whisker_len,
                muzzle_y + offset - 8 + whisker_slope,
            ),
            color.into(),
            interpolate,
        );
        draw_antialiased_line_segment_mut(
            image,
            (right_start, muzzle_y + offset),
            (
                right_start + whisker_len,
                muzzle_y + offset - 8 - whisker_slope,
            ),
            color.into(),
            interpolate,
        );
    }
}

fn draw_cat_markings(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    color: Color,
    genome: CatGenome,
) {
    let stripe_count = genome.stripe_count as i32;
    let forehead_y = center_y - head_ry / 2;
    let stripe_spacing = (head_rx / 5).max(6);
    let stripe_length = ((head_ry as f32) * (0.14 + genome.forehead_mark * 0.12)) as i32;

    for stripe in 0..stripe_count {
        let offset = stripe - stripe_count / 2;
        let x = center_x + offset * stripe_spacing / 2;
        draw_line_segment_mut(
            image,
            (x as f32, forehead_y as f32),
            ((x + offset * 2) as f32, (forehead_y + stripe_length) as f32),
            color.into(),
        );
    }

    if genome.cheek_spots > 0.35 {
        let cheek_y = center_y + head_ry / 5;
        let cheek_x = (head_rx as f32 * 0.55) as i32;
        let cheek_radius = ((head_rx as f32) * (0.05 + genome.cheek_spots * 0.04)) as i32;
        draw_filled_circle_mut(
            image,
            (center_x - cheek_x, cheek_y),
            cheek_radius,
            Color::rgba(color.0[0], color.0[1], color.0[2], 120).into(),
        );
        draw_filled_circle_mut(
            image,
            (center_x + cheek_x, cheek_y),
            cheek_radius,
            Color::rgba(color.0[0], color.0[1], color.0[2], 120).into(),
        );
    }
}

fn hsl_to_color(hue: f32, saturation: f32, lightness: f32) -> Color {
    let rgb_u8: Srgb<u8> = Srgb::from_color(Hsl::new(hue, saturation, lightness)).into_format();
    Color::rgb(rgb_u8.red, rgb_u8.green, rgb_u8.blue)
}

fn background_fill(background: AvatarBackground, themed: Color) -> Color {
    match background {
        AvatarBackground::Themed => themed,
        AvatarBackground::White => Color::rgb(255, 255, 255),
    }
}

fn color_hex(color: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0[0], color.0[1], color.0[2])
}

fn render_cat_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let rx = w * (0.26 + identity.unit_f32(20) * 0.07);
    let ry = h * (0.22 + identity.unit_f32(21) * 0.08);
    let head = hsl_to_color(20.0 + identity.unit_f32(0) * 40.0, 0.48, 0.64);
    let muzzle = hsl_to_color(28.0 + identity.unit_f32(1) * 18.0, 0.18, 0.90);
    let eye = hsl_to_color(90.0 + identity.unit_f32(2) * 40.0, 0.7, 0.55);
    let outline = Color::rgb(64, 45, 32);
    let left_ear = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.8,
        cy - ry * 0.4,
        cx - rx * 0.4,
        cy - ry * 1.3,
        cx - rx * 0.1,
        cy - ry * 0.1
    );
    let right_ear = format!(
        "{},{} {},{} {},{}",
        cx + rx * 0.8,
        cy - ry * 0.4,
        cx + rx * 0.4,
        cy - ry * 1.3,
        cx + rx * 0.1,
        cy - ry * 0.1
    );
    let nose = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.06,
        cy + ry * 0.1,
        cx + rx * 0.06,
        cy + ry * 0.1,
        cx,
        cy + ry * 0.2
    );
    format!(
        r##"<polygon points="{left_ear}" fill="{head}"/><polygon points="{right_ear}" fill="{head}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{head}"/><ellipse cx="{cx}" cy="{muzzle_y}" rx="{muzzle_rx}" ry="{muzzle_ry}" fill="{muzzle}"/><ellipse cx="{left_eye_x}" cy="{eye_y}" rx="{eye_rx}" ry="{eye_ry}" fill="{eye}"/><ellipse cx="{right_eye_x}" cy="{eye_y}" rx="{eye_rx}" ry="{eye_ry}" fill="{eye}"/><polygon points="{nose}" fill="#d6818d"/><path d="M {left_mx} {mouth_y} q {curve_x} {curve_y} {curve_end} 0 M {right_mx} {mouth_y} q {curve_x} {curve_y} {curve_end} 0" stroke="{outline}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        left_ear = left_ear,
        right_ear = right_ear,
        head = color_hex(head),
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        muzzle_y = cy + ry * 0.28,
        muzzle_rx = rx * 0.45,
        muzzle_ry = ry * 0.28,
        muzzle = color_hex(muzzle),
        left_eye_x = cx - rx * 0.34,
        right_eye_x = cx + rx * 0.34,
        eye_y = cy - ry * 0.1,
        eye_rx = rx * 0.13,
        eye_ry = ry * 0.16,
        eye = color_hex(eye),
        nose = nose,
        left_mx = cx - rx * 0.08,
        right_mx = cx + rx * 0.08,
        mouth_y = cy + ry * 0.22,
        curve_x = rx * 0.1,
        curve_y = ry * 0.12,
        curve_end = rx * 0.16,
        outline = color_hex(outline),
    )
}

fn render_dog_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let fur = hsl_to_color(18.0 + identity.unit_f32(5) * 45.0, 0.42, 0.60);
    let ear = hsl_to_color(18.0 + identity.unit_f32(6) * 30.0, 0.44, 0.40);
    let muzzle = hsl_to_color(34.0, 0.18, 0.92);
    format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><circle cx="{}" cy="{}" r="{}" fill="#fff"/><circle cx="{}" cy="{}" r="{}" fill="#241a14"/><circle cx="{}" cy="{}" r="{}" fill="#fff"/><circle cx="{}" cy="{}" r="{}" fill="#241a14"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#2d2422"/><path d="M {} {} q {} {} {} 0 M {} {} q {} {} {} 0" stroke="#2d2422" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx - w * 0.14,
        cy - h * 0.03,
        w * 0.09,
        h * 0.18,
        color_hex(ear),
        cx + w * 0.14,
        cy - h * 0.03,
        w * 0.09,
        h * 0.18,
        color_hex(ear),
        cx,
        cy,
        w * 0.26,
        h * 0.24,
        color_hex(fur),
        cx,
        cy + h * 0.08,
        w * 0.12,
        h * 0.07,
        color_hex(muzzle),
        cx - w * 0.08,
        cy - h * 0.05,
        w * 0.03,
        cx - w * 0.08,
        cy - h * 0.05,
        w * 0.015,
        cx + w * 0.08,
        cy - h * 0.05,
        w * 0.03,
        cx + w * 0.08,
        cy - h * 0.05,
        w * 0.015,
        cx,
        cy + h * 0.06,
        w * 0.035,
        h * 0.026,
        cx - w * 0.03,
        cy + h * 0.09,
        w * 0.05,
        h * 0.05,
        w * 0.10,
        cx + w * 0.03,
        cy + h * 0.09,
        w * 0.05,
        h * 0.05,
        w * 0.10,
    )
}

fn render_robot_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let metal = hsl_to_color(205.0 + identity.unit_f32(3) * 25.0, 0.16, 0.74);
    let trim = hsl_to_color(205.0 + identity.unit_f32(4) * 22.0, 0.18, 0.46);
    let light = hsl_to_color(60.0 + identity.unit_f32(5) * 110.0, 0.78, 0.66);
    let head_w = w * 0.48;
    let head_h = h * 0.38;
    let x = cx - head_w / 2.0;
    let y = cy - head_h / 2.0;
    format!(
        r##"<line x1="{cx}" y1="{a1}" x2="{cx}" y2="{a2}" stroke="{trim}" stroke-width="4"/><circle cx="{cx}" cy="{a1}" r="{ar}" fill="{light}"/><rect x="{x}" y="{y}" width="{head_w}" height="{head_h}" rx="14" fill="{metal}" stroke="{trim}" stroke-width="4"/><ellipse cx="{ex1}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{light}"/><ellipse cx="{ex2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{light}"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="6" fill="#2f3c48"/><circle cx="{bx1}" cy="{cy}" r="{br}" fill="{trim}"/><circle cx="{bx2}" cy="{cy}" r="{br}" fill="{trim}"/>"##,
        cx = cx,
        a1 = y - h * 0.10,
        a2 = y,
        ar = w * 0.02,
        x = x,
        y = y,
        head_w = head_w,
        head_h = head_h,
        metal = color_hex(metal),
        trim = color_hex(trim),
        light = color_hex(light),
        ex1 = cx - head_w * 0.24,
        ex2 = cx + head_w * 0.24,
        ey = cy - head_h * 0.14,
        erx = w * 0.055,
        ery = h * 0.04,
        mx = cx - head_w * 0.18,
        my = cy + head_h * 0.12,
        mw = head_w * 0.36,
        mh = head_h * 0.10,
        bx1 = x + head_w * 0.1,
        bx2 = x + head_w * 0.9,
        br = w * 0.02,
    )
}

fn render_fox_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let orange = hsl_to_color(18.0 + identity.unit_f32(1) * 20.0, 0.76, 0.58);
    let cream = hsl_to_color(40.0, 0.30, 0.94);
    format!(
        r##"<polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#fff"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#fff"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#221c18"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#221c18"/>"##,
        cx - w * 0.18,
        cy - h * 0.12,
        cx - w * 0.10,
        cy - h * 0.28,
        cx - w * 0.02,
        cy - h * 0.05,
        color_hex(orange),
        cx + w * 0.18,
        cy - h * 0.12,
        cx + w * 0.10,
        cy - h * 0.28,
        cx + w * 0.02,
        cy - h * 0.05,
        color_hex(orange),
        cx,
        cy,
        w * 0.24,
        h * 0.22,
        color_hex(orange),
        cx - w * 0.18,
        cy - h * 0.03,
        cx,
        cy + h * 0.10,
        cx - w * 0.06,
        cy + h * 0.18,
        color_hex(cream),
        cx + w * 0.18,
        cy - h * 0.03,
        cx,
        cy + h * 0.10,
        cx + w * 0.06,
        cy + h * 0.18,
        color_hex(cream),
        cx - w * 0.08,
        cy - h * 0.04,
        w * 0.03,
        h * 0.03,
        cx + w * 0.08,
        cy - h * 0.04,
        w * 0.03,
        h * 0.03,
        cx - w * 0.08,
        cy - h * 0.04,
        w * 0.013,
        h * 0.022,
        cx + w * 0.08,
        cy - h * 0.04,
        w * 0.013,
        h * 0.022,
    )
}

fn render_alien_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let skin = hsl_to_color(90.0 + identity.unit_f32(0) * 80.0, 0.48, 0.70);
    format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#261832"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#261832"/><circle cx="{}" cy="{}" r="{}" fill="#5e8c58"/>"##,
        cx,
        cy,
        w * 0.18,
        h * 0.28,
        color_hex(skin),
        cx - w * 0.08,
        cy - h * 0.07,
        w * 0.04,
        h * 0.09,
        cx + w * 0.08,
        cy - h * 0.07,
        w * 0.04,
        h * 0.09,
        cx,
        cy + h * 0.03,
        w * 0.012,
    )
}

fn render_monster_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let skin = hsl_to_color(
        identity.unit_f32(0) * 360.0,
        0.52 + identity.unit_f32(1) * 0.20,
        0.50 + identity.unit_f32(2) * 0.16,
    );
    let shade = hsl_to_color(
        identity.unit_f32(3) * 360.0,
        0.40 + identity.unit_f32(4) * 0.16,
        0.26 + identity.unit_f32(5) * 0.08,
    );
    let eyes = 1 + (identity.byte(12) % 3) as usize;
    let eye_spacing = if eyes == 1 { 0.0 } else { w * 0.22 / (eyes - 1) as f32 };
    let eye_start = cx - eye_spacing * (eyes.saturating_sub(1) as f32) / 2.0;
    let mut eye_markup = String::new();
    for index in 0..eyes {
        let ex = eye_start + eye_spacing * index as f32;
        eye_markup.push_str(&format!(
            r##"<ellipse cx="{ex}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#fcf8ec"/><ellipse cx="{ex}" cy="{ey}" rx="{prx}" ry="{pry}" fill="#18141c"/>"##,
            ey = cy - h * 0.08,
            erx = w * 0.038,
            ery = h * 0.042,
            prx = w * 0.012,
            pry = h * 0.030,
        ));
    }

    let horns = if identity.byte(18).is_multiple_of(2) {
        format!(
            r#"<polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/>"#,
            cx - w * 0.18,
            cy - h * 0.18,
            cx - w * 0.22,
            cy - h * 0.34,
            cx - w * 0.08,
            cy - h * 0.14,
            color_hex(shade),
            cx + w * 0.18,
            cy - h * 0.18,
            cx + w * 0.22,
            cy - h * 0.34,
            cx + w * 0.08,
            cy - h * 0.14,
            color_hex(shade),
        )
    } else {
        format!(
            r#"<polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/>"#,
            cx - w * 0.12,
            cy - h * 0.14,
            cx - w * 0.08,
            cy - h * 0.30,
            cx - w * 0.02,
            cy - h * 0.14,
            color_hex(shade),
            cx,
            cy - h * 0.15,
            cx,
            cy - h * 0.32,
            cx + w * 0.05,
            cy - h * 0.15,
            color_hex(shade),
            cx + w * 0.12,
            cy - h * 0.14,
            cx + w * 0.08,
            cy - h * 0.30,
            cx + w * 0.02,
            cy - h * 0.14,
            color_hex(shade),
        )
    };

    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{skin}"/>{horns}<circle cx="{sx1}" cy="{sy1}" r="{sr}" fill="{shade}" fill-opacity="0.55"/><circle cx="{sx2}" cy="{sy2}" r="{sr2}" fill="{shade}" fill-opacity="0.55"/>{eye_markup}<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="{mr}" fill="#301218"/><polygon points="{tx1},{ty1}, {tx2},{ty1}, {txm1},{ty2}" fill="#fcf8ec"/><polygon points="{tx3},{ty1}, {tx4},{ty1}, {txm2},{ty2}" fill="#fcf8ec"/>"##,
        cx = cx,
        cy = cy,
        rx = w * 0.24,
        ry = h * 0.23,
        skin = color_hex(skin),
        horns = horns,
        shade = color_hex(shade),
        sx1 = cx - w * 0.12,
        sy1 = cy - h * 0.02,
        sr = w * 0.034,
        sx2 = cx + w * 0.14,
        sy2 = cy + h * 0.07,
        sr2 = w * 0.026,
        eye_markup = eye_markup,
        mx = cx - w * 0.14,
        my = cy + h * 0.08,
        mw = w * 0.28,
        mh = h * 0.09,
        mr = w * 0.02,
        tx1 = cx - w * 0.10,
        tx2 = cx - w * 0.06,
        txm1 = cx - w * 0.08,
        tx3 = cx + w * 0.06,
        tx4 = cx + w * 0.10,
        txm2 = cx + w * 0.08,
        ty1 = cy + h * 0.08,
        ty2 = cy + h * 0.16,
    )
}

fn render_ghost_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let body = hsl_to_color(200.0 + identity.unit_f32(1) * 30.0, 0.14, 0.98);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{body}"/><rect x="{x}" y="{cy}" width="{rw}" height="{rh}" fill="{body}"/><circle cx="{c1}" cy="{scy}" r="{sr}" fill="{body}"/><circle cx="{c2}" cy="{scy}" r="{sr}" fill="{body}"/><circle cx="{c3}" cy="{scy}" r="{sr}" fill="{body}"/><ellipse cx="{lx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/><ellipse cx="{rx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/><path d="M {mx1} {my} q {cq} {cyq} {ce} 0 M {mx2} {my} q {cq} {cyq} {ce} 0" stroke="#8da0b2" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = w * 0.22,
        ry = h * 0.24,
        body = color_hex(body),
        x = cx - w * 0.22,
        rw = w * 0.44,
        rh = h * 0.22,
        c1 = cx - w * 0.16,
        c2 = cx,
        c3 = cx + w * 0.16,
        scy = cy + h * 0.22,
        sr = w * 0.06,
        lx = cx - w * 0.08,
        rx2 = cx + w * 0.08,
        ey = cy - h * 0.06,
        erx = w * 0.025,
        ery = h * 0.05,
        mx1 = cx - w * 0.03,
        mx2 = cx + w * 0.03,
        my = cy + h * 0.08,
        cq = w * 0.04,
        cyq = h * 0.05,
        ce = w * 0.06,
    )
}

fn render_slime_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let slime = hsl_to_color(95.0 + identity.unit_f32(4) * 70.0, 0.52, 0.56);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{slime}"/><rect x="{dx1}" y="{cy}" width="{dw}" height="{dh1}" fill="{slime}"/><rect x="{dx2}" y="{cy}" width="{dw}" height="{dh2}" fill="{slime}"/><rect x="{dx3}" y="{cy}" width="{dw}" height="{dh3}" fill="{slime}"/><circle cx="{lx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{cx}" cy="{ey2}" r="{er2}" fill="#f8ffec"/><circle cx="{rx2}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#203018"/><circle cx="{cx}" cy="{ey2}" r="{pr}" fill="#203018"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#203018"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="{mr}" fill="#305228"/>"##,
        cx = cx,
        cy = cy,
        rx = w * 0.24,
        ry = h * 0.18,
        slime = color_hex(slime),
        dx1 = cx - w * 0.16,
        dx2 = cx - w * 0.04,
        dx3 = cx + w * 0.08,
        dw = w * 0.08,
        dh1 = h * 0.16,
        dh2 = h * 0.12,
        dh3 = h * 0.18,
        lx = cx - w * 0.08,
        rx2 = cx + w * 0.08,
        ey = cy - h * 0.06,
        ey2 = cy - h * 0.04,
        er = w * 0.03,
        er2 = w * 0.026,
        pr = w * 0.012,
        mx = cx - w * 0.10,
        my = cy + h * 0.08,
        mw = w * 0.20,
        mh = h * 0.02,
        mr = w * 0.01,
    )
}

fn render_bird_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let plumage = hsl_to_color(identity.unit_f32(7) * 360.0, 0.42, 0.62);
    let wing = hsl_to_color(20.0 + identity.unit_f32(8) * 160.0, 0.32, 0.46);
    let beak = hsl_to_color(32.0 + identity.unit_f32(9) * 26.0, 0.82, 0.58);
    format!(
        r##"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{plumage}"/><ellipse cx="{lx}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{wing}"/><ellipse cx="{rx2}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{wing}"/><polygon points="{cx},{cy} {bx},{by} {cx},{by2}" fill="{beak}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#181822"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#181822"/>"##,
        cx = cx,
        cy = cy,
        r = w * 0.22,
        plumage = color_hex(plumage),
        lx = cx - w * 0.12,
        rx2 = cx + w * 0.12,
        wy = cy + h * 0.04,
        wrx = w * 0.08,
        wry = h * 0.12,
        wing = color_hex(wing),
        bx = cx + w * 0.12,
        by = cy + h * 0.04,
        by2 = cy + h * 0.10,
        beak = color_hex(beak),
        elx = cx - w * 0.07,
        erx = cx + w * 0.07,
        ey = cy - h * 0.05,
        er = w * 0.028,
        pr = w * 0.012,
    )
}

fn render_wizard_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.60;
    let hat = hsl_to_color(230.0 + identity.unit_f32(11) * 50.0, 0.42, 0.38);
    let band = hsl_to_color(28.0 + identity.unit_f32(12) * 30.0, 0.74, 0.58);
    let skin = hsl_to_color(22.0 + identity.unit_f32(13) * 18.0, 0.30, 0.82);
    let beard = hsl_to_color(40.0 + identity.unit_f32(14) * 25.0, 0.10, 0.92);
    format!(
        r##"<polygon points="{x1},{y1} {x2},{y1} {cx},{y2}" fill="{hat}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{band}"/><circle cx="{cx}" cy="{cy}" r="{r}" fill="{skin}"/><polygon points="{b1},{b2} {b3},{b2} {cx},{b4}" fill="{beard}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{sx}" cy="{sy}" r="{sr}" fill="{band}"/>"##,
        cx = cx,
        cy = cy,
        x1 = cx - w * 0.18,
        x2 = cx + w * 0.18,
        y1 = cy - h * 0.08,
        y2 = cy - h * 0.34,
        hat = color_hex(hat),
        bx = cx - w * 0.26,
        by = cy - h * 0.08,
        bw = w * 0.52,
        bh = h * 0.04,
        band = color_hex(band),
        r = w * 0.18,
        skin = color_hex(skin),
        b1 = cx - w * 0.10,
        b2 = cy + h * 0.06,
        b3 = cx + w * 0.10,
        b4 = cy + h * 0.26,
        beard = color_hex(beard),
        elx = cx - w * 0.06,
        erx = cx + w * 0.06,
        ey = cy - h * 0.03,
        er = w * 0.024,
        pr = w * 0.010,
        sx = cx + w * 0.12,
        sy = cy - h * 0.28,
        sr = w * 0.018,
    )
}

fn render_skull_svg(spec: AvatarSpec, _identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.54;
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="#e8e1d6"/><rect x="{jx}" y="{jy}" width="{jw}" height="{jh}" fill="#e8e1d6"/><ellipse cx="{elx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#2c2a2c"/><ellipse cx="{erx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#2c2a2c"/><polygon points="{cx},{ny} {nx1},{ny2} {nx2},{ny2}" fill="#5a544e"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" fill="#5a544e"/><line x1="{t1}" y1="{ty1}" x2="{t1}" y2="{ty2}" stroke="#5a544e" stroke-width="3"/><line x1="{t2}" y1="{ty1}" x2="{t2}" y2="{ty2}" stroke="#5a544e" stroke-width="3"/><line x1="{t3}" y1="{ty1}" x2="{t3}" y2="{ty2}" stroke="#5a544e" stroke-width="3"/>"##,
        cx = cx,
        cy = cy,
        rx = w * 0.20,
        ry = h * 0.20,
        jx = cx - w * 0.10,
        jy = cy + h * 0.10,
        jw = w * 0.20,
        jh = h * 0.10,
        elx = cx - w * 0.07,
        erx2 = cx + w * 0.07,
        ey = cy - h * 0.04,
        erx = w * 0.04,
        ery = h * 0.06,
        ny = cy,
        nx1 = cx - w * 0.02,
        nx2 = cx + w * 0.02,
        ny2 = cy + h * 0.08,
        mx = cx - w * 0.10,
        my = cy + h * 0.10,
        mw = w * 0.20,
        mh = h * 0.02,
        t1 = cx - w * 0.05,
        t2 = cx,
        t3 = cx + w * 0.05,
        ty1 = cy + h * 0.10,
        ty2 = cy + h * 0.20,
    )
}

fn render_paws_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let paw = hsl_to_color(identity.unit_f32(1) * 360.0, 0.38, 0.62);
    let pad = hsl_to_color(330.0 + identity.unit_f32(3) * 20.0, 0.40, 0.74);
    let cx = w * 0.52;
    let cy = h * 0.60;
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{prx}" ry="{pry}" fill="{paw}"/><ellipse cx="{cx}" cy="{py2}" rx="{padrx}" ry="{padry}" fill="{pad}"/><ellipse cx="{t1x}" cy="{ty1}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t2x}" cy="{ty2}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t3x}" cy="{ty2}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t4x}" cy="{ty1}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t1x}" cy="{ty1a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/><ellipse cx="{t2x}" cy="{ty2a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/><ellipse cx="{t3x}" cy="{ty2a}" rx="{padrx2}" ry="{pad}"/><ellipse cx="{t4x}" cy="{ty1a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/>"##,
        cx = cx,
        cy = cy,
        prx = w * 0.13,
        pry = h * 0.15,
        paw = color_hex(paw),
        py2 = cy + h * 0.015,
        padrx = w * 0.09,
        padry = h * 0.10,
        pad = color_hex(pad),
        t1x = cx - w * 0.12,
        t2x = cx - w * 0.04,
        t3x = cx + w * 0.04,
        t4x = cx + w * 0.12,
        ty1 = cy - h * 0.16,
        ty2 = cy - h * 0.14,
        ty1a = cy - h * 0.15,
        ty2a = cy - h * 0.13,
        trx = w * 0.035,
        try_ = h * 0.05,
        padrx2 = w * 0.022,
        padry2 = h * 0.032,
    )
}

fn encode_rgba_image(image: &RgbaImage, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    let mut bytes = Vec::new();
    {
        let cursor = Cursor::new(&mut bytes);
        encode_into_writer(image, format, cursor)?;
    }
    Ok(bytes)
}

fn write_rgba_image<P: AsRef<Path>>(
    image: &RgbaImage,
    format: AvatarOutputFormat,
    path: P,
) -> ImageResult<()> {
    let file = File::create(path).map_err(image::ImageError::IoError)?;
    let writer = BufWriter::new(file);
    encode_into_writer(image, format, writer)
}

fn encode_into_writer<W: std::io::Write>(
    image: &RgbaImage,
    format: AvatarOutputFormat,
    writer: W,
) -> ImageResult<()> {
    match format {
        AvatarOutputFormat::WebP => WebPEncoder::new_lossless(writer).write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        ),
        AvatarOutputFormat::Png => {
            PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    ExtendedColorType::Rgba8,
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;

    #[test]
    fn cat_avatar_is_deterministic_for_a_seed() {
        let spec = AvatarSpec::new(256, 256, 42);
        let left = render_cat_avatar(spec);
        let right = render_cat_avatar(spec);

        assert_eq!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn cat_avatar_uses_requested_dimensions() {
        let image = render_cat_avatar(AvatarSpec::new(192, 160, 7));

        assert_eq!(image.width(), 192);
        assert_eq!(image.height(), 160);
    }

    #[test]
    fn cat_avatar_has_non_background_pixels() {
        let spec = AvatarSpec::new(128, 128, 3);
        let image = render_cat_avatar(spec);
        let background = image.get_pixel(0, 0);

        assert!(image.pixels().any(|pixel| pixel != background));
    }

    #[test]
    fn avatar_identity_uses_sha512_digest() {
        let identity = AvatarIdentity::new("alice@example.com");

        assert_eq!(identity.as_digest().len(), 64);
        assert_ne!(identity.seed(), 0);
    }

    #[test]
    fn namespace_changes_identity_digest() {
        let left = AvatarIdentity::new_with_namespace(
            AvatarNamespace::new("tenant-a", "v2"),
            "alice@example.com",
        );
        let right = AvatarIdentity::new_with_namespace(
            AvatarNamespace::new("tenant-b", "v2"),
            "alice@example.com",
        );

        assert_ne!(left.as_digest(), right.as_digest());
    }

    #[test]
    fn hashed_cat_avatar_is_deterministic_for_same_id() {
        let spec = AvatarSpec::new(192, 192, 0);
        let left = render_cat_avatar_for_identity(spec, &AvatarIdentity::new("alice@example.com"));
        let right = render_cat_avatar_for_identity(spec, &AvatarIdentity::new("alice@example.com"));

        assert_eq!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn hashed_cat_avatar_changes_for_different_ids() {
        let spec = AvatarSpec::new(192, 192, 0);
        let left = render_cat_avatar_for_identity(spec, &AvatarIdentity::new("alice@example.com"));
        let right = render_cat_avatar_for_identity(spec, &AvatarIdentity::new("bob@example.com"));

        assert_ne!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn cat_avatar_webp_export_round_trips() {
        let bytes = encode_cat_avatar(AvatarSpec::new(128, 128, 11), AvatarOutputFormat::WebP)
            .expect("webp encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
            .expect("webp should decode");

        assert_eq!(decoded.width(), 128);
        assert_eq!(decoded.height(), 128);
    }

    #[test]
    fn cat_avatar_png_export_round_trips() {
        let bytes = encode_cat_avatar(AvatarSpec::new(96, 96, 99), AvatarOutputFormat::Png)
            .expect("png encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Png)
            .expect("png should decode");

        assert_eq!(decoded.width(), 96);
        assert_eq!(decoded.height(), 96);
    }

    #[test]
    fn webp_is_the_default_output_format() {
        assert_eq!(AvatarOutputFormat::default(), AvatarOutputFormat::WebP);
    }

    #[test]
    fn hashed_cat_avatar_webp_export_round_trips() {
        let bytes = encode_cat_avatar_for_id(
            AvatarSpec::new(128, 128, 0),
            "alice@example.com",
            AvatarOutputFormat::WebP,
        )
        .expect("webp encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
            .expect("webp should decode");

        assert_eq!(decoded.width(), 128);
        assert_eq!(decoded.height(), 128);
    }

    #[test]
    fn white_background_mode_renders_white_corner() {
        let image = render_cat_avatar_for_identity_with_background(
            AvatarSpec::new(128, 128, 0),
            &AvatarIdentity::new("alice@example.com"),
            AvatarBackground::White,
        );

        assert_eq!(image.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn dog_and_robot_variants_generate_distinct_images() {
        let spec = AvatarSpec::new(128, 128, 0);
        let id = AvatarIdentity::new("alice@example.com");
        let dog = render_dog_avatar_for_identity(spec, &id, AvatarBackground::Themed);
        let robot = render_robot_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(dog.as_raw(), robot.as_raw());
    }

    #[test]
    fn monster_variant_is_distinct_from_alien() {
        let spec = AvatarSpec::new(128, 128, 0);
        let id = AvatarIdentity::new("alice@example.com");
        let alien = render_alien_avatar_for_identity(spec, &id, AvatarBackground::Themed);
        let monster = render_monster_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(alien.as_raw(), monster.as_raw());
    }

    #[test]
    fn paws_variant_is_distinct_from_cat() {
        let spec = AvatarSpec::new(128, 128, 0);
        let id = AvatarIdentity::new("alice@example.com");
        let cat = render_cat_avatar_for_identity_with_background(spec, &id, AvatarBackground::Themed);
        let paws = render_paws_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(cat.as_raw(), paws.as_raw());
    }

    #[test]
    fn generic_avatar_encoder_supports_robot_and_white_background() {
        let bytes = encode_avatar_for_id(
            AvatarSpec::new(96, 96, 0),
            "robot@example.com",
            AvatarOutputFormat::WebP,
            AvatarOptions {
                kind: AvatarKind::Robot,
                background: AvatarBackground::White,
            },
        )
        .expect("robot webp encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
            .expect("robot webp should decode");

        assert_eq!(decoded.width(), 96);
        assert_eq!(decoded.height(), 96);
    }

    #[test]
    fn svg_export_contains_svg_root_and_kind_label() {
        let svg = render_avatar_svg_for_id(
            AvatarSpec::new(128, 128, 0),
            "vector@example.com",
            AvatarOptions::new(AvatarKind::Fox, AvatarBackground::White),
        );

        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("fox avatar"));
    }

    #[test]
    fn svg_output_is_minimal_and_safe() {
        let svg = render_avatar_svg_for_id(
            AvatarSpec::new(256, 256, 0),
            "ghost@example.com",
            AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
        );

        assert!(!svg.contains("<script"));
        assert!(!svg.contains("onload="));
        assert!(svg.len() < 8_000);
    }

    #[test]
    fn parser_round_trip_supports_public_enums() {
        for kind in AvatarKind::ALL {
            assert_eq!(kind.as_str().parse::<AvatarKind>().ok(), Some(kind));
        }
        for background in AvatarBackground::ALL {
            assert_eq!(
                background.as_str().parse::<AvatarBackground>().ok(),
                Some(background)
            );
        }
        for format in AvatarOutputFormat::ALL {
            assert_eq!(
                format.as_str().parse::<AvatarOutputFormat>().ok(),
                Some(format)
            );
        }
    }

    #[test]
    fn render_avatar_for_id_supports_all_avatar_kinds() {
        let spec = AvatarSpec::new(96, 96, 0);
        for kind in AvatarKind::ALL {
            let image = render_avatar_for_id(
                spec,
                "integration@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );
            assert_eq!(image.width(), 96);
            assert_eq!(image.height(), 96);
        }
    }

    #[test]
    fn visual_fingerprints_are_stable() {
        for (label, options) in regression_scenarios() {
            let image = render_avatar_for_id(
                AvatarSpec::new(128, 128, 0),
                "snapshot@example.com",
                options,
            );
            let fingerprint = image_fingerprint(&image);
            let expected =
                regression_fingerprint_for(label).expect("missing golden regression fingerprint");
            assert_eq!(fingerprint, expected, "fingerprint mismatch for {label}");
        }
    }

    #[ignore]
    #[test]
    fn print_visual_fingerprints() {
        for (label, options) in [
            (
                "cat-themed",
                AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
            ),
            (
                "cat-white",
                AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White),
            ),
            (
                "dog-themed",
                AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
            ),
            (
                "robot-white",
                AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White),
            ),
            (
                "monster-themed",
                AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed),
            ),
            (
                "ghost-themed",
                AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
            ),
            (
                "slime-white",
                AvatarOptions::new(AvatarKind::Slime, AvatarBackground::White),
            ),
            (
                "bird-themed",
                AvatarOptions::new(AvatarKind::Bird, AvatarBackground::Themed),
            ),
            (
                "wizard-white",
                AvatarOptions::new(AvatarKind::Wizard, AvatarBackground::White),
            ),
            (
                "skull-themed",
                AvatarOptions::new(AvatarKind::Skull, AvatarBackground::Themed),
            ),
            (
                "paws-themed",
                AvatarOptions::new(AvatarKind::Paws, AvatarBackground::Themed),
            ),
        ] {
            let image = render_avatar_for_id(
                AvatarSpec::new(128, 128, 0),
                "snapshot@example.com",
                options,
            );
            println!("{label}: {}", image_fingerprint(&image));
        }
    }

    fn regression_scenarios() -> [(&'static str, AvatarOptions); 11] {
        [
            ("cat-themed", AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed)),
            ("cat-white", AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White)),
            ("dog-themed", AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed)),
            ("robot-white", AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White)),
            ("monster-themed", AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed)),
            ("ghost-themed", AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed)),
            ("slime-white", AvatarOptions::new(AvatarKind::Slime, AvatarBackground::White)),
            ("bird-themed", AvatarOptions::new(AvatarKind::Bird, AvatarBackground::Themed)),
            ("wizard-white", AvatarOptions::new(AvatarKind::Wizard, AvatarBackground::White)),
            ("skull-themed", AvatarOptions::new(AvatarKind::Skull, AvatarBackground::Themed)),
            ("paws-themed", AvatarOptions::new(AvatarKind::Paws, AvatarBackground::Themed)),
        ]
    }

    fn regression_fingerprint_for(label: &str) -> Option<&'static str> {
        include_str!("../tests/golden_fingerprints.txt")
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .find_map(|line| {
                let (name, value) = line.split_once('=')?;
                (name.trim() == label).then_some(value.trim())
            })
    }

    fn image_fingerprint(image: &RgbaImage) -> String {
        let digest = Sha512::digest(image.as_raw());
        digest[..12]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    }
}
