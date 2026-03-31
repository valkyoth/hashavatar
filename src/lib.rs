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
        let mut hasher = Sha512::new();
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
}

impl AvatarKind {
    pub const ALL: [Self; 5] = [Self::Cat, Self::Dog, Self::Robot, Self::Fox, Self::Alien];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Robot => "robot",
            Self::Fox => "fox",
            Self::Alien => "alien",
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
        Self {
            identity: AvatarIdentity::new(input),
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
        Self {
            identity: AvatarIdentity::new(input),
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
        Self {
            identity: AvatarIdentity::new(input),
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
    match options.kind {
        AvatarKind::Cat => {
            let renderer = HashedCatAvatar::new(id);
            let image = render_cat_avatar_for_identity_with_background(
                spec,
                renderer.identity(),
                options.background,
            );
            encode_rgba_image(&image, format)
        }
        AvatarKind::Dog => {
            let renderer = HashedDogAvatar::new(id, options.background);
            encode_avatar(&renderer, spec, format)
        }
        AvatarKind::Robot => {
            let renderer = HashedRobotAvatar::new(id, options.background);
            encode_avatar(&renderer, spec, format)
        }
        AvatarKind::Fox => encode_rgba_image(
            &render_fox_avatar_for_identity(spec, &AvatarIdentity::new(id), options.background),
            format,
        ),
        AvatarKind::Alien => encode_rgba_image(
            &render_alien_avatar_for_identity(spec, &AvatarIdentity::new(id), options.background),
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
    let identity = AvatarIdentity::new(id);
    match options.kind {
        AvatarKind::Cat => {
            render_cat_avatar_for_identity_with_background(spec, &identity, options.background)
        }
        AvatarKind::Dog => render_dog_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Robot => render_robot_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Fox => render_fox_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Alien => render_alien_avatar_for_identity(spec, &identity, options.background),
    }
}

/// Render an avatar as a compact SVG string.
pub fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> String {
    let identity = AvatarIdentity::new(id);
    let bg = match options.background {
        AvatarBackground::Themed => match options.kind {
            AvatarKind::Cat => hsl_to_color(28.0 + identity.unit_f32(2) * 40.0, 0.25, 0.92),
            AvatarKind::Dog => hsl_to_color(200.0 + identity.unit_f32(3) * 60.0, 0.20, 0.92),
            AvatarKind::Robot => hsl_to_color(220.0 + identity.unit_f32(4) * 50.0, 0.18, 0.93),
            AvatarKind::Fox => hsl_to_color(18.0 + identity.unit_f32(5) * 30.0, 0.28, 0.93),
            AvatarKind::Alien => hsl_to_color(260.0 + identity.unit_f32(6) * 60.0, 0.20, 0.93),
        },
        AvatarBackground::White => Color::rgb(255, 255, 255),
    };

    let body = match options.kind {
        AvatarKind::Cat => render_cat_svg(spec, &identity),
        AvatarKind::Dog => render_dog_svg(spec, &identity),
        AvatarKind::Robot => render_robot_svg(spec, &identity),
        AvatarKind::Fox => render_fox_svg(spec, &identity),
        AvatarKind::Alien => render_alien_svg(spec, &identity),
    };

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label} avatar"><rect width="100%" height="100%" fill="{bg}"/>{body}</svg>"#,
        w = spec.width,
        h = spec.height,
        bg = color_hex(bg),
        label = options.kind.as_str(),
        body = body,
    )
}

pub fn export_avatar_svg_for_id<T: AsRef<[u8]>, P: AsRef<Path>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
    path: P,
) -> std::io::Result<()> {
    std::fs::write(path, render_avatar_svg_for_id(spec, id, options))
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
        let scenarios = [
            (
                AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
                "cat-themed",
                "d3613419db55afd76018121d",
            ),
            (
                AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White),
                "cat-white",
                "45a185a73cb4e7eaafae1f97",
            ),
            (
                AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
                "dog-themed",
                "8129402f95375c8e4cdac4dc",
            ),
            (
                AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White),
                "robot-white",
                "0d8d46e9cfa253c712490571",
            ),
        ];

        for (options, _label, expected) in scenarios {
            let image = render_avatar_for_id(
                AvatarSpec::new(128, 128, 0),
                "snapshot@example.com",
                options,
            );
            let fingerprint = image_fingerprint(&image);
            assert_eq!(fingerprint, expected);
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
        ] {
            let image = render_avatar_for_id(
                AvatarSpec::new(128, 128, 0),
                "snapshot@example.com",
                options,
            );
            println!("{label}: {}", image_fingerprint(&image));
        }
    }

    fn image_fingerprint(image: &RgbaImage) -> String {
        let digest = Sha512::digest(image.as_raw());
        digest[..12]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    }
}
