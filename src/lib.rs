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
//! let spec = AvatarSpec::new(256, 256, 0)?;
//! let bytes = encode_avatar_for_id(
//!     spec,
//!     "robot@hashavatar.app",
//!     AvatarOutputFormat::WebP,
//!     AvatarOptions {
//!         kind: AvatarKind::Robot,
//!         background: AvatarBackground::Transparent,
//!     },
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]

use std::io::Cursor;
use std::mem::swap;
use std::str::FromStr;

use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::error::{LimitError, LimitErrorKind};
use image::{
    ExtendedColorType, ImageBuffer, ImageEncoder, ImageError, ImageResult, Rgba, RgbaImage,
};
use palette::{FromColor, Hsl, Srgb};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use sha2::{Digest, Sha512};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

/// Rendering contract version for deterministic avatars.
///
/// Within a major crate release, the goal is to keep visuals stable for the
/// same `(namespace, id, kind, background, size)` tuple unless a documented bug
/// fix requires a targeted change.
pub const AVATAR_STYLE_VERSION: u32 = 2;

/// Smallest supported raster or SVG dimension.
pub const MIN_AVATAR_DIMENSION: u32 = 64;

/// Largest supported raster or SVG dimension.
///
/// This caps worst-case allocation and encoding work for callers that expose
/// avatar generation to untrusted input. A 2048 x 2048 RGBA buffer is 16 MiB
/// before encoder overhead.
pub const MAX_AVATAR_DIMENSION: u32 = 2048;

/// Largest supported identity input in bytes.
///
/// This prevents applications from accidentally hashing attacker-controlled
/// request bodies or other unbounded byte strings as avatar identities.
pub const MAX_AVATAR_ID_BYTES: usize = 1024;

/// Largest supported namespace component in bytes.
pub const MAX_AVATAR_NAMESPACE_COMPONENT_BYTES: usize = 128;

const HASH_DOMAIN: &[u8] = b"hashavatar";
const HASH_DOMAIN_ALGORITHM_COMPONENT: &[u8] = b"identity-hash";
#[cfg(feature = "xxh3")]
const HASH_XOF_CHUNK_COMPONENT: &[u8] = b"digest-chunk";

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

// Raster primitive implementations adapted from imageproc's MIT-licensed
// drawing modules. See THIRD_PARTY_NOTICES.md.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rect {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
}

impl Rect {
    const fn at(left: i32, top: i32) -> RectPosition {
        RectPosition { left, top }
    }

    const fn left(self) -> i32 {
        self.left
    }

    const fn top(self) -> i32 {
        self.top
    }

    fn right(self) -> i32 {
        let width_offset = self.width.saturating_sub(1).min(i32::MAX as u32) as i32;
        self.left.saturating_add(width_offset)
    }

    fn bottom(self) -> i32 {
        let height_offset = self.height.saturating_sub(1).min(i32::MAX as u32) as i32;
        self.top.saturating_add(height_offset)
    }

    const fn width(self) -> u32 {
        self.width
    }

    const fn height(self) -> u32 {
        self.height
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let left = self.left.max(other.left);
        let top = self.top.max(other.top);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(Self::at(left, top).of_size((right - left + 1) as u32, (bottom - top + 1) as u32))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RectPosition {
    left: i32,
    top: i32,
}

impl RectPosition {
    const fn of_size(self, width: u32, height: u32) -> Rect {
        Rect {
            left: self.left,
            top: self.top,
            width: if width == 0 { 1 } else { width },
            height: if height == 0 { 1 } else { height },
        }
    }
}

fn draw_filled_rect_mut(image: &mut RgbaImage, rect: Rect, color: Rgba<u8>) {
    if image.width() == 0 || image.height() == 0 {
        return;
    }

    let bounds = Rect::at(0, 0).of_size(image.width(), image.height());
    if let Some(intersection) = bounds.intersect(rect) {
        for dy in 0..intersection.height() {
            for dx in 0..intersection.width() {
                let x = intersection.left() as u32 + dx;
                let y = intersection.top() as u32 + dy;
                image.put_pixel(x, y, color);
            }
        }
    }
}

fn draw_line_segment_mut(
    image: &mut RgbaImage,
    start: (f32, f32),
    end: (f32, f32),
    color: Rgba<u8>,
) {
    for (x, y) in BresenhamLineIter::new(start, end) {
        draw_if_in_bounds(image, x, y, color);
    }
}

struct BresenhamLineIter {
    dx: f32,
    dy: f32,
    x: i32,
    y: i32,
    error: f32,
    end_x: i32,
    is_steep: bool,
    y_step: i32,
}

impl BresenhamLineIter {
    fn new(start: (f32, f32), end: (f32, f32)) -> Self {
        let (mut x0, mut y0) = (start.0, start.1);
        let (mut x1, mut y1) = (end.0, end.1);

        let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
        if is_steep {
            swap(&mut x0, &mut y0);
            swap(&mut x1, &mut y1);
        }

        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        Self {
            dx,
            dy: (y1 - y0).abs(),
            x: x0 as i32,
            y: y0 as i32,
            error: dx / 2.0,
            end_x: x1 as i32,
            is_steep,
            y_step: if y0 < y1 { 1 } else { -1 },
        }
    }
}

impl Iterator for BresenhamLineIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x > self.end_x {
            return None;
        }

        let point = if self.is_steep {
            (self.y, self.x)
        } else {
            (self.x, self.y)
        };

        self.x += 1;
        self.error -= self.dy;
        if self.error < 0.0 {
            self.y += self.y_step;
            self.error += self.dx;
        }

        Some(point)
    }
}

fn draw_antialiased_line_segment_mut<B>(
    image: &mut RgbaImage,
    start: (i32, i32),
    end: (i32, i32),
    color: Rgba<u8>,
    blend: B,
) where
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    let (mut x0, mut y0) = (start.0, start.1);
    let (mut x1, mut y1) = (end.0, end.1);

    let is_steep = (y1 - y0).abs() > (x1 - x0).abs();
    if is_steep {
        if y0 > y1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        plot_wu_line(image, (y0, x0), (y1, x1), color, |x, y| (y, x), blend);
    } else {
        if x0 > x1 {
            swap(&mut x0, &mut x1);
            swap(&mut y0, &mut y1);
        }
        plot_wu_line(image, (x0, y0), (x1, y1), color, |x, y| (x, y), blend);
    }
}

fn plot_wu_line<T, B>(
    image: &mut RgbaImage,
    start: (i32, i32),
    end: (i32, i32),
    color: Rgba<u8>,
    transform: T,
    blend: B,
) where
    T: Fn(i32, i32) -> (i32, i32),
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    if dx == 0 {
        plot_antialiased_pixel(image, transform(start.0, start.1), color, 1.0, &blend);
        return;
    }
    let gradient = dy as f32 / dx as f32;
    let mut fy = start.1 as f32;

    for x in start.0..=end.0 {
        plot_antialiased_pixel(
            image,
            transform(x, fy as i32),
            color,
            1.0 - fy.fract(),
            &blend,
        );
        plot_antialiased_pixel(
            image,
            transform(x, fy as i32 + 1),
            color,
            fy.fract(),
            &blend,
        );
        fy += gradient;
    }
}

fn plot_antialiased_pixel<B>(
    image: &mut RgbaImage,
    (x, y): (i32, i32),
    color: Rgba<u8>,
    weight: f32,
    blend: &B,
) where
    B: Fn(Rgba<u8>, Rgba<u8>, f32) -> Rgba<u8>,
{
    if in_bounds(image, x, y) {
        let original = *image.get_pixel(x as u32, y as u32);
        image.put_pixel(x as u32, y as u32, blend(color, original, weight));
    }
}

fn draw_filled_ellipse_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
    color: Rgba<u8>,
) {
    if width_radius == height_radius {
        draw_filled_circle_mut(image, center, width_radius, color);
        return;
    }

    draw_ellipse(
        |image, x0, y0, x, y| {
            draw_line_segment_mut(
                image,
                ((x0 - x) as f32, (y0 + y) as f32),
                ((x0 + x) as f32, (y0 + y) as f32),
                color,
            );
            draw_line_segment_mut(
                image,
                ((x0 - x) as f32, (y0 - y) as f32),
                ((x0 + x) as f32, (y0 - y) as f32),
                color,
            );
        },
        image,
        center,
        width_radius,
        height_radius,
    );
}

fn draw_ellipse<F>(
    mut render: F,
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
) where
    F: FnMut(&mut RgbaImage, i32, i32, i32, i32),
{
    let (x0, y0) = center;
    let w2 = (width_radius as f64).powi(2);
    let h2 = (height_radius as f64).powi(2);
    let mut x = 0;
    let mut y = height_radius;
    let mut px = 0.0;
    let mut py = 2.0 * w2 * y as f64;

    render(image, x0, y0, x, y);

    let mut p = h2 - (w2 * height_radius as f64) + (0.25 * w2);
    while px < py {
        x += 1;
        px += 2.0 * h2;
        if p < 0.0 {
            p += h2 + px;
        } else {
            y -= 1;
            py += -2.0 * w2;
            p += h2 + px - py;
        }
        render(image, x0, y0, x, y);
    }

    p = h2 * (x as f64 + 0.5).powi(2) + w2 * ((y - 1) as f64).powi(2) - w2 * h2;
    while y > 0 {
        y -= 1;
        py += -2.0 * w2;
        if p > 0.0 {
            p += w2 - py;
        } else {
            x += 1;
            px += 2.0 * h2;
            p += w2 - py + px;
        }
        render(image, x0, y0, x, y);
    }
}

fn draw_hollow_circle_mut(image: &mut RgbaImage, center: (i32, i32), radius: i32, color: Rgba<u8>) {
    let mut x = 0;
    let mut y = radius;
    let mut p = 1 - radius;
    let (x0, y0) = center;

    while x <= y {
        draw_if_in_bounds(image, x0 + x, y0 + y, color);
        draw_if_in_bounds(image, x0 + y, y0 + x, color);
        draw_if_in_bounds(image, x0 - y, y0 + x, color);
        draw_if_in_bounds(image, x0 - x, y0 + y, color);
        draw_if_in_bounds(image, x0 - x, y0 - y, color);
        draw_if_in_bounds(image, x0 - y, y0 - x, color);
        draw_if_in_bounds(image, x0 + y, y0 - x, color);
        draw_if_in_bounds(image, x0 + x, y0 - y, color);

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

fn draw_filled_circle_mut(image: &mut RgbaImage, center: (i32, i32), radius: i32, color: Rgba<u8>) {
    let mut x = 0;
    let mut y = radius;
    let mut p = 1 - radius;
    let (x0, y0) = center;

    while x <= y {
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 + y) as f32),
            ((x0 + x) as f32, (y0 + y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 + x) as f32),
            ((x0 + y) as f32, (y0 + x) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - x) as f32, (y0 - y) as f32),
            ((x0 + x) as f32, (y0 - y) as f32),
            color,
        );
        draw_line_segment_mut(
            image,
            ((x0 - y) as f32, (y0 - x) as f32),
            ((x0 + y) as f32, (y0 - x) as f32),
            color,
        );

        x += 1;
        if p < 0 {
            p += 2 * x + 1;
        } else {
            y -= 1;
            p += 2 * (x - y) + 1;
        }
    }
}

fn draw_polygon_mut(image: &mut RgbaImage, poly: &[Point<i32>], color: Rgba<u8>) {
    if poly.is_empty() {
        return;
    }

    let mut y_min = i32::MAX;
    let mut y_max = i32::MIN;
    for point in poly {
        y_min = y_min.min(point.y);
        y_max = y_max.max(point.y);
    }

    y_min = 0.max(y_min.min(image.height() as i32 - 1));
    y_max = 0.max(y_max.min(image.height() as i32 - 1));

    let mut closed = poly.to_vec();
    let first = poly[0];
    let last = poly[poly.len() - 1];
    if first != last {
        closed.push(first);
    }

    let edges: Vec<&[Point<i32>]> = closed.windows(2).collect();
    let mut intersections = Vec::new();

    for y in y_min..=y_max {
        for edge in &edges {
            let p0 = edge[0];
            let p1 = edge[1];

            if p0.y <= y && p1.y >= y || p1.y <= y && p0.y >= y {
                if p0.y == p1.y {
                    intersections.push(p0.x);
                    intersections.push(p1.x);
                } else if p0.y == y || p1.y == y {
                    if p1.y > y {
                        intersections.push(p0.x);
                    }
                    if p0.y > y {
                        intersections.push(p1.x);
                    }
                } else {
                    let fraction = (y - p0.y) as f32 / (p1.y - p0.y) as f32;
                    let inter = p0.x as f32 + fraction * (p1.x - p0.x) as f32;
                    intersections.push(inter.round() as i32);
                }
            }
        }

        intersections.sort_unstable();
        for range in intersections.chunks(2) {
            if range.len() < 2 {
                continue;
            }
            let mut from = range[0].min(image.width() as i32);
            let mut to = range[1].min(image.width() as i32 - 1);
            if from < image.width() as i32 && to >= 0 {
                from = from.max(0);
                to = to.max(0);
                for x in from..=to {
                    image.put_pixel(x as u32, y as u32, color);
                }
            }
        }

        intersections.clear();
    }

    for edge in edges {
        draw_line_segment_mut(
            image,
            (edge[0].x as f32, edge[0].y as f32),
            (edge[1].x as f32, edge[1].y as f32),
            color,
        );
    }
}

fn interpolate(left: Rgba<u8>, right: Rgba<u8>, left_weight: f32) -> Rgba<u8> {
    let right_weight = 1.0 - left_weight;
    Rgba([
        weighted_channel_sum(left.0[0], right.0[0], left_weight, right_weight),
        weighted_channel_sum(left.0[1], right.0[1], left_weight, right_weight),
        weighted_channel_sum(left.0[2], right.0[2], left_weight, right_weight),
        weighted_channel_sum(left.0[3], right.0[3], left_weight, right_weight),
    ])
}

fn weighted_channel_sum(left: u8, right: u8, left_weight: f32, right_weight: f32) -> u8 {
    let value = left as f32 * left_weight + right as f32 * right_weight;
    if value < u8::MAX as f32 {
        if value > u8::MIN as f32 {
            value as u8
        } else {
            u8::MIN
        }
    } else {
        u8::MAX
    }
}

fn draw_if_in_bounds(image: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if in_bounds(image, x, y) {
        image.put_pixel(x as u32, y as u32, color);
    }
}

fn in_bounds(image: &RgbaImage, x: i32, y: i32) -> bool {
    x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32
}

/// Input parameters for a generated avatar image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpec {
    width: u32,
    height: u32,
    seed: u64,
}

impl AvatarSpec {
    pub const fn new(width: u32, height: u32, seed: u64) -> Result<Self, AvatarSpecError> {
        if Self::dimensions_are_supported(width, height) {
            Ok(Self {
                width,
                height,
                seed,
            })
        } else {
            Err(AvatarSpecError { width, height })
        }
    }

    const fn new_unchecked(width: u32, height: u32, seed: u64) -> Self {
        Self {
            width,
            height,
            seed,
        }
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }

    pub const fn seed(self) -> u64 {
        self.seed
    }

    pub const fn is_supported(self) -> bool {
        Self::dimensions_are_supported(self.width, self.height)
    }

    const fn dimensions_are_supported(width: u32, height: u32) -> bool {
        width >= MIN_AVATAR_DIMENSION
            && height >= MIN_AVATAR_DIMENSION
            && width <= MAX_AVATAR_DIMENSION
            && height <= MAX_AVATAR_DIMENSION
    }

    pub fn validate(self) -> Result<(), AvatarSpecError> {
        if self.is_supported() {
            Ok(())
        } else {
            Err(AvatarSpecError {
                width: self.width,
                height: self.height,
            })
        }
    }
}

impl Default for AvatarSpec {
    fn default() -> Self {
        Self::new_unchecked(256, 256, 1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpecError {
    width: u32,
    height: u32,
}

impl AvatarSpecError {
    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

impl std::fmt::Display for AvatarSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "avatar dimensions must be between {MIN_AVATAR_DIMENSION} and {MAX_AVATAR_DIMENSION} pixels per side, got {}x{}",
            self.width, self.height
        )
    }
}

impl std::error::Error for AvatarSpecError {}

fn validate_image_avatar_spec(spec: AvatarSpec) -> ImageResult<()> {
    spec.validate().map_err(avatar_spec_error_to_image_error)
}

fn avatar_spec_error_to_image_error(_: AvatarSpecError) -> ImageError {
    ImageError::Limits(LimitError::from_kind(LimitErrorKind::DimensionError))
}

fn avatar_identity_error_to_image_error(error: AvatarIdentityError) -> ImageError {
    ImageError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
}

fn avatar_render_error_to_image_error(error: AvatarRenderError) -> ImageError {
    match error {
        AvatarRenderError::Spec(error) => avatar_spec_error_to_image_error(error),
        AvatarRenderError::Identity(error) => avatar_identity_error_to_image_error(error),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvatarIdentityComponent {
    Input,
    Tenant,
    StyleVersion,
}

impl AvatarIdentityComponent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "identity input",
            Self::Tenant => "namespace tenant",
            Self::StyleVersion => "namespace style version",
        }
    }
}

impl std::fmt::Display for AvatarIdentityComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityError {
    component: AvatarIdentityComponent,
    length: usize,
    max: usize,
}

impl AvatarIdentityError {
    pub const fn component(self) -> AvatarIdentityComponent {
        self.component
    }

    pub const fn length(self) -> usize {
        self.length
    }

    pub const fn max(self) -> usize {
        self.max
    }
}

impl std::fmt::Display for AvatarIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} must be at most {} bytes, got {} bytes",
            self.component, self.max, self.length
        )
    }
}

impl std::error::Error for AvatarIdentityError {}

#[derive(Debug)]
pub enum AvatarRenderError {
    Spec(AvatarSpecError),
    Identity(AvatarIdentityError),
}

impl From<AvatarSpecError> for AvatarRenderError {
    fn from(error: AvatarSpecError) -> Self {
        Self::Spec(error)
    }
}

impl From<AvatarIdentityError> for AvatarRenderError {
    fn from(error: AvatarIdentityError) -> Self {
        Self::Identity(error)
    }
}

impl std::fmt::Display for AvatarRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spec(error) => error.fmt(f),
            Self::Identity(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for AvatarRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spec(error) => Some(error),
            Self::Identity(error) => Some(error),
        }
    }
}

/// Identity hash algorithm used to derive an avatar genome.
///
/// SHA-512 is always available and remains the default. Additional algorithms
/// are available only when their Cargo features are enabled.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarHashAlgorithm {
    #[default]
    Sha512,
    #[cfg(feature = "blake3")]
    Blake3,
    #[cfg(feature = "xxh3")]
    Xxh3_128,
}

impl AvatarHashAlgorithm {
    #[cfg(all(feature = "blake3", feature = "xxh3"))]
    pub const ALL: [Self; 3] = [Self::Sha512, Self::Blake3, Self::Xxh3_128];

    #[cfg(all(feature = "blake3", not(feature = "xxh3")))]
    pub const ALL: [Self; 2] = [Self::Sha512, Self::Blake3];

    #[cfg(all(not(feature = "blake3"), feature = "xxh3"))]
    pub const ALL: [Self; 2] = [Self::Sha512, Self::Xxh3_128];

    #[cfg(all(not(feature = "blake3"), not(feature = "xxh3")))]
    pub const ALL: [Self; 1] = [Self::Sha512];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha512 => "sha512",
            #[cfg(feature = "blake3")]
            Self::Blake3 => "blake3",
            #[cfg(feature = "xxh3")]
            Self::Xxh3_128 => "xxh3-128",
        }
    }

    const fn domain_label(self) -> &'static [u8] {
        match self {
            Self::Sha512 => b"sha512",
            #[cfg(feature = "blake3")]
            Self::Blake3 => b"blake3",
            #[cfg(feature = "xxh3")]
            Self::Xxh3_128 => b"xxh3-128",
        }
    }
}

impl FromStr for AvatarHashAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "sha512" | "sha-512" => Ok(Self::Sha512),
            #[cfg(feature = "blake3")]
            "blake3" => Ok(Self::Blake3),
            #[cfg(feature = "xxh3")]
            "xxh3" | "xxh3-128" | "xxh3_128" => Ok(Self::Xxh3_128),
            _ => Err("unsupported avatar hash algorithm"),
        }
    }
}

impl std::fmt::Display for AvatarHashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Options for deriving a stable avatar identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityOptions<'a> {
    namespace: AvatarNamespace<'a>,
    algorithm: AvatarHashAlgorithm,
}

impl<'a> AvatarIdentityOptions<'a> {
    pub const fn new(namespace: AvatarNamespace<'a>, algorithm: AvatarHashAlgorithm) -> Self {
        Self {
            namespace,
            algorithm,
        }
    }

    pub const fn namespace(self) -> AvatarNamespace<'a> {
        self.namespace
    }

    pub const fn algorithm(self) -> AvatarHashAlgorithm {
        self.algorithm
    }
}

impl Default for AvatarIdentityOptions<'static> {
    fn default() -> Self {
        Self::new(AvatarNamespace::DEFAULT, AvatarHashAlgorithm::Sha512)
    }
}

/// A stable avatar identity derived from a fixed 64-byte digest.
///
/// This is intended for Robohash-style uniqueness: the same input always maps
/// to the same visual genome, while different inputs produce different shape
/// and palette parameters with negligible collision risk.
#[derive(Clone, Debug, Eq)]
pub struct AvatarIdentity {
    digest: [u8; 64],
}

impl AvatarIdentity {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_options(
            AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
            input,
        )
    }

    pub fn new_with_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        let input = input.as_ref();
        validate_identity_component(
            AvatarIdentityComponent::Input,
            input.len(),
            MAX_AVATAR_ID_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::Tenant,
            options.namespace.tenant.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::StyleVersion,
            options.namespace.style_version.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        Ok(Self::new_unchecked(options, input))
    }

    fn new_unchecked(options: AvatarIdentityOptions<'_>, input: &[u8]) -> Self {
        Self {
            digest: derive_identity_digest(options, input),
        }
    }

    pub const fn as_digest(&self) -> &[u8; 64] {
        &self.digest
    }

    pub fn rng_seed(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&self.digest[32..64]);
        seed
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

impl Zeroize for AvatarIdentity {
    fn zeroize(&mut self) {
        self.digest.zeroize();
    }
}

impl PartialEq for AvatarIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.digest.ct_eq(&other.digest).into()
    }
}

impl Drop for AvatarIdentity {
    fn drop(&mut self) {
        self.zeroize();
    }
}

fn validate_identity_component(
    component: AvatarIdentityComponent,
    length: usize,
    max: usize,
) -> Result<(), AvatarIdentityError> {
    if length <= max {
        Ok(())
    } else {
        Err(AvatarIdentityError {
            component,
            length,
            max,
        })
    }
}

fn derive_identity_digest(options: AvatarIdentityOptions<'_>, input: &[u8]) -> [u8; 64] {
    let mut preimage = identity_hash_preimage(options, input);
    let digest = match options.algorithm {
        AvatarHashAlgorithm::Sha512 => sha512_digest(&preimage),
        #[cfg(feature = "blake3")]
        AvatarHashAlgorithm::Blake3 => blake3_digest(&preimage),
        #[cfg(feature = "xxh3")]
        AvatarHashAlgorithm::Xxh3_128 => xxh3_128_digest(&preimage),
    };
    preimage.zeroize();
    digest
}

fn identity_hash_preimage(options: AvatarIdentityOptions<'_>, input: &[u8]) -> Vec<u8> {
    let algorithm_overhead = if options.algorithm == AvatarHashAlgorithm::Sha512 {
        0
    } else {
        length_prefixed_component_size(HASH_DOMAIN_ALGORITHM_COMPONENT)
            + length_prefixed_component_size(options.algorithm.domain_label())
    };
    let mut preimage = Vec::with_capacity(
        length_prefixed_component_size(HASH_DOMAIN)
            + algorithm_overhead
            + length_prefixed_component_size(options.namespace.tenant.as_bytes())
            + length_prefixed_component_size(options.namespace.style_version.as_bytes())
            + length_prefixed_component_size(input),
    );

    update_hash_input_component(&mut preimage, HASH_DOMAIN);
    if options.algorithm != AvatarHashAlgorithm::Sha512 {
        update_hash_input_component(&mut preimage, HASH_DOMAIN_ALGORITHM_COMPONENT);
        update_hash_input_component(&mut preimage, options.algorithm.domain_label());
    }
    update_hash_input_component(&mut preimage, options.namespace.tenant.as_bytes());
    update_hash_input_component(&mut preimage, options.namespace.style_version.as_bytes());
    update_hash_input_component(&mut preimage, input);
    preimage
}

const fn length_prefixed_component_size(bytes: &[u8]) -> usize {
    std::mem::size_of::<u64>() + bytes.len()
}

fn update_hash_input_component(preimage: &mut Vec<u8>, bytes: &[u8]) {
    preimage.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    preimage.extend_from_slice(bytes);
}

fn sha512_digest(preimage: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(preimage);
    hasher.finalize().into()
}

#[cfg(feature = "blake3")]
fn blake3_digest(preimage: &[u8]) -> [u8; 64] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(preimage);
    let mut digest = [0u8; 64];
    hasher.finalize_xof().fill(&mut digest);
    digest
}

#[cfg(feature = "xxh3")]
fn xxh3_128_digest(preimage: &[u8]) -> [u8; 64] {
    let mut digest = [0u8; 64];
    for chunk in 0..4 {
        let mut chunk_input = Vec::with_capacity(
            preimage.len()
                + length_prefixed_component_size(HASH_XOF_CHUNK_COMPONENT)
                + length_prefixed_component_size(&[chunk as u8]),
        );
        chunk_input.extend_from_slice(preimage);
        update_hash_input_component(&mut chunk_input, HASH_XOF_CHUNK_COMPONENT);
        update_hash_input_component(&mut chunk_input, &[chunk as u8]);
        let mut chunk_digest = xxhash_rust::xxh3::xxh3_128(&chunk_input).to_le_bytes();
        let offset = chunk * chunk_digest.len();
        digest[offset..offset + chunk_digest.len()].copy_from_slice(&chunk_digest);
        chunk_digest.zeroize();
        chunk_input.zeroize();
    }
    digest
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarNamespace<'a> {
    tenant: &'a str,
    style_version: &'a str,
}

impl<'a> AvatarNamespace<'a> {
    pub const DEFAULT: Self = Self {
        tenant: "public",
        style_version: "v2",
    };

    pub fn new(tenant: &'a str, style_version: &'a str) -> Result<Self, AvatarIdentityError> {
        validate_identity_component(
            AvatarIdentityComponent::Tenant,
            tenant.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::StyleVersion,
            style_version.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        Ok(Self {
            tenant,
            style_version,
        })
    }

    pub const fn tenant(self) -> &'a str {
        self.tenant
    }

    pub const fn style_version(self) -> &'a str {
        self.style_version
    }
}

impl Default for AvatarNamespace<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
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
    #[default]
    WebP,
    Png,
    Jpeg,
    Gif,
}

impl AvatarOutputFormat {
    pub const ALL: [Self; 4] = [Self::WebP, Self::Png, Self::Jpeg, Self::Gif];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
        }
    }
}

impl FromStr for AvatarOutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "webp" => Ok(Self::WebP),
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "gif" => Ok(Self::Gif),
            _ => Err("unsupported avatar output format"),
        }
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
}

impl AvatarKind {
    pub const ALL: [Self; 23] = [
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
            "planet" => Ok(Self::Planet),
            "rocket" => Ok(Self::Rocket),
            "mushroom" => Ok(Self::Mushroom),
            "cactus" => Ok(Self::Cactus),
            "frog" => Ok(Self::Frog),
            "panda" => Ok(Self::Panda),
            "cupcake" => Ok(Self::Cupcake),
            "pizza" => Ok(Self::Pizza),
            "icecream" | "ice-cream" | "ice_cream" => Ok(Self::Icecream),
            "octopus" => Ok(Self::Octopus),
            "knight" => Ok(Self::Knight),
            _ => Err("unsupported avatar kind"),
        }
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
}

impl AvatarBackground {
    pub const ALL: [Self; 6] = [
        Self::Themed,
        Self::White,
        Self::Black,
        Self::Dark,
        Self::Light,
        Self::Transparent,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Themed => "themed",
            Self::White => "white",
            Self::Black => "black",
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Transparent => "transparent",
        }
    }
}

impl FromStr for AvatarBackground {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "themed" => Ok(Self::Themed),
            "white" => Ok(Self::White),
            "black" => Ok(Self::Black),
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            "transparent" => Ok(Self::Transparent),
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
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_cat_avatar(spec)
    }
}

/// Cat-face avatar renderer driven by a stable identity digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedCatAvatar {
    identity: AvatarIdentity,
}

impl HashedCatAvatar {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
        })
    }

    pub fn identity(&self) -> &AvatarIdentity {
        &self.identity
    }
}

impl AvatarRenderer for HashedCatAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_cat_avatar_for_identity(spec, &self.identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedDogAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedDogAvatar {
    pub fn new<T: AsRef<[u8]>>(
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
            background,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
            background,
        })
    }
}

impl AvatarRenderer for HashedDogAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_dog_avatar_for_identity(spec, &self.identity, self.background)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedRobotAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedRobotAvatar {
    pub fn new<T: AsRef<[u8]>>(
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
            background,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
            background,
        })
    }
}

impl AvatarRenderer for HashedRobotAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_robot_avatar_for_identity(spec, &self.identity, self.background)
    }
}

/// Render and encode an avatar into memory.
pub fn encode_avatar<R: AvatarRenderer>(
    renderer: &R,
    spec: AvatarSpec,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    validate_image_avatar_spec(spec)?;
    let image = renderer
        .render(spec)
        .map_err(avatar_spec_error_to_image_error)?;
    encode_rgba_image(&image, format)
}

/// Render and encode a cat avatar into memory.
pub fn encode_cat_avatar(spec: AvatarSpec, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    encode_avatar(&CatAvatar, spec, format)
}

/// Render and encode a cat avatar for a stable identity string.
pub fn encode_cat_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    let renderer = HashedCatAvatar::new(id).map_err(avatar_identity_error_to_image_error)?;
    encode_avatar(&renderer, spec, format)
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
    encode_avatar_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
        id,
        format,
        options,
    )
}

pub fn encode_avatar_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    let image = render_avatar_with_identity_options(spec, identity_options, id, options)
        .map_err(avatar_render_error_to_image_error)?;
    encode_rgba_image(&image, format)
}

/// Render an avatar image directly without encoding it.
pub fn render_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
        id,
        options,
    )
}

pub fn render_avatar_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    spec.validate()?;
    let identity = AvatarIdentity::new_with_options(identity_options, id)?;
    let image = match options.kind {
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
        AvatarKind::Planet => {
            render_planet_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Rocket => {
            render_rocket_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Mushroom => {
            render_mushroom_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Cactus => {
            render_cactus_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Frog => render_frog_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Panda => render_panda_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Cupcake => {
            render_cupcake_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Pizza => render_pizza_avatar_for_identity(spec, &identity, options.background),
        AvatarKind::Icecream => {
            render_icecream_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Octopus => {
            render_octopus_avatar_for_identity(spec, &identity, options.background)
        }
        AvatarKind::Knight => {
            render_knight_avatar_for_identity(spec, &identity, options.background)
        }
    }?;
    Ok(image)
}

/// Render an avatar as a compact SVG string.
pub fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_svg_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
        id,
        options,
    )
}

pub fn render_avatar_svg_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    spec.validate()?;
    let identity = AvatarIdentity::new_with_options(identity_options, id)?;
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
            AvatarKind::Planet => hsl_to_color(215.0 + identity.unit_f32(14) * 90.0, 0.24, 0.91),
            AvatarKind::Rocket => hsl_to_color(205.0 + identity.unit_f32(15) * 70.0, 0.22, 0.92),
            AvatarKind::Mushroom => hsl_to_color(18.0 + identity.unit_f32(16) * 35.0, 0.20, 0.93),
            AvatarKind::Cactus => hsl_to_color(80.0 + identity.unit_f32(17) * 55.0, 0.20, 0.92),
            AvatarKind::Frog => hsl_to_color(95.0 + identity.unit_f32(18) * 65.0, 0.23, 0.92),
            AvatarKind::Panda => hsl_to_color(200.0 + identity.unit_f32(19) * 45.0, 0.08, 0.94),
            AvatarKind::Cupcake => hsl_to_color(320.0 + identity.unit_f32(20) * 45.0, 0.22, 0.94),
            AvatarKind::Pizza => hsl_to_color(36.0 + identity.unit_f32(21) * 30.0, 0.24, 0.93),
            AvatarKind::Icecream => hsl_to_color(190.0 + identity.unit_f32(22) * 95.0, 0.18, 0.94),
            AvatarKind::Octopus => hsl_to_color(185.0 + identity.unit_f32(23) * 70.0, 0.22, 0.92),
            AvatarKind::Knight => hsl_to_color(215.0 + identity.unit_f32(24) * 30.0, 0.12, 0.92),
        },
        AvatarBackground::White => Color::rgb(255, 255, 255),
        AvatarBackground::Black => Color::rgb(0, 0, 0),
        AvatarBackground::Dark => Color::rgb(17, 24, 39),
        AvatarBackground::Light => Color::rgb(248, 250, 247),
        AvatarBackground::Transparent => Color::rgba(255, 255, 255, 0),
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
        AvatarKind::Planet => render_planet_svg(spec, &identity),
        AvatarKind::Rocket => render_rocket_svg(spec, &identity),
        AvatarKind::Mushroom => render_mushroom_svg(spec, &identity),
        AvatarKind::Cactus => render_cactus_svg(spec, &identity),
        AvatarKind::Frog => render_frog_svg(spec, &identity),
        AvatarKind::Panda => render_panda_svg(spec, &identity),
        AvatarKind::Cupcake => render_cupcake_svg(spec, &identity),
        AvatarKind::Pizza => render_pizza_svg(spec, &identity),
        AvatarKind::Icecream => render_icecream_svg(spec, &identity),
        AvatarKind::Octopus => render_octopus_svg(spec, &identity),
        AvatarKind::Knight => render_knight_svg(spec, &identity),
    };

    let background = match options.background {
        AvatarBackground::Transparent => String::new(),
        AvatarBackground::Themed
        | AvatarBackground::White
        | AvatarBackground::Black
        | AvatarBackground::Dark
        | AvatarBackground::Light => {
            format!(
                r#"<rect width="100%" height="100%" fill="{}"/>"#,
                color_hex(bg)
            )
        }
    };

    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label} avatar">{background}{body}</svg>"#,
        w = spec.width,
        h = spec.height,
        label = options.kind.as_str(),
        background = background,
        body = body,
    )
    .replace('\n', "")
    .replace("  ", ""))
}

/// Render a cat face avatar into an RGBA image.
pub fn render_cat_avatar(spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let seed = spec.seed.to_le_bytes();
    let identity = AvatarIdentity::new_unchecked(AvatarIdentityOptions::default(), &seed);
    Ok(render_cat_avatar_with_identity(
        spec,
        &identity,
        AvatarBackground::Themed,
    ))
}

/// Render a cat face avatar from a stable identity digest.
pub fn render_cat_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    Ok(render_cat_avatar_with_identity(
        spec,
        identity,
        AvatarBackground::Themed,
    ))
}

pub fn render_cat_avatar_for_identity_with_background(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    Ok(render_cat_avatar_with_identity(spec, identity, background))
}

fn render_cat_avatar_with_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let mut rng_seed = identity.rng_seed();
    for (index, byte) in spec.seed.to_le_bytes().iter().enumerate() {
        rng_seed[index] ^= *byte;
    }
    let mut rng = StdRng::from_seed(rng_seed);
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
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

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

    if identity.byte(14).is_multiple_of(2) {
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

    if !identity.byte(16).is_multiple_of(3) {
        draw_filled_ellipse_mut(
            &mut image,
            (center_x, nose_y + head_ry / 4),
            head_rx / 10,
            head_ry / 7,
            tongue.into(),
        );
    }

    Ok(image)
}

pub fn render_robot_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

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
        if identity.byte(8).is_multiple_of(2) {
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
    Ok(image)
}

pub fn render_fox_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

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
    Ok(image)
}

pub fn render_alien_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

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
    if identity.byte(6).is_multiple_of(2) {
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
    Ok(image)
}

pub fn render_monster_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

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
        0 => draw_filled_ellipse_mut(
            &mut image,
            (center_x, center_y),
            head_rx,
            head_ry,
            skin.into(),
        ),
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
                Rect::at(center_x - head_rx, center_y - head_ry)
                    .of_size((head_rx * 2) as u32, (head_ry * 2) as u32),
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
                Point::new(
                    center_x - head_rx / 3 - horn_width,
                    center_y - head_ry - horn_height,
                ),
                Point::new(center_x - head_rx / 8, center_y - head_ry / 2),
            ],
            shade.into(),
        );
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x + head_rx / 2, center_y - head_ry),
                Point::new(
                    center_x + head_rx / 3 + horn_width,
                    center_y - head_ry - horn_height,
                ),
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
        let radius =
            (head_rx as f32 * (0.05 + ((index + 1) as f32 / spot_count as f32) * 0.06)) as i32;
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
                Rect::at(center_x - head_rx / 3, mouth_y - head_ry / 10)
                    .of_size((head_rx * 2 / 3) as u32, (head_ry / 5).max(1) as u32),
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
            draw_filled_circle_mut(
                &mut image,
                (end_x, end_y),
                (head_rx / 18).max(2),
                shade.into(),
            );
        }
    }

    Ok(image)
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
            for tooth_x in [
                layout.center_x - layout.head_rx / 7,
                layout.center_x + layout.head_rx / 7,
            ] {
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
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.19 + identity.unit_f32(3) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.21 + identity.unit_f32(4) * 0.08)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.52 + identity.unit_f32(5) * 0.06)) as i32,
        head_rx,
        head_ry,
    };
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(
            background,
            hsl_to_color(
                205.0 + identity.unit_f32(0) * 55.0,
                0.16 + identity.unit_f32(6) * 0.08,
                0.95,
            ),
        )
        .into(),
    );
    let body = hsl_to_color(
        190.0 + identity.unit_f32(1) * 55.0,
        0.10 + identity.unit_f32(7) * 0.10,
        0.94 + identity.unit_f32(8) * 0.04,
    );
    let shade = hsl_to_color(
        210.0 + identity.unit_f32(2) * 34.0,
        0.16 + identity.unit_f32(9) * 0.12,
        0.70 + identity.unit_f32(10) * 0.12,
    );
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
    if background == AvatarBackground::Themed && identity.byte(11).is_multiple_of(2) {
        draw_filled_ellipse_mut(
            &mut image,
            (
                layout.center_x - layout.head_rx / 2,
                layout.center_y + layout.head_ry / 3,
            ),
            (layout.head_rx as f32 * 0.42) as i32,
            (layout.head_ry as f32 * 0.18) as i32,
            Color::rgba(shade.0[0], shade.0[1], shade.0[2], 80).into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        body.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(layout.center_x - layout.head_rx, layout.center_y).of_size(
            (layout.head_rx * 2) as u32,
            (layout.head_ry + layout.head_ry / 2) as u32,
        ),
        body.into(),
    );
    let scallops = 3 + (identity.byte(12) % 3) as i32;
    for index in 0..scallops {
        let denominator = (scallops - 1).max(1);
        let x = layout.center_x - layout.head_rx + index * (layout.head_rx * 2 / denominator);
        let radius = ((layout.head_rx as f32)
            * (0.18 + identity.unit_f32(13 + index as usize) * 0.12)) as i32;
        draw_filled_circle_mut(
            &mut image,
            (x, layout.center_y + layout.head_ry + layout.head_ry / 2),
            radius.max(3),
            body.into(),
        );
    }
    if identity.byte(16).is_multiple_of(2) {
        for side in [-1, 1] {
            draw_filled_ellipse_mut(
                &mut image,
                (
                    layout.center_x + side * (layout.head_rx + layout.head_rx / 5),
                    layout.center_y + layout.head_ry / 4,
                ),
                (layout.head_rx as f32 * (0.20 + identity.unit_f32(17) * 0.08)) as i32,
                (layout.head_ry as f32 * 0.16) as i32,
                Color::rgba(body.0[0], body.0[1], body.0[2], 210).into(),
            );
        }
    }
    draw_creature_eyes(
        &mut image,
        layout,
        if identity.byte(18).is_multiple_of(5) {
            3
        } else {
            2
        },
        if identity.byte(19).is_multiple_of(2) {
            CreatureEyeStyle::Tall
        } else {
            CreatureEyeStyle::Hollow
        },
        Color::rgb(42, 48, 68),
        Color::rgb(42, 48, 68),
    );
    let mouth_style = match identity.byte(20) % 3 {
        0 => CreatureMouthStyle::Smile,
        1 => CreatureMouthStyle::Fang,
        _ => CreatureMouthStyle::Flat,
    };
    draw_creature_mouth(&mut image, layout, mouth_style, shade);
    Ok(image)
}

pub fn render_slime_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.20 + identity.unit_f32(6) * 0.10)) as i32;
    let head_ry = (height as f32 * (0.16 + identity.unit_f32(7) * 0.08)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.56 + identity.unit_f32(8) * 0.08)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(110.0 + identity.unit_f32(3) * 80.0, 0.18, 0.93);
    let slime = hsl_to_color(
        70.0 + identity.unit_f32(4) * 130.0,
        0.44 + identity.unit_f32(9) * 0.22,
        0.46 + identity.unit_f32(10) * 0.18,
    );
    let dark = hsl_to_color(
        95.0 + identity.unit_f32(5) * 80.0,
        0.34 + identity.unit_f32(11) * 0.18,
        0.25 + identity.unit_f32(12) * 0.14,
    );
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
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
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        slime.into(),
    );
    let drip_count = 2 + (identity.byte(13) % 4) as i32;
    for index in 0..drip_count {
        let spacing = (layout.head_rx * 2 / drip_count.max(1)).max(1);
        let drip_w =
            (layout.head_rx as f32 * (0.18 + identity.unit_f32(14 + index as usize) * 0.12)) as i32;
        let drip_x = layout.center_x - layout.head_rx
            + index * spacing
            + (identity.byte(18 + index as usize) as i32 % spacing.max(1) / 3);
        let drip_h =
            (layout.head_ry as f32 * (0.35 + identity.unit_f32(22 + index as usize) * 0.55)) as i32;
        draw_filled_rect_mut(
            &mut image,
            Rect::at(drip_x, layout.center_y).of_size(drip_w.max(2) as u32, drip_h.max(2) as u32),
            slime.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (drip_x + drip_w / 2, layout.center_y + drip_h),
            layout.head_rx / 7,
            slime.into(),
        );
    }
    let bubble_count = 3 + (identity.byte(27) % 5) as i32;
    for bubble in 0..bubble_count {
        let bx = layout.center_x - layout.head_rx
            + (identity.byte(28 + bubble as usize) as i32 % (layout.head_rx * 2).max(1));
        let by = layout.center_y - layout.head_ry / 2
            + (identity.byte(35 + bubble as usize) as i32 % layout.head_ry.max(1));
        draw_filled_circle_mut(
            &mut image,
            (bx, by),
            ((layout.head_rx as f32) * (0.05 + identity.unit_f32(42 + bubble as usize) * 0.07))
                as i32,
            Color::rgba(255, 255, 255, 90).into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        1 + (identity.byte(49) % 3) as usize,
        if identity.byte(50).is_multiple_of(2) {
            CreatureEyeStyle::Round
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(248, 255, 236),
        Color::rgb(32, 48, 24),
    );
    let mouth_style = match identity.byte(51) % 3 {
        0 => CreatureMouthStyle::Flat,
        1 => CreatureMouthStyle::Smile,
        _ => CreatureMouthStyle::Fang,
    };
    draw_creature_mouth(&mut image, layout, mouth_style, dark);
    Ok(image)
}

pub fn render_bird_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
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
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        wing,
        0.24,
        background,
    );
    draw_filled_circle_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        plumage.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (
            layout.center_x - layout.head_rx / 2,
            layout.center_y + layout.head_ry / 6,
        ),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (
            layout.center_x + layout.head_rx / 2,
            layout.center_y + layout.head_ry / 6,
        ),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(
                layout.center_x + layout.head_rx / 2,
                layout.center_y + layout.head_ry / 6,
            ),
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
                Point::new(
                    fx + layout.head_rx / 10,
                    layout.center_y - layout.head_ry - layout.head_ry / 2,
                ),
                Point::new(
                    fx + layout.head_rx / 5,
                    layout.center_y - layout.head_ry / 2,
                ),
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
    Ok(image)
}

pub fn render_wizard_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.16 + identity.unit_f32(15) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.16 + identity.unit_f32(16) * 0.06)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.57 + identity.unit_f32(17) * 0.07)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(
        220.0 + identity.unit_f32(10) * 85.0,
        0.20 + identity.unit_f32(18) * 0.12,
        0.90 + identity.unit_f32(19) * 0.04,
    );
    let hat = hsl_to_color(
        210.0 + identity.unit_f32(11) * 110.0,
        0.34 + identity.unit_f32(20) * 0.22,
        0.28 + identity.unit_f32(21) * 0.16,
    );
    let hat_band = hsl_to_color(
        24.0 + identity.unit_f32(12) * 160.0,
        0.62 + identity.unit_f32(22) * 0.24,
        0.48 + identity.unit_f32(23) * 0.18,
    );
    let skin = hsl_to_color(
        18.0 + identity.unit_f32(13) * 28.0,
        0.22 + identity.unit_f32(24) * 0.20,
        0.74 + identity.unit_f32(25) * 0.12,
    );
    let beard = hsl_to_color(
        35.0 + identity.unit_f32(14) * 45.0,
        0.06 + identity.unit_f32(26) * 0.12,
        0.80 + identity.unit_f32(27) * 0.16,
    );
    let hat_width = (layout.head_rx as f32 * (1.0 + identity.unit_f32(28) * 0.55)) as i32;
    let hat_height = (layout.head_ry as f32 * (1.7 + identity.unit_f32(29) * 0.75)) as i32;
    let tip_shift = (layout.head_rx as f32 * (identity.unit_f32(30) - 0.5) * 0.8) as i32;
    let brim_width = (layout.head_rx as f32 * (2.45 + identity.unit_f32(31) * 0.75)) as i32;
    let brim_height = (layout.head_ry as f32 * (0.22 + identity.unit_f32(32) * 0.16)) as i32;
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        hat_band,
        0.20,
        background,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(
                layout.center_x - hat_width,
                layout.center_y - layout.head_ry / 2,
            ),
            Point::new(
                layout.center_x + hat_width,
                layout.center_y - layout.head_ry / 2,
            ),
            Point::new(
                layout.center_x + tip_shift,
                layout.center_y - layout.head_ry / 2 - hat_height,
            ),
        ],
        hat.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(
            layout.center_x - brim_width / 2,
            layout.center_y - layout.head_ry / 2,
        )
        .of_size(brim_width.max(2) as u32, brim_height.max(2) as u32),
        hat_band.into(),
    );
    let star_count = 1 + (identity.byte(33) % 4) as i32;
    for star in 0..star_count {
        let sx = layout.center_x - hat_width / 2
            + (identity.byte(34 + star as usize) as i32 % hat_width.max(1));
        let sy = layout.center_y - layout.head_ry / 2 - hat_height / 2
            + (identity.byte(39 + star as usize) as i32 % (hat_height / 2).max(1));
        draw_filled_circle_mut(
            &mut image,
            (sx, sy),
            (layout.head_rx / 12).max(2),
            Color::rgba(hat_band.0[0], hat_band.0[1], hat_band.0[2], 210).into(),
        );
    }
    draw_filled_circle_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        skin.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(
                layout.center_x
                    - layout.head_rx / 2
                    - (identity.byte(44) as i32 % layout.head_rx.max(1)) / 6,
                layout.center_y + layout.head_ry / 3,
            ),
            Point::new(
                layout.center_x
                    + layout.head_rx / 2
                    + (identity.byte(45) as i32 % layout.head_rx.max(1)) / 6,
                layout.center_y + layout.head_ry / 3,
            ),
            Point::new(
                layout.center_x + (identity.unit_f32(46) * layout.head_rx as f32 * 0.4) as i32
                    - layout.head_rx / 5,
                layout.center_y
                    + layout.head_ry
                    + (layout.head_ry as f32 * (0.35 + identity.unit_f32(47) * 0.55)) as i32,
            ),
        ],
        beard.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        if identity.byte(48).is_multiple_of(7) {
            1
        } else {
            2
        },
        if identity.byte(49).is_multiple_of(2) {
            CreatureEyeStyle::Round
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(255, 255, 255),
        Color::rgb(36, 30, 52),
    );
    draw_creature_mouth(
        &mut image,
        layout,
        CreatureMouthStyle::Smile,
        Color::rgb(86, 64, 58),
    );
    draw_filled_circle_mut(
        &mut image,
        (
            layout.center_x + tip_shift + layout.head_rx / 2,
            layout.center_y - layout.head_ry / 2 - hat_height,
        ),
        (layout.head_rx / 6).max(3),
        hat_band.into(),
    );
    Ok(image)
}

pub fn render_skull_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.18 + identity.unit_f32(17) * 0.07)) as i32;
    let head_ry = (height as f32 * (0.18 + identity.unit_f32(18) * 0.07)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.51 + identity.unit_f32(19) * 0.07)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(
        195.0 + identity.unit_f32(15) * 55.0,
        0.06 + identity.unit_f32(20) * 0.08,
        0.92 + identity.unit_f32(21) * 0.04,
    );
    let bone = hsl_to_color(
        28.0 + identity.unit_f32(16) * 34.0,
        0.08 + identity.unit_f32(22) * 0.10,
        0.82 + identity.unit_f32(23) * 0.12,
    );
    let crack = hsl_to_color(
        20.0 + identity.unit_f32(24) * 40.0,
        0.06,
        0.22 + identity.unit_f32(25) * 0.12,
    );
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        crack,
        0.16,
        background,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        bone.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(
            layout.center_x - layout.head_rx / 2,
            layout.center_y + layout.head_ry / 2,
        )
        .of_size(
            (layout.head_rx as f32 * (0.82 + identity.unit_f32(26) * 0.34)) as u32,
            (layout.head_ry as f32 * (0.34 + identity.unit_f32(27) * 0.28)) as u32,
        ),
        bone.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        if identity.byte(28).is_multiple_of(2) {
            CreatureEyeStyle::Hollow
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(44, 42, 44),
        Color::rgb(44, 42, 44),
    );
    let nose_half_width = (layout.head_rx as f32 * (0.08 + identity.unit_f32(29) * 0.08)) as i32;
    let nose_height = (layout.head_ry as f32 * (0.12 + identity.unit_f32(30) * 0.12)) as i32;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(
                layout.center_x - nose_half_width,
                layout.center_y + nose_height,
            ),
            Point::new(
                layout.center_x + nose_half_width,
                layout.center_y + nose_height,
            ),
        ],
        crack.into(),
    );
    draw_creature_mouth(
        &mut image,
        layout,
        if identity.byte(31).is_multiple_of(2) {
            CreatureMouthStyle::Flat
        } else {
            CreatureMouthStyle::Smile
        },
        crack,
    );
    let tooth_count = 3 + (identity.byte(32) % 4) as i32;
    for tooth in 0..tooth_count {
        let x = layout.center_x - layout.head_rx / 3
            + tooth * (layout.head_rx * 2 / tooth_count.max(1));
        draw_line_segment_mut(
            &mut image,
            (x as f32, (layout.center_y + layout.head_ry / 2) as f32),
            (x as f32, (layout.center_y + layout.head_ry) as f32),
            crack.into(),
        );
    }
    let crack_count = 1 + (identity.byte(33) % 3) as i32;
    for line in 0..crack_count {
        let start_x = layout.center_x - layout.head_rx / 4
            + (identity.byte(34 + line as usize) as i32 % (layout.head_rx / 2).max(1));
        let start_y = layout.center_y - layout.head_ry / 2
            + (identity.byte(38 + line as usize) as i32 % (layout.head_ry / 2).max(1));
        let end_x = start_x
            + ((identity.unit_f32(42 + line as usize) - 0.5) * layout.head_rx as f32 * 0.45) as i32;
        let end_y = start_y
            + (layout.head_ry as f32 * (0.18 + identity.unit_f32(46 + line as usize) * 0.32))
                as i32;
        draw_line_segment_mut(
            &mut image,
            (start_x as f32, start_y as f32),
            (end_x as f32, end_y as f32),
            crack.into(),
        );
    }
    draw_line_segment_mut(
        &mut image,
        (
            (layout.center_x + layout.head_rx / 4) as f32,
            (layout.center_y - layout.head_ry / 2) as f32,
        ),
        (
            (layout.center_x + layout.head_rx / 8) as f32,
            (layout.center_y - layout.head_ry / 8) as f32,
        ),
        crack.into(),
    );
    Ok(image)
}

/// Render a ringed planet avatar from a stable identity.
pub fn render_planet_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.53 + identity.unit_f32(8) * 0.06)) as i32;
    let bg = hsl_to_color(215.0 + identity.unit_f32(0) * 90.0, 0.24, 0.91);
    let planet = hsl_to_color(identity.unit_f32(1) * 360.0, 0.46, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.42);
    let ring = hsl_to_color(32.0 + identity.unit_f32(3) * 120.0, 0.44, 0.72);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );

    if background == AvatarBackground::Themed {
        let star_count = 3 + (identity.byte(4) % 4) as i32;
        for star in 0..star_count {
            let x = width / 8 + (identity.byte(5 + star as usize) as i32 % (width * 3 / 4).max(1));
            let y =
                height / 8 + (identity.byte(10 + star as usize) as i32 % (height * 3 / 4).max(1));
            draw_filled_circle_mut(
                &mut image,
                (x, y),
                (width as f32 * (0.010 + identity.unit_f32(15 + star as usize) * 0.010)) as i32,
                Color::rgba(255, 255, 255, 170).into(),
            );
        }
    }

    let radius = (width.min(height) as f32 * (0.18 + identity.unit_f32(20) * 0.08)) as i32;
    let ring_rx = (radius as f32 * (1.55 + identity.unit_f32(21) * 0.28)) as i32;
    let ring_ry = (radius as f32 * (0.38 + identity.unit_f32(22) * 0.12)) as i32;
    let bg_fill = background_fill(background, bg);
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        ring_rx,
        ring_ry,
        ring.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        (ring_rx as f32 * 0.84) as i32,
        (ring_ry as f32 * 0.58) as i32,
        bg_fill.into(),
    );
    draw_filled_circle_mut(&mut image, (center_x, center_y), radius, planet.into());
    draw_filled_ellipse_mut(
        &mut image,
        (center_x - radius / 4, center_y - radius / 5),
        radius / 2,
        radius / 5,
        Color::rgba(shade.0[0], shade.0[1], shade.0[2], 120).into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + radius / 4, center_y + radius / 5),
        radius / 2,
        radius / 6,
        Color::rgba(255, 255, 255, 80).into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - ring_rx, center_y - ring_ry / 5)
            .of_size((ring_rx * 2) as u32, (ring_ry / 3).max(1) as u32),
        Color::rgba(ring.0[0], ring.0[1], ring.0[2], 190).into(),
    );
    Ok(image)
}

/// Render a rocket avatar from a stable identity.
pub fn render_rocket_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.52) as i32;
    let bg = hsl_to_color(205.0 + identity.unit_f32(0) * 70.0, 0.22, 0.92);
    let hull = hsl_to_color(200.0 + identity.unit_f32(1) * 50.0, 0.12, 0.88);
    let trim = hsl_to_color(identity.unit_f32(2) * 360.0, 0.58, 0.54);
    let window = hsl_to_color(185.0 + identity.unit_f32(3) * 70.0, 0.54, 0.72);
    let flame = hsl_to_color(20.0 + identity.unit_f32(4) * 30.0, 0.86, 0.58);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        width / 5,
        height / 5,
        trim,
        0.18,
        background,
    );

    let body_w = (width as f32 * (0.18 + identity.unit_f32(5) * 0.05)) as i32;
    let body_h = (height as f32 * (0.42 + identity.unit_f32(6) * 0.08)) as i32;
    let top_y = center_y - body_h / 2;
    let bottom_y = center_y + body_h / 2;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 2, top_y + body_w / 2),
            Point::new(center_x + body_w / 2, top_y + body_w / 2),
            Point::new(center_x, top_y - body_w / 2),
        ],
        trim.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - body_w / 2, top_y + body_w / 2)
            .of_size(body_w as u32, (body_h - body_w / 2).max(1) as u32),
        hull.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y + body_w / 2),
        body_w / 2,
        body_w / 5,
        hull.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 2, bottom_y - body_w / 2),
            Point::new(center_x - body_w, bottom_y + body_w / 3),
            Point::new(center_x - body_w / 2, bottom_y),
        ],
        trim.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x + body_w / 2, bottom_y - body_w / 2),
            Point::new(center_x + body_w, bottom_y + body_w / 3),
            Point::new(center_x + body_w / 2, bottom_y),
        ],
        trim.into(),
    );
    let window_count = 1 + (identity.byte(7) % 2) as i32;
    for window_index in 0..window_count {
        let y = top_y + body_h / 3 + window_index * body_w;
        draw_filled_circle_mut(
            &mut image,
            (center_x, y),
            (body_w as f32 * 0.22) as i32,
            trim.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x, y),
            (body_w as f32 * 0.15) as i32,
            window.into(),
        );
    }
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 4, bottom_y),
            Point::new(center_x + body_w / 4, bottom_y),
            Point::new(
                center_x,
                bottom_y + (height as f32 * (0.10 + identity.unit_f32(8) * 0.08)) as i32,
            ),
        ],
        flame.into(),
    );
    Ok(image)
}

/// Render a mushroom avatar from a stable identity.
pub fn render_mushroom_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let bg = hsl_to_color(18.0 + identity.unit_f32(0) * 35.0, 0.20, 0.93);
    let cap = hsl_to_color(350.0 + identity.unit_f32(1) * 45.0, 0.58, 0.52);
    let stem = hsl_to_color(35.0 + identity.unit_f32(2) * 20.0, 0.24, 0.86);
    let gill = hsl_to_color(26.0 + identity.unit_f32(3) * 20.0, 0.20, 0.70);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let cap_rx = (width as f32 * (0.24 + identity.unit_f32(4) * 0.08)) as i32;
    let cap_ry = (height as f32 * (0.14 + identity.unit_f32(5) * 0.06)) as i32;
    let stem_rx = (width as f32 * (0.09 + identity.unit_f32(6) * 0.04)) as i32;
    let stem_ry = (height as f32 * (0.18 + identity.unit_f32(7) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, cap_rx, cap_ry, gill, 0.24, background,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + stem_ry / 3),
        stem_rx,
        stem_ry,
        stem.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y - cap_ry / 2),
        cap_rx,
        cap_ry,
        cap.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - cap_rx, center_y - cap_ry / 2)
            .of_size((cap_rx * 2) as u32, cap_ry.max(1) as u32),
        cap.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + cap_ry / 3),
        cap_rx,
        cap_ry / 3,
        gill.into(),
    );
    let spot_count = 3 + (identity.byte(8) % 4) as i32;
    for spot in 0..spot_count {
        let sx = center_x - cap_rx / 2 + (identity.byte(9 + spot as usize) as i32 % cap_rx.max(1));
        let sy = center_y - cap_ry + (identity.byte(14 + spot as usize) as i32 % cap_ry.max(1));
        draw_filled_circle_mut(
            &mut image,
            (sx, sy),
            (cap_rx as f32 * (0.06 + identity.unit_f32(19 + spot as usize) * 0.04)) as i32,
            Color::rgba(255, 246, 230, 230).into(),
        );
    }
    Ok(image)
}

/// Render a cactus avatar from a stable identity.
pub fn render_cactus_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let bg = hsl_to_color(80.0 + identity.unit_f32(0) * 55.0, 0.20, 0.92);
    let cactus = hsl_to_color(105.0 + identity.unit_f32(1) * 60.0, 0.42, 0.42);
    let shadow = hsl_to_color(105.0 + identity.unit_f32(2) * 60.0, 0.38, 0.30);
    let flower = hsl_to_color(320.0 + identity.unit_f32(3) * 55.0, 0.58, 0.64);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let body_w = (width as f32 * (0.13 + identity.unit_f32(4) * 0.04)) as i32;
    let body_h = (height as f32 * (0.36 + identity.unit_f32(5) * 0.10)) as i32;
    let top_y = center_y - body_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        body_w * 2,
        body_h / 2,
        shadow,
        0.20,
        background,
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - body_w / 2, top_y).of_size(body_w as u32, body_h as u32),
        cactus.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y),
        body_w / 2,
        body_w / 2,
        cactus.into(),
    );
    for side in [-1, 1] {
        if side == 1 || identity.byte(6).is_multiple_of(2) {
            let arm_y =
                center_y - body_h / 5 + side * (identity.byte(7) as i32 % (body_h / 6).max(1));
            let arm_len = (width as f32
                * (0.11 + identity.unit_f32(8 + side.unsigned_abs() as usize) * 0.05))
                as i32;
            let arm_x = if side < 0 {
                center_x - body_w / 3 - arm_len
            } else {
                center_x + body_w / 3
            };
            let cap_x = if side < 0 {
                center_x - body_w / 3 - arm_len
            } else {
                center_x + body_w / 3 + arm_len
            };
            draw_filled_rect_mut(
                &mut image,
                Rect::at(arm_x, arm_y - body_w / 4)
                    .of_size(arm_len.max(2) as u32, (body_w / 2).max(2) as u32),
                cactus.into(),
            );
            draw_filled_ellipse_mut(
                &mut image,
                (cap_x, arm_y),
                body_w / 4,
                body_w / 4,
                cactus.into(),
            );
        }
    }
    for needle in 0..5 {
        let y = top_y + body_h / 5 + needle * body_h / 7;
        draw_line_segment_mut(
            &mut image,
            ((center_x - body_w / 8) as f32, y as f32),
            ((center_x - body_w / 4) as f32, (y - body_w / 8) as f32),
            Color::rgba(242, 255, 224, 180).into(),
        );
        draw_line_segment_mut(
            &mut image,
            ((center_x + body_w / 8) as f32, (y + body_w / 12) as f32),
            ((center_x + body_w / 4) as f32, y as f32),
            Color::rgba(242, 255, 224, 180).into(),
        );
    }
    if !identity.byte(12).is_multiple_of(3) {
        draw_filled_circle_mut(
            &mut image,
            (center_x, top_y - body_w / 2),
            body_w / 4,
            flower.into(),
        );
    }
    Ok(image)
}

/// Render a frog avatar from a stable identity.
pub fn render_frog_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.57 + identity.unit_f32(6) * 0.04)) as i32;
    let bg = hsl_to_color(95.0 + identity.unit_f32(0) * 65.0, 0.23, 0.92);
    let green = hsl_to_color(92.0 + identity.unit_f32(1) * 72.0, 0.46, 0.54);
    let dark = hsl_to_color(98.0 + identity.unit_f32(2) * 60.0, 0.40, 0.28);
    let cheek = hsl_to_color(335.0 + identity.unit_f32(3) * 24.0, 0.42, 0.76);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(4) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.18 + identity.unit_f32(5) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, dark, 0.20, background,
    );
    let eye_offset = (head_rx as f32 * 0.50) as i32;
    let eye_r = (head_rx as f32 * (0.18 + identity.unit_f32(7) * 0.04)) as i32;
    for side in [-1, 1] {
        draw_filled_circle_mut(
            &mut image,
            (center_x + side * eye_offset, center_y - head_ry),
            eye_r,
            green.into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        green.into(),
    );
    for side in [-1, 1] {
        let ex = center_x + side * eye_offset;
        let ey = center_y - head_ry;
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            (eye_r as f32 * 0.64) as i32,
            Color::rgb(255, 255, 245).into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            (eye_r as f32 * 0.30) as i32,
            dark.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x + side * head_rx / 2, center_y + head_ry / 4),
            head_rx / 9,
            Color::rgba(cheek.0[0], cheek.0[1], cheek.0[2], 150).into(),
        );
    }
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 9,
        center_y + head_ry / 4,
        head_rx / 4,
        dark,
        0.50,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 9,
        center_y + head_ry / 4,
        head_rx / 4,
        dark,
        0.50,
    );
    if identity.byte(8).is_multiple_of(2) {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x - head_rx / 10) as f32,
                (center_y + head_ry / 6) as f32,
            ),
            (
                (center_x - head_rx / 10) as f32,
                (center_y + head_ry / 4) as f32,
            ),
            dark.into(),
        );
        draw_line_segment_mut(
            &mut image,
            (
                (center_x + head_rx / 10) as f32,
                (center_y + head_ry / 6) as f32,
            ),
            (
                (center_x + head_rx / 10) as f32,
                (center_y + head_ry / 4) as f32,
            ),
            dark.into(),
        );
    }
    Ok(image)
}

/// Render a panda avatar from a stable identity.
pub fn render_panda_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.56 + identity.unit_f32(5) * 0.04)) as i32;
    let bg = hsl_to_color(200.0 + identity.unit_f32(0) * 45.0, 0.08, 0.94);
    let white = hsl_to_color(36.0 + identity.unit_f32(1) * 18.0, 0.10, 0.92);
    let black = hsl_to_color(210.0 + identity.unit_f32(2) * 28.0, 0.10, 0.18);
    let blush = hsl_to_color(345.0 + identity.unit_f32(3) * 25.0, 0.32, 0.78);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(6) * 0.05)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(7) * 0.04)) as i32;
    let ear_r = (head_rx as f32 * (0.28 + identity.unit_f32(8) * 0.08)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, black, 0.16, background,
    );
    for side in [-1, 1] {
        draw_filled_circle_mut(
            &mut image,
            (
                center_x + side * head_rx * 3 / 4,
                center_y - head_ry * 3 / 4,
            ),
            ear_r,
            black.into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        white.into(),
    );
    for side in [-1, 1] {
        let patch_x = center_x + side * head_rx / 3;
        let patch_y = center_y - head_ry / 8;
        draw_filled_ellipse_mut(
            &mut image,
            (patch_x, patch_y),
            (head_rx as f32 * (0.20 + identity.unit_f32(9) * 0.05)) as i32,
            (head_ry as f32 * (0.26 + identity.unit_f32(10) * 0.05)) as i32,
            black.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (patch_x, patch_y),
            (head_rx as f32 * 0.055) as i32,
            Color::rgb(248, 248, 244).into(),
        );
        if identity.byte(11).is_multiple_of(2) {
            draw_filled_circle_mut(
                &mut image,
                (center_x + side * head_rx / 2, center_y + head_ry / 4),
                head_rx / 10,
                Color::rgba(blush.0[0], blush.0[1], blush.0[2], 120).into(),
            );
        }
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 5),
        head_rx / 9,
        head_ry / 12,
        black.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        center_y + head_ry / 4,
        head_rx / 6,
        black,
        0.45,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        center_y + head_ry / 4,
        head_rx / 6,
        black,
        0.45,
    );
    Ok(image)
}

/// Render a cupcake avatar from a stable identity.
pub fn render_cupcake_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let bg = hsl_to_color(320.0 + identity.unit_f32(0) * 45.0, 0.22, 0.94);
    let wrapper = hsl_to_color(28.0 + identity.unit_f32(1) * 35.0, 0.46, 0.62);
    let frosting = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.78);
    let shadow = hsl_to_color(330.0 + identity.unit_f32(3) * 45.0, 0.28, 0.58);
    let cherry = hsl_to_color(345.0 + identity.unit_f32(4) * 22.0, 0.66, 0.50);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let cup_w = (width as f32 * (0.26 + identity.unit_f32(5) * 0.07)) as i32;
    let cup_h = (height as f32 * (0.22 + identity.unit_f32(6) * 0.05)) as i32;
    let frosting_rx = (cup_w as f32 * (0.58 + identity.unit_f32(7) * 0.10)) as i32;
    let frosting_ry = (height as f32 * (0.13 + identity.unit_f32(8) * 0.04)) as i32;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        cup_w / 2,
        cup_h,
        shadow,
        0.24,
        background,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - cup_w / 2, center_y),
            Point::new(center_x + cup_w / 2, center_y),
            Point::new(center_x + cup_w / 3, center_y + cup_h),
            Point::new(center_x - cup_w / 3, center_y + cup_h),
        ],
        wrapper.into(),
    );
    for stripe in [-2, 0, 2] {
        let x = center_x + stripe * cup_w / 10;
        draw_line_segment_mut(
            &mut image,
            (x as f32, center_y as f32),
            ((x - stripe * cup_w / 40) as f32, (center_y + cup_h) as f32),
            Color::rgba(255, 244, 214, 115).into(),
        );
    }
    let base_y = center_y - frosting_ry / 2;
    for layer in 0..3 {
        let y = base_y - layer * frosting_ry / 2;
        let rx = (frosting_rx as f32 * (1.0 - layer as f32 * 0.22)) as i32;
        let ry = (frosting_ry as f32 * (0.82 - layer as f32 * 0.10)) as i32;
        draw_filled_ellipse_mut(&mut image, (center_x, y), rx, ry, frosting.into());
    }
    let sprinkle_count = 3 + (identity.byte(9) % 5) as i32;
    for sprinkle in 0..sprinkle_count {
        let sx = center_x - frosting_rx / 2
            + (identity.byte(10 + sprinkle as usize) as i32 % frosting_rx.max(1));
        let sy = base_y - frosting_ry
            + (identity.byte(16 + sprinkle as usize) as i32 % (frosting_ry * 2).max(1));
        let color = hsl_to_color(
            identity.unit_f32(23 + sprinkle as usize) * 360.0,
            0.62,
            0.55,
        );
        draw_filled_rect_mut(
            &mut image,
            Rect::at(sx, sy).of_size((width / 40).max(2) as u32, (height / 80).max(2) as u32),
            color.into(),
        );
    }
    if !identity.byte(30).is_multiple_of(3) {
        draw_filled_circle_mut(
            &mut image,
            (center_x, base_y - frosting_ry),
            (width as f32 * 0.035) as i32,
            cherry.into(),
        );
    }
    Ok(image)
}

/// Render a pizza-slice avatar from a stable identity.
pub fn render_pizza_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.53) as i32;
    let bg = hsl_to_color(36.0 + identity.unit_f32(0) * 30.0, 0.24, 0.93);
    let crust = hsl_to_color(30.0 + identity.unit_f32(1) * 28.0, 0.54, 0.58);
    let cheese = hsl_to_color(45.0 + identity.unit_f32(2) * 16.0, 0.74, 0.70);
    let sauce = hsl_to_color(8.0 + identity.unit_f32(3) * 16.0, 0.62, 0.48);
    let topping = hsl_to_color(350.0 + identity.unit_f32(4) * 22.0, 0.54, 0.46);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let half_w = (width as f32 * (0.22 + identity.unit_f32(5) * 0.06)) as i32;
    let slice_h = (height as f32 * (0.44 + identity.unit_f32(6) * 0.07)) as i32;
    let top_y = center_y - slice_h / 2;
    let tip_y = center_y + slice_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        half_w,
        slice_h / 2,
        sauce,
        0.16,
        background,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - half_w, top_y),
            Point::new(center_x + half_w, top_y),
            Point::new(center_x, tip_y),
        ],
        cheese.into(),
    );
    draw_line_segment_mut(
        &mut image,
        ((center_x - half_w) as f32, top_y as f32),
        ((center_x + half_w) as f32, top_y as f32),
        crust.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y),
        half_w,
        (height as f32 * 0.035) as i32,
        crust.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - half_w + half_w / 6, top_y + slice_h / 5),
            Point::new(center_x + half_w - half_w / 6, top_y + slice_h / 5),
            Point::new(center_x, tip_y - slice_h / 10),
        ],
        Color::rgba(sauce.0[0], sauce.0[1], sauce.0[2], 95).into(),
    );
    let topping_count = 3 + (identity.byte(7) % 4) as i32;
    for item in 0..topping_count {
        let y = top_y + slice_h / 5 + item * slice_h / (topping_count + 2);
        let span = half_w - (y - top_y) * half_w / slice_h;
        let x = center_x - span / 2 + (identity.byte(8 + item as usize) as i32 % span.max(1));
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            (width as f32 * (0.025 + identity.unit_f32(14 + item as usize) * 0.012)) as i32,
            topping.into(),
        );
    }
    Ok(image)
}

/// Render an ice cream cone avatar from a stable identity.
pub fn render_icecream_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.55) as i32;
    let bg = hsl_to_color(190.0 + identity.unit_f32(0) * 95.0, 0.18, 0.94);
    let scoop = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.76);
    let cone = hsl_to_color(32.0 + identity.unit_f32(2) * 22.0, 0.50, 0.64);
    let waffle = hsl_to_color(28.0 + identity.unit_f32(3) * 22.0, 0.42, 0.45);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let scoop_r = (width as f32 * (0.18 + identity.unit_f32(4) * 0.06)) as i32;
    let cone_w = (width as f32 * (0.24 + identity.unit_f32(5) * 0.05)) as i32;
    let cone_h = (height as f32 * (0.32 + identity.unit_f32(6) * 0.06)) as i32;
    let scoop_y = center_y - scoop_r / 2;
    let cone_top_y = scoop_y + scoop_r / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        scoop_r,
        cone_h / 2,
        waffle,
        0.18,
        background,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - cone_w / 2, cone_top_y),
            Point::new(center_x + cone_w / 2, cone_top_y),
            Point::new(center_x, cone_top_y + cone_h),
        ],
        cone.into(),
    );
    for line in [-1, 1] {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x + line * cone_w / 3) as f32,
                (cone_top_y + cone_h / 8) as f32,
            ),
            (center_x as f32, (cone_top_y + cone_h * 3 / 4) as f32),
            waffle.into(),
        );
    }
    draw_filled_circle_mut(&mut image, (center_x, scoop_y), scoop_r, scoop.into());
    if identity.byte(7).is_multiple_of(2) {
        draw_filled_circle_mut(
            &mut image,
            (center_x - scoop_r / 2, scoop_y + scoop_r / 3),
            scoop_r / 5,
            scoop.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x + scoop_r / 3, scoop_y + scoop_r / 2),
            scoop_r / 6,
            scoop.into(),
        );
    }
    let chip_count = 2 + (identity.byte(8) % 4) as i32;
    for chip in 0..chip_count {
        let x = center_x - scoop_r / 2 + (identity.byte(9 + chip as usize) as i32 % scoop_r.max(1));
        let y = scoop_y - scoop_r / 3 + (identity.byte(14 + chip as usize) as i32 % scoop_r.max(1));
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            (width as f32 * 0.010).max(2.0) as i32,
            waffle.into(),
        );
    }
    Ok(image)
}

/// Render an octopus avatar from a stable identity.
pub fn render_octopus_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.54 + identity.unit_f32(5) * 0.05)) as i32;
    let bg = hsl_to_color(185.0 + identity.unit_f32(0) * 70.0, 0.22, 0.92);
    let body = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.34, 0.38);
    let eye = Color::rgb(28, 26, 38);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.21 + identity.unit_f32(3) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.20 + identity.unit_f32(4) * 0.06)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, shade, 0.22, background,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        body.into(),
    );
    let tentacles = 4 + (identity.byte(6) % 3) as i32;
    for index in 0..tentacles {
        let denominator = (tentacles - 1).max(1);
        let x = center_x - head_rx + index * (head_rx * 2 / denominator);
        let length =
            (head_ry as f32 * (0.42 + identity.unit_f32(7 + index as usize) * 0.35)) as i32;
        draw_filled_rect_mut(
            &mut image,
            Rect::at(x - head_rx / 12, center_y + head_ry / 2)
                .of_size((head_rx / 6).max(2) as u32, length.max(2) as u32),
            body.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (x, center_y + head_ry / 2 + length),
            (head_rx / 10).max(3),
            body.into(),
        );
    }
    for side in [-1, 1] {
        let ex = center_x + side * head_rx / 3;
        let ey = center_y - head_ry / 6;
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            head_rx / 9,
            Color::rgb(255, 255, 248).into(),
        );
        draw_filled_circle_mut(&mut image, (ex, ey), head_rx / 20, eye.into());
    }
    let mouth = match identity.byte(14) % 3 {
        0 => CreatureMouthStyle::Smile,
        1 => CreatureMouthStyle::Flat,
        _ => CreatureMouthStyle::Fang,
    };
    draw_creature_mouth(
        &mut image,
        FaceLayout {
            center_x,
            center_y,
            head_rx,
            head_ry,
        },
        mouth,
        shade,
    );
    Ok(image)
}

/// Render a knight helmet avatar from a stable identity.
pub fn render_knight_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.55) as i32;
    let bg = hsl_to_color(215.0 + identity.unit_f32(0) * 30.0, 0.12, 0.92);
    let steel = hsl_to_color(205.0 + identity.unit_f32(1) * 45.0, 0.12, 0.66);
    let dark = hsl_to_color(215.0 + identity.unit_f32(2) * 45.0, 0.14, 0.22);
    let plume = hsl_to_color(identity.unit_f32(3) * 360.0, 0.58, 0.54);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let helm_rx = (width as f32 * (0.20 + identity.unit_f32(4) * 0.05)) as i32;
    let helm_ry = (height as f32 * (0.24 + identity.unit_f32(5) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, helm_rx, helm_ry, plume, 0.18, background,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y - helm_ry / 5),
        helm_rx,
        helm_ry,
        steel.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - helm_rx, center_y - helm_ry / 5)
            .of_size((helm_rx * 2) as u32, (helm_ry * 6 / 5) as u32),
        steel.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - helm_rx * 3 / 4, center_y - helm_ry / 5)
            .of_size((helm_rx * 3 / 2) as u32, (helm_ry / 5).max(2) as u32),
        dark.into(),
    );
    let slit_count = 2 + (identity.byte(6) % 3) as i32;
    for slit in 0..slit_count {
        let x = center_x - helm_rx / 2 + slit * helm_rx / slit_count.max(1);
        draw_filled_rect_mut(
            &mut image,
            Rect::at(x, center_y - helm_ry / 5)
                .of_size((helm_rx / 10).max(2) as u32, (helm_ry / 5).max(2) as u32),
            Color::rgba(255, 255, 255, 90).into(),
        );
    }
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, (center_y - helm_ry) as f32),
        (center_x as f32, (center_y + helm_ry) as f32),
        Color::rgba(255, 255, 255, 130).into(),
    );
    if !identity.byte(7).is_multiple_of(3) {
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x, center_y - helm_ry),
                Point::new(center_x - helm_rx / 5, center_y - helm_ry - helm_ry / 2),
                Point::new(center_x + helm_rx / 4, center_y - helm_ry - helm_ry / 3),
            ],
            plume.into(),
        );
    }
    Ok(image)
}

pub fn render_paws_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let bg = hsl_to_color(24.0 + identity.unit_f32(0) * 36.0, 0.20, 0.94);
    let fur = hsl_to_color(
        identity.unit_f32(1) * 360.0,
        0.32 + identity.unit_f32(2) * 0.18,
        0.60,
    );
    let pad = hsl_to_color(
        330.0 + identity.unit_f32(3) * 20.0,
        0.36 + identity.unit_f32(4) * 0.18,
        0.72,
    );
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

    if !identity.byte(12).is_multiple_of(3) {
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

    Ok(image)
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
    if background != AvatarBackground::Themed {
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
        AvatarBackground::Black => Color::rgb(0, 0, 0),
        AvatarBackground::Dark => Color::rgb(17, 24, 39),
        AvatarBackground::Light => Color::rgb(248, 250, 247),
        AvatarBackground::Transparent => Color::rgba(255, 255, 255, 0),
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
    let eye_spacing = if eyes == 1 {
        0.0
    } else {
        w * 0.22 / (eyes - 1) as f32
    };
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
    let cy = h * (0.53 + identity.unit_f32(5) * 0.06);
    let rx = w * (0.19 + identity.unit_f32(3) * 0.08);
    let ry = h * (0.21 + identity.unit_f32(4) * 0.08);
    let body = hsl_to_color(
        190.0 + identity.unit_f32(1) * 55.0,
        0.10 + identity.unit_f32(7) * 0.10,
        0.94 + identity.unit_f32(8) * 0.04,
    );
    let eye_style = if identity.byte(19).is_multiple_of(2) {
        (w * 0.026, h * 0.054)
    } else {
        (w * 0.038, h * 0.038)
    };
    let mouth = if identity.byte(20) % 3 == 1 {
        format!(
            r##"<ellipse cx="{cx}" cy="{my}" rx="{mrx}" ry="{mry}" fill="#8da0b2"/>"##,
            my = cy + h * 0.08,
            mrx = w * 0.035,
            mry = h * 0.045,
        )
    } else {
        format!(
            r##"<path d="M {mx1} {my} q {cq} {cyq} {ce} 0 M {mx2} {my} q {cq} {cyq} {ce} 0" stroke="#8da0b2" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
            mx1 = cx - w * 0.03,
            mx2 = cx + w * 0.03,
            my = cy + h * 0.08,
            cq = w * 0.04,
            cyq = if identity.byte(20) % 3 == 2 {
                0.0
            } else {
                h * 0.05
            },
            ce = w * 0.06,
        )
    };
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{body}"/><rect x="{x}" y="{cy}" width="{rw}" height="{rh}" fill="{body}"/><circle cx="{c1}" cy="{scy}" r="{sr1}" fill="{body}"/><circle cx="{c2}" cy="{scy}" r="{sr2}" fill="{body}"/><circle cx="{c3}" cy="{scy}" r="{sr3}" fill="{body}"/><ellipse cx="{lx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/><ellipse cx="{rx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/>{mouth}"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        body = color_hex(body),
        x = cx - rx,
        rw = rx * 2.0,
        rh = ry * (0.82 + identity.unit_f32(12) * 0.28),
        c1 = cx - rx * 0.70,
        c2 = cx,
        c3 = cx + rx * 0.70,
        scy = cy + ry * 1.36,
        sr1 = rx * (0.18 + identity.unit_f32(13) * 0.12),
        sr2 = rx * (0.18 + identity.unit_f32(14) * 0.12),
        sr3 = rx * (0.18 + identity.unit_f32(15) * 0.12),
        lx = cx - rx * 0.36,
        rx2 = cx + rx * 0.36,
        ey = cy - ry * 0.25,
        erx = eye_style.0,
        ery = eye_style.1,
        mouth = mouth,
    )
}

fn render_slime_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.56 + identity.unit_f32(8) * 0.08);
    let rx = w * (0.20 + identity.unit_f32(6) * 0.10);
    let ry = h * (0.16 + identity.unit_f32(7) * 0.08);
    let slime = hsl_to_color(
        70.0 + identity.unit_f32(4) * 130.0,
        0.44 + identity.unit_f32(9) * 0.22,
        0.46 + identity.unit_f32(10) * 0.18,
    );
    let eye_count = 1 + (identity.byte(49) % 3) as usize;
    let eye_markup = match eye_count {
        1 => format!(
            r##"<circle cx="{cx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{cx}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            ey = cy - ry * 0.20,
            er = rx * 0.14,
            pr = rx * 0.055,
        ),
        2 => format!(
            r##"<circle cx="{lx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{rx2}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#203018"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            lx = cx - rx * 0.34,
            rx2 = cx + rx * 0.34,
            ey = cy - ry * 0.22,
            er = rx * 0.12,
            pr = rx * 0.050,
        ),
        _ => format!(
            r##"<circle cx="{lx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{cx}" cy="{ey2}" r="{er2}" fill="#f8ffec"/><circle cx="{rx2}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#203018"/><circle cx="{cx}" cy="{ey2}" r="{pr}" fill="#203018"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            lx = cx - rx * 0.34,
            rx2 = cx + rx * 0.34,
            ey = cy - ry * 0.26,
            ey2 = cy - ry * 0.14,
            er = rx * 0.11,
            er2 = rx * 0.095,
            pr = rx * 0.045,
        ),
    };
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{slime}"/><rect x="{dx1}" y="{cy}" width="{dw1}" height="{dh1}" fill="{slime}"/><rect x="{dx2}" y="{cy}" width="{dw2}" height="{dh2}" fill="{slime}"/><rect x="{dx3}" y="{cy}" width="{dw3}" height="{dh3}" fill="{slime}"/>{eye_markup}<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="{mr}" fill="#305228"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        slime = color_hex(slime),
        dx1 = cx - rx * 0.66,
        dx2 = cx - rx * 0.14,
        dx3 = cx + rx * 0.34,
        dw1 = rx * (0.24 + identity.unit_f32(14) * 0.12),
        dw2 = rx * (0.20 + identity.unit_f32(15) * 0.14),
        dw3 = rx * (0.22 + identity.unit_f32(16) * 0.14),
        dh1 = ry * (0.62 + identity.unit_f32(22) * 0.60),
        dh2 = ry * (0.42 + identity.unit_f32(23) * 0.55),
        dh3 = ry * (0.54 + identity.unit_f32(24) * 0.62),
        eye_markup = eye_markup,
        mx = cx - rx * 0.42,
        my = cy + ry * 0.40,
        mw = rx * 0.84,
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
    let cy = h * (0.57 + identity.unit_f32(17) * 0.07);
    let r = w * (0.16 + identity.unit_f32(15) * 0.06);
    let hat = hsl_to_color(
        210.0 + identity.unit_f32(11) * 110.0,
        0.34 + identity.unit_f32(20) * 0.22,
        0.28 + identity.unit_f32(21) * 0.16,
    );
    let band = hsl_to_color(
        24.0 + identity.unit_f32(12) * 160.0,
        0.62 + identity.unit_f32(22) * 0.24,
        0.48 + identity.unit_f32(23) * 0.18,
    );
    let skin = hsl_to_color(
        18.0 + identity.unit_f32(13) * 28.0,
        0.22 + identity.unit_f32(24) * 0.20,
        0.74 + identity.unit_f32(25) * 0.12,
    );
    let beard = hsl_to_color(
        35.0 + identity.unit_f32(14) * 45.0,
        0.06 + identity.unit_f32(26) * 0.12,
        0.80 + identity.unit_f32(27) * 0.16,
    );
    let hat_width = r * (1.0 + identity.unit_f32(28) * 0.55);
    let hat_height = h * (0.28 + identity.unit_f32(29) * 0.12);
    let tip_shift = (identity.unit_f32(30) - 0.5) * r * 0.9;
    let stars = format!(
        r##"<circle cx="{s1x}" cy="{s1y}" r="{sr}" fill="{band}"/><circle cx="{s2x}" cy="{s2y}" r="{sr2}" fill="{band}"/>"##,
        s1x = cx - hat_width * 0.35,
        s1y = cy - h * 0.20,
        s2x = cx + tip_shift * 0.5 + hat_width * 0.22,
        s2y = cy - h * 0.28,
        sr = w * (0.010 + identity.unit_f32(34) * 0.012),
        sr2 = w * (0.008 + identity.unit_f32(35) * 0.010),
        band = color_hex(band),
    );
    format!(
        r##"<polygon points="{x1},{y1} {x2},{y1} {tx},{y2}" fill="{hat}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{band}"/>{stars}<circle cx="{cx}" cy="{cy}" r="{r}" fill="{skin}"/><polygon points="{b1},{b2} {b3},{b2} {bt},{b4}" fill="{beard}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{sx}" cy="{sy}" r="{sr}" fill="{band}"/>"##,
        cx = cx,
        cy = cy,
        x1 = cx - hat_width,
        x2 = cx + hat_width,
        tx = cx + tip_shift,
        y1 = cy - h * 0.08,
        y2 = cy - h * 0.08 - hat_height,
        hat = color_hex(hat),
        bx = cx - w * (0.24 + identity.unit_f32(31) * 0.08),
        by = cy - h * 0.08,
        bw = w * (0.48 + identity.unit_f32(31) * 0.16),
        bh = h * (0.030 + identity.unit_f32(32) * 0.025),
        band = color_hex(band),
        stars = stars,
        r = r,
        skin = color_hex(skin),
        b1 = cx - r * (0.52 + identity.unit_f32(44) * 0.20),
        b2 = cy + h * 0.06,
        b3 = cx + r * (0.52 + identity.unit_f32(45) * 0.20),
        bt = cx + (identity.unit_f32(46) - 0.5) * r * 0.45,
        b4 = cy + h * (0.22 + identity.unit_f32(47) * 0.10),
        beard = color_hex(beard),
        elx = cx - r * 0.36,
        erx = cx + r * 0.36,
        ey = cy - h * 0.03,
        er = r * 0.13,
        pr = r * 0.055,
        sx = cx + tip_shift + r * 0.50,
        sy = cy - h * 0.08 - hat_height,
        sr = r * 0.10,
    )
}

fn render_skull_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.51 + identity.unit_f32(19) * 0.07);
    let rx = w * (0.18 + identity.unit_f32(17) * 0.07);
    let ry = h * (0.18 + identity.unit_f32(18) * 0.07);
    let bone = hsl_to_color(
        28.0 + identity.unit_f32(16) * 34.0,
        0.08 + identity.unit_f32(22) * 0.10,
        0.82 + identity.unit_f32(23) * 0.12,
    );
    let crack = color_hex(hsl_to_color(
        20.0 + identity.unit_f32(24) * 40.0,
        0.06,
        0.22 + identity.unit_f32(25) * 0.12,
    ));
    let teeth = 3 + (identity.byte(32) % 4) as usize;
    let mut tooth_markup = String::new();
    for tooth in 0..teeth {
        let x = cx - rx * 0.34 + tooth as f32 * (rx * 0.68 / teeth.max(1) as f32);
        tooth_markup.push_str(&format!(
            r##"<line x1="{x}" y1="{ty1}" x2="{x}" y2="{ty2}" stroke="{crack}" stroke-width="3"/>"##,
            ty1 = cy + ry * 0.52,
            ty2 = cy + ry * (0.86 + identity.unit_f32(46 + tooth) * 0.25),
        ));
    }
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{bone}"/><rect x="{jx}" y="{jy}" width="{jw}" height="{jh}" fill="{bone}"/><ellipse cx="{elx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{crack}"/><ellipse cx="{erx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{crack}"/><polygon points="{cx},{ny} {nx1},{ny2} {nx2},{ny2}" fill="{crack}"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" fill="{crack}"/>{tooth_markup}<line x1="{cx1}" y1="{cy1}" x2="{cx2}" y2="{cy2}" stroke="{crack}" stroke-width="3"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        bone = color_hex(bone),
        crack = crack,
        jx = cx - rx * 0.45,
        jy = cy + ry * 0.50,
        jw = rx * (0.82 + identity.unit_f32(26) * 0.34),
        jh = ry * (0.34 + identity.unit_f32(27) * 0.28),
        elx = cx - rx * 0.34,
        erx2 = cx + rx * 0.34,
        ey = cy - ry * 0.20,
        erx = rx * (0.18 + identity.unit_f32(29) * 0.08),
        ery = ry * (0.25 + identity.unit_f32(30) * 0.12),
        ny = cy,
        nx1 = cx - rx * 0.10,
        nx2 = cx + rx * 0.10,
        ny2 = cy + ry * (0.32 + identity.unit_f32(30) * 0.16),
        mx = cx - rx * 0.48,
        my = cy + ry * 0.50,
        mw = rx * 0.96,
        mh = h * 0.02,
        tooth_markup = tooth_markup,
        cx1 = cx - rx * 0.15,
        cy1 = cy - ry * 0.45,
        cx2 = cx + (identity.unit_f32(42) - 0.5) * rx * 0.40,
        cy2 = cy + ry * 0.10,
    )
}

fn render_planet_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.53 + identity.unit_f32(8) * 0.06);
    let r = w.min(h) * (0.18 + identity.unit_f32(20) * 0.08);
    let planet = hsl_to_color(identity.unit_f32(1) * 360.0, 0.46, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.42);
    let ring = hsl_to_color(32.0 + identity.unit_f32(3) * 120.0, 0.44, 0.72);
    let ring_rx = r * (1.55 + identity.unit_f32(21) * 0.28);
    let ring_ry = r * (0.38 + identity.unit_f32(22) * 0.12);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rrx}" ry="{rry}" fill="{ring}"/><circle cx="{cx}" cy="{cy}" r="{r}" fill="{planet}"/><ellipse cx="{sx}" cy="{sy}" rx="{srx}" ry="{sry}" fill="{shade}" opacity="0.45"/><ellipse cx="{hx}" cy="{hy}" rx="{hrx}" ry="{hry}" fill="#ffffff" opacity="0.32"/><rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" rx="{cr}" fill="{ring}" opacity="0.78"/>"##,
        cx = cx,
        cy = cy,
        r = r,
        rrx = ring_rx,
        rry = ring_ry,
        ring = color_hex(ring),
        planet = color_hex(planet),
        sx = cx - r * 0.25,
        sy = cy - r * 0.20,
        srx = r * 0.50,
        sry = r * 0.20,
        shade = color_hex(shade),
        hx = cx + r * 0.25,
        hy = cy + r * 0.20,
        hrx = r * 0.50,
        hry = r * 0.16,
        rx = cx - ring_rx,
        ry = cy - ring_ry * 0.16,
        rw = ring_rx * 2.0,
        rh = ring_ry * 0.32,
        cr = ring_ry * 0.16,
    )
}

fn render_rocket_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.52;
    let body_w = w * (0.18 + identity.unit_f32(5) * 0.05);
    let body_h = h * (0.42 + identity.unit_f32(6) * 0.08);
    let top_y = cy - body_h / 2.0;
    let bottom_y = cy + body_h / 2.0;
    let hull = hsl_to_color(200.0 + identity.unit_f32(1) * 50.0, 0.12, 0.88);
    let trim = hsl_to_color(identity.unit_f32(2) * 360.0, 0.58, 0.54);
    let window = hsl_to_color(185.0 + identity.unit_f32(3) * 70.0, 0.54, 0.72);
    let flame = hsl_to_color(20.0 + identity.unit_f32(4) * 30.0, 0.86, 0.58);
    let mut windows = String::new();
    for index in 0..(1 + (identity.byte(7) % 2) as usize) {
        let wy = top_y + body_h / 3.0 + index as f32 * body_w;
        windows.push_str(&format!(
            r##"<circle cx="{cx}" cy="{wy}" r="{wr}" fill="{trim}"/><circle cx="{cx}" cy="{wy}" r="{ir}" fill="{window}"/>"##,
            wr = body_w * 0.22,
            ir = body_w * 0.15,
            trim = color_hex(trim),
            window = color_hex(window),
        ));
    }
    format!(
        r##"<polygon points="{nx1},{ny1} {nx2},{ny1} {cx},{ny2}" fill="{trim}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{hull}"/><ellipse cx="{cx}" cy="{by}" rx="{erx}" ry="{ery}" fill="{hull}"/><polygon points="{lf1},{lfy1} {lf2},{lfy2} {lf3},{lfy3}" fill="{trim}"/><polygon points="{rf1},{lfy1} {rf2},{lfy2} {rf3},{lfy3}" fill="{trim}"/>{windows}<polygon points="{fx1},{fy1} {fx2},{fy1} {cx},{fy2}" fill="{flame}"/>"##,
        cx = cx,
        nx1 = cx - body_w / 2.0,
        nx2 = cx + body_w / 2.0,
        ny1 = top_y + body_w / 2.0,
        ny2 = top_y - body_w / 2.0,
        trim = color_hex(trim),
        bx = cx - body_w / 2.0,
        by = top_y + body_w / 2.0,
        bw = body_w,
        bh = body_h - body_w / 2.0,
        hull = color_hex(hull),
        erx = body_w / 2.0,
        ery = body_w / 5.0,
        lf1 = cx - body_w / 2.0,
        lf2 = cx - body_w,
        lf3 = cx - body_w / 2.0,
        rf1 = cx + body_w / 2.0,
        rf2 = cx + body_w,
        rf3 = cx + body_w / 2.0,
        lfy1 = bottom_y - body_w / 2.0,
        lfy2 = bottom_y + body_w / 3.0,
        lfy3 = bottom_y,
        windows = windows,
        fx1 = cx - body_w / 4.0,
        fx2 = cx + body_w / 4.0,
        fy1 = bottom_y,
        fy2 = bottom_y + h * (0.10 + identity.unit_f32(8) * 0.08),
        flame = color_hex(flame),
    )
}

fn render_mushroom_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let cap_rx = w * (0.24 + identity.unit_f32(4) * 0.08);
    let cap_ry = h * (0.14 + identity.unit_f32(5) * 0.06);
    let stem_rx = w * (0.09 + identity.unit_f32(6) * 0.04);
    let stem_ry = h * (0.18 + identity.unit_f32(7) * 0.05);
    let cap = hsl_to_color(350.0 + identity.unit_f32(1) * 45.0, 0.58, 0.52);
    let stem = hsl_to_color(35.0 + identity.unit_f32(2) * 20.0, 0.24, 0.86);
    let gill = hsl_to_color(26.0 + identity.unit_f32(3) * 20.0, 0.20, 0.70);
    let spots = format!(
        r##"<circle cx="{s1x}" cy="{s1y}" r="{sr1}" fill="#fff6e6" opacity="0.92"/><circle cx="{s2x}" cy="{s2y}" r="{sr2}" fill="#fff6e6" opacity="0.92"/><circle cx="{s3x}" cy="{s3y}" r="{sr3}" fill="#fff6e6" opacity="0.92"/>"##,
        s1x = cx - cap_rx * 0.36,
        s2x = cx + cap_rx * 0.08,
        s3x = cx + cap_rx * 0.40,
        s1y = cy - cap_ry * 0.88,
        s2y = cy - cap_ry * 0.58,
        s3y = cy - cap_ry * 0.76,
        sr1 = cap_rx * (0.06 + identity.unit_f32(19) * 0.04),
        sr2 = cap_rx * (0.07 + identity.unit_f32(20) * 0.04),
        sr3 = cap_rx * (0.05 + identity.unit_f32(21) * 0.04),
    );
    format!(
        r##"<ellipse cx="{cx}" cy="{scy}" rx="{srx}" ry="{sry}" fill="{stem}"/><ellipse cx="{cx}" cy="{ccy}" rx="{crx}" ry="{cry}" fill="{cap}"/><rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" fill="{cap}"/><ellipse cx="{cx}" cy="{gcy}" rx="{grx}" ry="{gry}" fill="{gill}"/>{spots}"##,
        cx = cx,
        scy = cy + stem_ry / 3.0,
        srx = stem_rx,
        sry = stem_ry,
        stem = color_hex(stem),
        ccy = cy - cap_ry / 2.0,
        crx = cap_rx,
        cry = cap_ry,
        cap = color_hex(cap),
        rx = cx - cap_rx,
        ry = cy - cap_ry / 2.0,
        rw = cap_rx * 2.0,
        rh = cap_ry,
        gcy = cy + cap_ry / 3.0,
        grx = cap_rx,
        gry = cap_ry / 3.0,
        gill = color_hex(gill),
        spots = spots,
    )
}

fn render_cactus_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let body_w = w * (0.13 + identity.unit_f32(4) * 0.04);
    let body_h = h * (0.36 + identity.unit_f32(5) * 0.10);
    let top_y = cy - body_h / 2.0;
    let cactus = hsl_to_color(105.0 + identity.unit_f32(1) * 60.0, 0.42, 0.42);
    let flower = hsl_to_color(320.0 + identity.unit_f32(3) * 55.0, 0.58, 0.64);
    format!(
        r##"<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="{br}" fill="{cactus}"/><circle cx="{cx}" cy="{by}" r="{br}" fill="{cactus}"/><rect x="{lax}" y="{lay}" width="{al}" height="{ah}" rx="{ar}" fill="{cactus}"/><circle cx="{lax}" cy="{lcy}" r="{ar}" fill="{cactus}"/><rect x="{rax}" y="{ray}" width="{al}" height="{ah}" rx="{ar}" fill="{cactus}"/><circle cx="{rcx}" cy="{rcy}" r="{ar}" fill="{cactus}"/><line x1="{n1x}" y1="{n1y}" x2="{n2x}" y2="{n2y}" stroke="#f2ffe0" stroke-width="2" opacity="0.7"/><line x1="{n3x}" y1="{n3y}" x2="{n4x}" y2="{n4y}" stroke="#f2ffe0" stroke-width="2" opacity="0.7"/><circle cx="{cx}" cy="{fy}" r="{fr}" fill="{flower}"/>"##,
        bx = cx - body_w / 2.0,
        by = top_y,
        bw = body_w,
        bh = body_h,
        br = body_w / 2.0,
        cactus = color_hex(cactus),
        lax = cx - body_w * 1.45,
        lay = cy - body_h * 0.13,
        lcy = cy - body_h * 0.13 + body_w * 0.25,
        rax = cx + body_w * 0.35,
        ray = cy - body_h * 0.23,
        rcx = cx + body_w * 1.45,
        rcy = cy - body_h * 0.23 + body_w * 0.25,
        al = body_w * 1.10,
        ah = body_w * 0.50,
        ar = body_w * 0.25,
        n1x = cx - body_w * 0.12,
        n1y = top_y + body_h * 0.34,
        n2x = cx - body_w * 0.34,
        n2y = top_y + body_h * 0.29,
        n3x = cx + body_w * 0.10,
        n3y = top_y + body_h * 0.58,
        n4x = cx + body_w * 0.33,
        n4y = top_y + body_h * 0.54,
        fy = top_y - body_w * 0.42,
        fr = body_w * (0.16 + identity.unit_f32(12) * 0.08),
        flower = color_hex(flower),
    )
}

fn render_frog_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.57 + identity.unit_f32(6) * 0.04);
    let rx = w * (0.24 + identity.unit_f32(4) * 0.06);
    let ry = h * (0.18 + identity.unit_f32(5) * 0.05);
    let green = hsl_to_color(92.0 + identity.unit_f32(1) * 72.0, 0.46, 0.54);
    let dark = hsl_to_color(98.0 + identity.unit_f32(2) * 60.0, 0.40, 0.28);
    let cheek = hsl_to_color(335.0 + identity.unit_f32(3) * 24.0, 0.42, 0.76);
    let er = rx * (0.18 + identity.unit_f32(7) * 0.04);
    format!(
        r##"<circle cx="{elx}" cy="{ey}" r="{er}" fill="{green}"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="{green}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{green}"/><circle cx="{elx}" cy="{ey}" r="{ew}" fill="#fffff5"/><circle cx="{erx}" cy="{ey}" r="{ew}" fill="#fffff5"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="{dark}"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="{dark}"/><circle cx="{clx}" cy="{ccy}" r="{cr}" fill="{cheek}" opacity="0.6"/><circle cx="{crx}" cy="{ccy}" r="{cr}" fill="{cheek}" opacity="0.6"/><path d="M {mx1} {my} q {q1} {qd} {q2} 0 M {mx2} {my} q {q1} {qd} {q2} 0" stroke="{dark}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        green = color_hex(green),
        elx = cx - rx * 0.50,
        erx = cx + rx * 0.50,
        ey = cy - ry,
        er = er,
        ew = er * 0.64,
        pr = er * 0.30,
        dark = color_hex(dark),
        clx = cx - rx * 0.50,
        crx = cx + rx * 0.50,
        ccy = cy + ry * 0.25,
        cr = rx * 0.09,
        cheek = color_hex(cheek),
        mx1 = cx - rx * 0.20,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.22,
        q1 = rx * 0.20,
        qd = ry * 0.35,
        q2 = rx * 0.36,
    )
}

fn render_panda_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.56 + identity.unit_f32(5) * 0.04);
    let rx = w * (0.24 + identity.unit_f32(6) * 0.05);
    let ry = h * (0.22 + identity.unit_f32(7) * 0.04);
    let white = hsl_to_color(36.0 + identity.unit_f32(1) * 18.0, 0.10, 0.92);
    let black = hsl_to_color(210.0 + identity.unit_f32(2) * 28.0, 0.10, 0.18);
    let er = rx * (0.28 + identity.unit_f32(8) * 0.08);
    format!(
        r##"<circle cx="{lelx}" cy="{eary}" r="{er}" fill="{black}"/><circle cx="{rerx}" cy="{eary}" r="{er}" fill="{black}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{white}"/><ellipse cx="{plx}" cy="{py}" rx="{prx}" ry="{pry}" fill="{black}"/><ellipse cx="{prx2}" cy="{py}" rx="{prx}" ry="{pry}" fill="{black}"/><circle cx="{plx}" cy="{py}" r="{eye}" fill="#f8f8f4"/><circle cx="{prx2}" cy="{py}" r="{eye}" fill="#f8f8f4"/><ellipse cx="{cx}" cy="{ny}" rx="{nrx}" ry="{nry}" fill="{black}"/><path d="M {mx1} {my} q {mq} {md} {me} 0 M {mx2} {my} q {mq} {md} {me} 0" stroke="{black}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        white = color_hex(white),
        black = color_hex(black),
        lelx = cx - rx * 0.75,
        rerx = cx + rx * 0.75,
        eary = cy - ry * 0.75,
        er = er,
        plx = cx - rx * 0.33,
        prx2 = cx + rx * 0.33,
        py = cy - ry * 0.08,
        prx = rx * (0.20 + identity.unit_f32(9) * 0.05),
        pry = ry * (0.26 + identity.unit_f32(10) * 0.05),
        eye = rx * 0.055,
        ny = cy + ry * 0.20,
        nrx = rx * 0.09,
        nry = ry * 0.12,
        mx1 = cx - rx * 0.08,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.26,
        mq = rx * 0.12,
        md = ry * 0.24,
        me = rx * 0.22,
    )
}

fn render_cupcake_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let cup_w = w * (0.26 + identity.unit_f32(5) * 0.07);
    let cup_h = h * (0.22 + identity.unit_f32(6) * 0.05);
    let frx = cup_w * (0.58 + identity.unit_f32(7) * 0.10);
    let fry = h * (0.13 + identity.unit_f32(8) * 0.04);
    let wrapper = hsl_to_color(28.0 + identity.unit_f32(1) * 35.0, 0.46, 0.62);
    let frosting = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.78);
    let cherry = hsl_to_color(345.0 + identity.unit_f32(4) * 22.0, 0.66, 0.50);
    let by = cy - fry / 2.0;
    format!(
        r##"<polygon points="{x1},{cy} {x2},{cy} {x3},{yb} {x4},{yb}" fill="{wrapper}"/><line x1="{sx1}" y1="{cy}" x2="{sx2}" y2="{yb}" stroke="#fff4d6" stroke-width="3" opacity="0.45"/><line x1="{sx3}" y1="{cy}" x2="{sx4}" y2="{yb}" stroke="#fff4d6" stroke-width="3" opacity="0.45"/><ellipse cx="{cx}" cy="{f1y}" rx="{frx}" ry="{fry}" fill="{frosting}"/><ellipse cx="{cx}" cy="{f2y}" rx="{frx2}" ry="{fry2}" fill="{frosting}"/><ellipse cx="{cx}" cy="{f3y}" rx="{frx3}" ry="{fry3}" fill="{frosting}"/><rect x="{spx1}" y="{spy1}" width="{spw}" height="{sph}" fill="#f05f7e"/><rect x="{spx2}" y="{spy2}" width="{spw}" height="{sph}" fill="#5fb6f0"/><rect x="{spx3}" y="{spy3}" width="{spw}" height="{sph}" fill="#f0d15f"/><circle cx="{cx}" cy="{chy}" r="{chr}" fill="{cherry}"/>"##,
        cx = cx,
        cy = cy,
        x1 = cx - cup_w / 2.0,
        x2 = cx + cup_w / 2.0,
        x3 = cx + cup_w / 3.0,
        x4 = cx - cup_w / 3.0,
        yb = cy + cup_h,
        wrapper = color_hex(wrapper),
        sx1 = cx - cup_w * 0.25,
        sx2 = cx - cup_w * 0.16,
        sx3 = cx + cup_w * 0.25,
        sx4 = cx + cup_w * 0.16,
        f1y = by,
        f2y = by - fry * 0.50,
        f3y = by - fry,
        frx = frx,
        fry = fry,
        frx2 = frx * 0.78,
        fry2 = fry * 0.72,
        frx3 = frx * 0.56,
        fry3 = fry * 0.62,
        frosting = color_hex(frosting),
        spx1 = cx - frx * 0.35,
        spx2 = cx + frx * 0.10,
        spx3 = cx - frx * 0.02,
        spy1 = by - fry * 0.50,
        spy2 = by - fry * 0.88,
        spy3 = by - fry * 0.12,
        spw = w * 0.035,
        sph = h * 0.012,
        chy = by - fry,
        chr = w * 0.035,
        cherry = color_hex(cherry),
    )
}

fn render_pizza_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.53;
    let half_w = w * (0.22 + identity.unit_f32(5) * 0.06);
    let slice_h = h * (0.44 + identity.unit_f32(6) * 0.07);
    let top_y = cy - slice_h / 2.0;
    let tip_y = cy + slice_h / 2.0;
    let crust = hsl_to_color(30.0 + identity.unit_f32(1) * 28.0, 0.54, 0.58);
    let cheese = hsl_to_color(45.0 + identity.unit_f32(2) * 16.0, 0.74, 0.70);
    let sauce = hsl_to_color(8.0 + identity.unit_f32(3) * 16.0, 0.62, 0.48);
    let topping = hsl_to_color(350.0 + identity.unit_f32(4) * 22.0, 0.54, 0.46);
    format!(
        r##"<polygon points="{x1},{ty} {x2},{ty} {cx},{tip}" fill="{cheese}"/><polygon points="{sx1},{sy} {sx2},{sy} {cx},{stip}" fill="{sauce}" opacity="0.38"/><ellipse cx="{cx}" cy="{ty}" rx="{half_w}" ry="{crh}" fill="{crust}"/><circle cx="{p1x}" cy="{p1y}" r="{pr}" fill="{topping}"/><circle cx="{p2x}" cy="{p2y}" r="{pr2}" fill="{topping}"/><circle cx="{p3x}" cy="{p3y}" r="{pr3}" fill="{topping}"/>"##,
        cx = cx,
        x1 = cx - half_w,
        x2 = cx + half_w,
        ty = top_y,
        tip = tip_y,
        cheese = color_hex(cheese),
        sx1 = cx - half_w * 0.78,
        sx2 = cx + half_w * 0.78,
        sy = top_y + slice_h * 0.20,
        stip = tip_y - slice_h * 0.10,
        sauce = color_hex(sauce),
        half_w = half_w,
        crh = h * 0.035,
        crust = color_hex(crust),
        p1x = cx - half_w * 0.35,
        p2x = cx + half_w * 0.28,
        p3x = cx,
        p1y = top_y + slice_h * 0.28,
        p2y = top_y + slice_h * 0.40,
        p3y = top_y + slice_h * 0.62,
        pr = w * (0.025 + identity.unit_f32(14) * 0.012),
        pr2 = w * (0.026 + identity.unit_f32(15) * 0.012),
        pr3 = w * (0.024 + identity.unit_f32(16) * 0.012),
        topping = color_hex(topping),
    )
}

fn render_icecream_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.55;
    let scoop_r = w * (0.18 + identity.unit_f32(4) * 0.06);
    let cone_w = w * (0.24 + identity.unit_f32(5) * 0.05);
    let cone_h = h * (0.32 + identity.unit_f32(6) * 0.06);
    let scoop_y = cy - scoop_r / 2.0;
    let cone_top = scoop_y + scoop_r / 2.0;
    let scoop = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.76);
    let cone = hsl_to_color(32.0 + identity.unit_f32(2) * 22.0, 0.50, 0.64);
    let waffle = hsl_to_color(28.0 + identity.unit_f32(3) * 22.0, 0.42, 0.45);
    format!(
        r##"<polygon points="{x1},{ct} {x2},{ct} {cx},{cb}" fill="{cone}"/><line x1="{lx1}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="{waffle}" stroke-width="3"/><line x1="{lx2}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="{waffle}" stroke-width="3"/><circle cx="{cx}" cy="{sy}" r="{sr}" fill="{scoop}"/><circle cx="{d1x}" cy="{d1y}" r="{dr1}" fill="{scoop}"/><circle cx="{d2x}" cy="{d2y}" r="{dr2}" fill="{scoop}"/><circle cx="{c1x}" cy="{c1y}" r="{chip}" fill="{waffle}"/><circle cx="{c2x}" cy="{c2y}" r="{chip}" fill="{waffle}"/>"##,
        cx = cx,
        x1 = cx - cone_w / 2.0,
        x2 = cx + cone_w / 2.0,
        ct = cone_top,
        cb = cone_top + cone_h,
        cone = color_hex(cone),
        lx1 = cx - cone_w / 3.0,
        lx2 = cx + cone_w / 3.0,
        ly1 = cone_top + cone_h / 8.0,
        ly2 = cone_top + cone_h * 0.75,
        waffle = color_hex(waffle),
        sy = scoop_y,
        sr = scoop_r,
        scoop = color_hex(scoop),
        d1x = cx - scoop_r / 2.0,
        d1y = scoop_y + scoop_r / 3.0,
        dr1 = scoop_r / 5.0,
        d2x = cx + scoop_r / 3.0,
        d2y = scoop_y + scoop_r / 2.0,
        dr2 = scoop_r / 6.0,
        c1x = cx - scoop_r * 0.25,
        c1y = scoop_y - scoop_r * 0.18,
        c2x = cx + scoop_r * 0.18,
        c2y = scoop_y + scoop_r * 0.12,
        chip = w * 0.010,
    )
}

fn render_octopus_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.54 + identity.unit_f32(5) * 0.05);
    let rx = w * (0.21 + identity.unit_f32(3) * 0.06);
    let ry = h * (0.20 + identity.unit_f32(4) * 0.06);
    let body = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.34, 0.38);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{body}"/><rect x="{t1x}" y="{ty}" width="{tw}" height="{th1}" rx="{tr}" fill="{body}"/><rect x="{t2x}" y="{ty}" width="{tw}" height="{th2}" rx="{tr}" fill="{body}"/><rect x="{t3x}" y="{ty}" width="{tw}" height="{th3}" rx="{tr}" fill="{body}"/><rect x="{t4x}" y="{ty}" width="{tw}" height="{th4}" rx="{tr}" fill="{body}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fffff8"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fffff8"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#1c1a26"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#1c1a26"/><path d="M {mx1} {my} q {mq} {md} {me} 0 M {mx2} {my} q {mq} {md} {me} 0" stroke="{shade}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        body = color_hex(body),
        t1x = cx - rx * 0.82,
        t2x = cx - rx * 0.28,
        t3x = cx + rx * 0.20,
        t4x = cx + rx * 0.68,
        ty = cy + ry * 0.45,
        tw = rx * 0.14,
        th1 = ry * (0.50 + identity.unit_f32(7) * 0.30),
        th2 = ry * (0.42 + identity.unit_f32(8) * 0.30),
        th3 = ry * (0.45 + identity.unit_f32(9) * 0.30),
        th4 = ry * (0.50 + identity.unit_f32(10) * 0.30),
        tr = rx * 0.07,
        elx = cx - rx / 3.0,
        erx = cx + rx / 3.0,
        ey = cy - ry / 6.0,
        er = rx / 9.0,
        pr = rx / 20.0,
        mx1 = cx - rx * 0.12,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.30,
        mq = rx * 0.14,
        md = ry * 0.22,
        me = rx * 0.24,
        shade = color_hex(shade),
    )
}

fn render_knight_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.55;
    let rx = w * (0.20 + identity.unit_f32(4) * 0.05);
    let ry = h * (0.24 + identity.unit_f32(5) * 0.05);
    let steel = hsl_to_color(205.0 + identity.unit_f32(1) * 45.0, 0.12, 0.66);
    let dark = hsl_to_color(215.0 + identity.unit_f32(2) * 45.0, 0.14, 0.22);
    let plume = hsl_to_color(identity.unit_f32(3) * 360.0, 0.58, 0.54);
    format!(
        r##"<ellipse cx="{cx}" cy="{hy}" rx="{rx}" ry="{ry}" fill="{steel}"/><rect x="{x}" y="{y}" width="{rw}" height="{rh}" fill="{steel}"/><rect x="{vx}" y="{vy}" width="{vw}" height="{vh}" fill="{dark}"/><rect x="{s1x}" y="{vy}" width="{sw}" height="{vh}" fill="#ffffff" opacity="0.35"/><rect x="{s2x}" y="{vy}" width="{sw}" height="{vh}" fill="#ffffff" opacity="0.35"/><line x1="{cx}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="#ffffff" stroke-width="3" opacity="0.5"/><polygon points="{cx},{py1} {px2},{py2} {px3},{py3}" fill="{plume}"/>"##,
        cx = cx,
        hy = cy - ry / 5.0,
        rx = rx,
        ry = ry,
        steel = color_hex(steel),
        x = cx - rx,
        y = cy - ry / 5.0,
        rw = rx * 2.0,
        rh = ry * 1.20,
        vx = cx - rx * 0.75,
        vy = cy - ry / 5.0,
        vw = rx * 1.50,
        vh = ry * 0.20,
        dark = color_hex(dark),
        s1x = cx - rx * 0.34,
        s2x = cx + rx * 0.22,
        sw = rx * 0.10,
        ly1 = cy - ry,
        ly2 = cy + ry,
        py1 = cy - ry,
        px2 = cx - rx * 0.20,
        py2 = cy - ry * 1.50,
        px3 = cx + rx * 0.25,
        py3 = cy - ry * 1.32,
        plume = color_hex(plume),
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
        AvatarOutputFormat::Jpeg => {
            let rgb = rgba_to_rgb_over_white(image);
            JpegEncoder::new_with_quality(writer, 92).write_image(
                &rgb,
                image.width(),
                image.height(),
                ExtendedColorType::Rgb8,
            )
        }
        AvatarOutputFormat::Gif => GifEncoder::new(writer).write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        ),
    }
}

fn rgba_to_rgb_over_white(image: &RgbaImage) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(image.as_raw().len() / 4 * 3);
    for pixel in image.pixels() {
        let [red, green, blue, alpha] = pixel.0;
        let alpha = u32::from(alpha);
        let inverse_alpha = 255 - alpha;
        rgb.push(((u32::from(red) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
        rgb.push(((u32::from(green) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
        rgb.push(((u32::from(blue) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
    }
    rgb
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;

    fn valid_spec(width: u32, height: u32, seed: u64) -> AvatarSpec {
        AvatarSpec::new(width, height, seed).expect("test avatar spec should be valid")
    }

    fn valid_namespace<'a>(tenant: &'a str, style_version: &'a str) -> AvatarNamespace<'a> {
        super::AvatarNamespace::new(tenant, style_version).expect("test namespace should be valid")
    }

    fn valid_identity<T: AsRef<[u8]>>(input: T) -> AvatarIdentity {
        super::AvatarIdentity::new(input).expect("test identity should be valid")
    }

    fn valid_identity_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> AvatarIdentity {
        AvatarIdentity::new_with_namespace(namespace, input).expect("test identity should be valid")
    }

    fn render_avatar_for_id<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        id: T,
        options: AvatarOptions,
    ) -> RgbaImage {
        super::render_avatar_for_id(spec, id, options).expect("valid avatar spec should render")
    }

    fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        id: T,
        options: AvatarOptions,
    ) -> String {
        super::render_avatar_svg_for_id(spec, id, options)
            .expect("valid avatar spec should render as svg")
    }

    fn render_cat_avatar(spec: AvatarSpec) -> RgbaImage {
        super::render_cat_avatar(spec).expect("valid avatar spec should render")
    }

    fn render_cat_avatar_for_identity(spec: AvatarSpec, identity: &AvatarIdentity) -> RgbaImage {
        super::render_cat_avatar_for_identity(spec, identity)
            .expect("valid avatar spec should render")
    }

    fn render_cat_avatar_for_identity_with_background(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_cat_avatar_for_identity_with_background(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    fn render_dog_avatar_for_identity(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_dog_avatar_for_identity(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    fn render_robot_avatar_for_identity(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_robot_avatar_for_identity(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    fn render_alien_avatar_for_identity(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_alien_avatar_for_identity(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    fn render_monster_avatar_for_identity(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_monster_avatar_for_identity(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    fn render_paws_avatar_for_identity(
        spec: AvatarSpec,
        identity: &AvatarIdentity,
        background: AvatarBackground,
    ) -> RgbaImage {
        super::render_paws_avatar_for_identity(spec, identity, background)
            .expect("valid avatar spec should render")
    }

    #[test]
    fn cat_avatar_is_deterministic_for_a_seed() {
        let spec = valid_spec(256, 256, 42);
        let left = render_cat_avatar(spec);
        let right = render_cat_avatar(spec);

        assert_eq!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn cat_avatar_uses_requested_dimensions() {
        let image = render_cat_avatar(valid_spec(192, 160, 7));

        assert_eq!(image.width(), 192);
        assert_eq!(image.height(), 160);
    }

    #[test]
    fn cat_avatar_has_non_background_pixels() {
        let spec = valid_spec(128, 128, 3);
        let image = render_cat_avatar(spec);
        let background = image.get_pixel(0, 0);

        assert!(image.pixels().any(|pixel| pixel != background));
    }

    #[test]
    fn avatar_identity_uses_sha512_digest() {
        let identity = valid_identity("alice@example.com");

        assert_eq!(identity.as_digest().len(), 64);
        assert_ne!(identity.seed(), 0);
        assert_eq!(&identity.rng_seed(), &identity.as_digest()[32..64]);
    }

    #[test]
    fn rng_seed_uses_second_half_of_identity_digest() {
        let identity = valid_identity("alice@example.com");
        let rng_seed = identity.rng_seed();

        assert_eq!(rng_seed.len(), 32);
        assert_eq!(&rng_seed, &identity.as_digest()[32..64]);
        assert_ne!(&identity.as_digest()[..32], &rng_seed);
    }

    #[test]
    fn avatar_identity_equality_compares_digest_values() {
        let left = valid_identity("alice@example.com");
        let same = valid_identity("alice@example.com");
        let different = valid_identity("bob@example.com");

        assert_eq!(left, same);
        assert_ne!(left, different);
    }

    #[test]
    fn default_identity_options_match_sha512_namespace_constructor() {
        let namespace = valid_namespace("tenant-a", "v2");
        let default = AvatarIdentity::new_with_namespace(namespace, "alice@example.com")
            .expect("sha512 identity should be valid");
        let explicit = AvatarIdentity::new_with_options(
            AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
            "alice@example.com",
        )
        .expect("explicit sha512 identity should be valid");

        assert_eq!(default.as_digest(), explicit.as_digest());
    }

    #[test]
    fn hash_algorithm_parser_round_trips_enabled_algorithms() {
        for algorithm in AvatarHashAlgorithm::ALL {
            assert_eq!(
                algorithm.as_str().parse::<AvatarHashAlgorithm>().ok(),
                Some(algorithm)
            );
        }
    }

    #[test]
    fn oversized_identity_is_rejected_for_every_enabled_hash_algorithm() {
        let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
        for algorithm in AvatarHashAlgorithm::ALL {
            let error = AvatarIdentity::new_with_options(
                AvatarIdentityOptions::new(AvatarNamespace::default(), algorithm),
                &too_long,
            )
            .expect_err("oversized identity should fail");

            assert_eq!(error.component(), AvatarIdentityComponent::Input);
            assert_eq!(error.length(), MAX_AVATAR_ID_BYTES + 1);
            assert_eq!(error.max(), MAX_AVATAR_ID_BYTES);
        }
    }

    #[test]
    fn enabled_hash_algorithms_have_separate_identity_domains() {
        for left in AvatarHashAlgorithm::ALL {
            for right in AvatarHashAlgorithm::ALL {
                if left == right {
                    continue;
                }

                let left_identity = AvatarIdentity::new_with_options(
                    AvatarIdentityOptions::new(AvatarNamespace::default(), left),
                    "alice@example.com",
                )
                .expect("left identity should be valid");
                let right_identity = AvatarIdentity::new_with_options(
                    AvatarIdentityOptions::new(AvatarNamespace::default(), right),
                    "alice@example.com",
                )
                .expect("right identity should be valid");

                assert_ne!(
                    left_identity.as_digest(),
                    right_identity.as_digest(),
                    "{left} and {right} must use separate domains"
                );
            }
        }
    }

    #[cfg(feature = "blake3")]
    #[test]
    fn blake3_identity_mode_renders_avatar() {
        let image = render_avatar_with_identity_options(
            valid_spec(96, 96, 0),
            AvatarIdentityOptions::new(AvatarNamespace::default(), AvatarHashAlgorithm::Blake3),
            "alice@example.com",
            AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
        )
        .expect("blake3-backed avatar should render");

        assert_eq!(image.width(), 96);
        assert_eq!(image.height(), 96);
    }

    #[cfg(feature = "xxh3")]
    #[test]
    fn xxh3_identity_mode_renders_avatar() {
        let image = render_avatar_with_identity_options(
            valid_spec(96, 96, 0),
            AvatarIdentityOptions::new(AvatarNamespace::default(), AvatarHashAlgorithm::Xxh3_128),
            "alice@example.com",
            AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
        )
        .expect("xxh3-backed avatar should render");

        assert_eq!(image.width(), 96);
        assert_eq!(image.height(), 96);
    }

    #[test]
    fn namespace_changes_identity_digest() {
        let left =
            valid_identity_with_namespace(valid_namespace("tenant-a", "v2"), "alice@example.com");
        let right =
            valid_identity_with_namespace(valid_namespace("tenant-b", "v2"), "alice@example.com");

        assert_ne!(left.as_digest(), right.as_digest());
    }

    #[test]
    fn namespace_hashing_is_not_ambiguous_with_nul_bytes() {
        let left =
            valid_identity_with_namespace(valid_namespace("tenant\0v2", "v1"), "alice@example.com");
        let right =
            valid_identity_with_namespace(valid_namespace("tenant", "v2\0v1"), "alice@example.com");

        assert_ne!(left.as_digest(), right.as_digest());
    }

    #[test]
    fn identity_construction_rejects_oversized_input() {
        let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
        let error = AvatarIdentity::new(&too_long).expect_err("oversized identity should fail");

        assert_eq!(error.component(), AvatarIdentityComponent::Input);
        assert_eq!(error.length(), MAX_AVATAR_ID_BYTES + 1);
        assert_eq!(error.max(), MAX_AVATAR_ID_BYTES);
    }

    #[test]
    fn namespace_construction_rejects_oversized_components() {
        let too_long = "a".repeat(MAX_AVATAR_NAMESPACE_COMPONENT_BYTES + 1);
        let error =
            AvatarNamespace::new(&too_long, "v2").expect_err("oversized tenant should fail");

        assert_eq!(error.component(), AvatarIdentityComponent::Tenant);
        assert_eq!(error.length(), MAX_AVATAR_NAMESPACE_COMPONENT_BYTES + 1);
        assert_eq!(error.max(), MAX_AVATAR_NAMESPACE_COMPONENT_BYTES);
    }

    #[test]
    fn render_avatar_for_id_rejects_oversized_identity() {
        let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
        let error = super::render_avatar_for_id(
            valid_spec(128, 128, 0),
            &too_long,
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
        )
        .expect_err("oversized identity should fail");

        assert!(matches!(
            error,
            AvatarRenderError::Identity(AvatarIdentityError {
                component: AvatarIdentityComponent::Input,
                ..
            })
        ));
    }

    #[test]
    fn hashed_cat_avatar_is_deterministic_for_same_id() {
        let spec = valid_spec(192, 192, 0);
        let left = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));
        let right = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));

        assert_eq!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn hashed_cat_avatar_changes_for_different_ids() {
        let spec = valid_spec(192, 192, 0);
        let left = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));
        let right = render_cat_avatar_for_identity(spec, &valid_identity("bob@example.com"));

        assert_ne!(left.as_raw(), right.as_raw());
    }

    #[test]
    fn cat_avatar_webp_export_round_trips() {
        let bytes = encode_cat_avatar(valid_spec(128, 128, 11), AvatarOutputFormat::WebP)
            .expect("webp encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
            .expect("webp should decode");

        assert_eq!(decoded.width(), 128);
        assert_eq!(decoded.height(), 128);
    }

    #[test]
    fn cat_avatar_png_export_round_trips() {
        let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Png)
            .expect("png encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Png)
            .expect("png should decode");

        assert_eq!(decoded.width(), 96);
        assert_eq!(decoded.height(), 96);
    }

    #[test]
    fn cat_avatar_jpeg_export_round_trips() {
        let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Jpeg)
            .expect("jpeg encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg)
            .expect("jpeg should decode");

        assert_eq!(decoded.width(), 96);
        assert_eq!(decoded.height(), 96);
    }

    #[test]
    fn cat_avatar_gif_export_round_trips() {
        let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Gif)
            .expect("gif encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Gif)
            .expect("gif should decode");

        assert_eq!(decoded.width(), 96);
        assert_eq!(decoded.height(), 96);
    }

    #[test]
    fn jpeg_export_flattens_transparency_over_white() {
        let bytes = encode_avatar_for_id(
            valid_spec(96, 96, 0),
            "cat@hashavatar.app",
            AvatarOutputFormat::Jpeg,
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent),
        )
        .expect("jpeg encoding should succeed");
        let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg)
            .expect("jpeg should decode")
            .to_rgb8();

        let corner = decoded.get_pixel(0, 0);
        assert!(corner.0.iter().all(|channel| *channel > 245));
    }

    #[test]
    fn webp_is_the_default_output_format() {
        assert_eq!(AvatarOutputFormat::default(), AvatarOutputFormat::WebP);
    }

    #[test]
    fn hashed_cat_avatar_webp_export_round_trips() {
        let bytes = encode_cat_avatar_for_id(
            valid_spec(128, 128, 0),
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
            valid_spec(128, 128, 0),
            &valid_identity("alice@example.com"),
            AvatarBackground::White,
        );

        assert_eq!(image.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
    }

    #[test]
    fn fixed_background_modes_render_expected_corners() {
        for (background, expected) in [
            (AvatarBackground::Black, Rgba([0, 0, 0, 255])),
            (AvatarBackground::Dark, Rgba([17, 24, 39, 255])),
            (AvatarBackground::Light, Rgba([248, 250, 247, 255])),
        ] {
            let image = render_cat_avatar_for_identity_with_background(
                valid_spec(128, 128, 0),
                &valid_identity("cat@hashavatar.app"),
                background,
            );

            assert_eq!(image.get_pixel(0, 0), &expected, "{background}");
        }
    }

    #[test]
    fn transparent_background_mode_renders_clear_corner() {
        let image = render_cat_avatar_for_identity_with_background(
            valid_spec(128, 128, 0),
            &valid_identity("cat@hashavatar.app"),
            AvatarBackground::Transparent,
        );

        assert_eq!(image.get_pixel(0, 0), &Rgba([255, 255, 255, 0]));
    }

    #[test]
    fn dog_and_robot_variants_generate_distinct_images() {
        let spec = valid_spec(128, 128, 0);
        let id = valid_identity("alice@example.com");
        let dog = render_dog_avatar_for_identity(spec, &id, AvatarBackground::Themed);
        let robot = render_robot_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(dog.as_raw(), robot.as_raw());
    }

    #[test]
    fn monster_variant_is_distinct_from_alien() {
        let spec = valid_spec(128, 128, 0);
        let id = valid_identity("alice@example.com");
        let alien = render_alien_avatar_for_identity(spec, &id, AvatarBackground::Themed);
        let monster = render_monster_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(alien.as_raw(), monster.as_raw());
    }

    #[test]
    fn paws_variant_is_distinct_from_cat() {
        let spec = valid_spec(128, 128, 0);
        let id = valid_identity("alice@example.com");
        let cat =
            render_cat_avatar_for_identity_with_background(spec, &id, AvatarBackground::Themed);
        let paws = render_paws_avatar_for_identity(spec, &id, AvatarBackground::Themed);

        assert_ne!(cat.as_raw(), paws.as_raw());
    }

    #[test]
    fn generic_avatar_encoder_supports_robot_and_white_background() {
        let bytes = encode_avatar_for_id(
            valid_spec(96, 96, 0),
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
            valid_spec(128, 128, 0),
            "vector@example.com",
            AvatarOptions::new(AvatarKind::Fox, AvatarBackground::White),
        );

        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("fox avatar"));
    }

    #[test]
    fn svg_output_is_minimal_and_safe() {
        let svg = render_avatar_svg_for_id(
            valid_spec(256, 256, 0),
            "ghost@example.com",
            AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
        );

        assert!(!svg.contains("<script"));
        assert!(!svg.contains("onload="));
        assert!(svg.len() < 8_000);
    }

    #[test]
    fn transparent_svg_output_has_no_background_rect() {
        let svg = render_avatar_svg_for_id(
            valid_spec(128, 128, 0),
            "cat@hashavatar.app",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent),
        );

        assert!(!svg.contains(r#"<rect width="100%" height="100%""#));
        assert!(svg.contains("cat avatar"));
    }

    #[test]
    fn dark_svg_output_has_background_rect() {
        let svg = render_avatar_svg_for_id(
            valid_spec(128, 128, 0),
            "cat@hashavatar.app",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Dark),
        );

        assert!(svg.contains(r##"<rect width="100%" height="100%" fill="#111827"/>"##));
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
    fn avatar_spec_validation_rejects_resource_extremes() {
        assert!(AvatarSpec::new(MIN_AVATAR_DIMENSION, MIN_AVATAR_DIMENSION, 0).is_ok());
        assert!(AvatarSpec::new(MAX_AVATAR_DIMENSION, MAX_AVATAR_DIMENSION, 0).is_ok());

        let too_small = AvatarSpec::new(MIN_AVATAR_DIMENSION - 1, 256, 0)
            .expect_err("undersized width should be rejected");
        let too_large = AvatarSpec::new(256, MAX_AVATAR_DIMENSION + 1, 0)
            .expect_err("oversized height should be rejected");

        assert_eq!(too_small.width(), MIN_AVATAR_DIMENSION - 1);
        assert_eq!(too_large.height(), MAX_AVATAR_DIMENSION + 1);
    }

    #[test]
    fn rect_edges_saturate_on_extreme_coordinates() {
        let rect = Rect {
            left: i32::MAX,
            top: i32::MAX,
            width: 64,
            height: 64,
        };

        assert_eq!(rect.right(), i32::MAX);
        assert_eq!(rect.bottom(), i32::MAX);
    }

    #[test]
    fn rect_size_builder_clamps_zero_dimensions() {
        let rect = Rect::at(4, 8).of_size(0, 0);

        assert_eq!(rect.width(), 1);
        assert_eq!(rect.height(), 1);
    }

    #[test]
    fn avatar_identity_implements_zeroize() {
        fn assert_zeroize<T: Zeroize>() {}

        assert_zeroize::<AvatarIdentity>();
    }

    #[test]
    fn antialiased_zero_length_line_draws_single_pixel() {
        let mut image = RgbaImage::new(4, 4);

        draw_antialiased_line_segment_mut(
            &mut image,
            (1, 1),
            (1, 1),
            Rgba([10, 20, 30, 255]),
            interpolate,
        );

        assert_eq!(image.get_pixel(1, 1), &Rgba([10, 20, 30, 255]));
    }

    #[test]
    fn polygon_rasterizer_skips_unpaired_intersections() {
        let mut image = RgbaImage::new(32, 32);
        let triangle_with_horizontal_base =
            [Point::new(0, 0), Point::new(16, 16), Point::new(31, 0)];

        draw_polygon_mut(
            &mut image,
            &triangle_with_horizontal_base,
            Rgba([255, 0, 0, 255]),
        );

        assert!(image.pixels().any(|pixel| pixel.0[3] == 255));
    }

    #[test]
    fn ellipse_rasterizer_handles_max_supported_radius() {
        let mut image = RgbaImage::new(1, 1);
        let mut render_calls = 0;

        draw_ellipse(
            |_, _, _, _, _| render_calls += 1,
            &mut image,
            (
                MAX_AVATAR_DIMENSION as i32 / 2,
                MAX_AVATAR_DIMENSION as i32 / 2,
            ),
            MAX_AVATAR_DIMENSION as i32 / 2,
            MAX_AVATAR_DIMENSION as i32 / 2,
        );

        assert!(render_calls > 0);
    }

    #[test]
    fn jpeg_alpha_flattening_uses_wide_intermediates() {
        let image = RgbaImage::from_vec(
            3,
            1,
            vec![
                0, 0, 0, 0, // transparent black over white
                0, 0, 0, 128, // half alpha black over white
                10, 20, 30, 255, // opaque color
            ],
        )
        .expect("test image should be valid");

        let rgb = rgba_to_rgb_over_white(&image);

        assert_eq!(
            rgb,
            vec![
                255, 255, 255, // transparent becomes white
                127, 127, 127, // rounded half-alpha black over white
                10, 20, 30,
            ]
        );
    }

    #[test]
    fn render_avatar_for_id_supports_all_avatar_kinds() {
        let spec = valid_spec(96, 96, 0);
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
    fn render_avatar_svg_for_id_supports_all_avatar_kinds() {
        let spec = valid_spec(96, 96, 0);
        for kind in AvatarKind::ALL {
            let svg = render_avatar_svg_for_id(
                spec,
                "integration@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );

            assert!(svg.contains("<svg"));
            assert!(svg.contains(&format!("{kind} avatar")));
        }
    }

    #[test]
    fn lower_variation_presets_change_for_different_identities() {
        let spec = valid_spec(128, 128, 0);
        for kind in [
            AvatarKind::Ghost,
            AvatarKind::Slime,
            AvatarKind::Wizard,
            AvatarKind::Skull,
        ] {
            let left = render_avatar_for_id(
                spec,
                "alice@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );
            let right = render_avatar_for_id(
                spec,
                "bob@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );

            assert_ne!(
                image_fingerprint(&left),
                image_fingerprint(&right),
                "{kind}"
            );
        }
    }

    #[test]
    fn lower_variation_svg_presets_change_for_different_identities() {
        let spec = valid_spec(128, 128, 0);
        for kind in [
            AvatarKind::Ghost,
            AvatarKind::Slime,
            AvatarKind::Wizard,
            AvatarKind::Skull,
        ] {
            let left = render_avatar_svg_for_id(
                spec,
                "alice@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );
            let right = render_avatar_svg_for_id(
                spec,
                "bob@example.com",
                AvatarOptions::new(kind, AvatarBackground::Themed),
            );

            assert_ne!(left, right, "{kind}");
        }
    }

    #[test]
    fn visual_fingerprints_are_stable() {
        for (label, options) in regression_scenarios() {
            let image =
                render_avatar_for_id(valid_spec(128, 128, 0), "snapshot@example.com", options);
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
            (
                "planet-themed",
                AvatarOptions::new(AvatarKind::Planet, AvatarBackground::Themed),
            ),
            (
                "rocket-themed",
                AvatarOptions::new(AvatarKind::Rocket, AvatarBackground::Themed),
            ),
            (
                "mushroom-themed",
                AvatarOptions::new(AvatarKind::Mushroom, AvatarBackground::Themed),
            ),
            (
                "cactus-themed",
                AvatarOptions::new(AvatarKind::Cactus, AvatarBackground::Themed),
            ),
            (
                "frog-themed",
                AvatarOptions::new(AvatarKind::Frog, AvatarBackground::Themed),
            ),
            (
                "panda-themed",
                AvatarOptions::new(AvatarKind::Panda, AvatarBackground::Themed),
            ),
            (
                "cupcake-themed",
                AvatarOptions::new(AvatarKind::Cupcake, AvatarBackground::Themed),
            ),
            (
                "pizza-themed",
                AvatarOptions::new(AvatarKind::Pizza, AvatarBackground::Themed),
            ),
            (
                "icecream-themed",
                AvatarOptions::new(AvatarKind::Icecream, AvatarBackground::Themed),
            ),
            (
                "octopus-themed",
                AvatarOptions::new(AvatarKind::Octopus, AvatarBackground::Themed),
            ),
            (
                "knight-themed",
                AvatarOptions::new(AvatarKind::Knight, AvatarBackground::Themed),
            ),
        ] {
            let image =
                render_avatar_for_id(valid_spec(128, 128, 0), "snapshot@example.com", options);
            println!("{label}: {}", image_fingerprint(&image));
        }
    }

    fn regression_scenarios() -> [(&'static str, AvatarOptions); 22] {
        [
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
            (
                "planet-themed",
                AvatarOptions::new(AvatarKind::Planet, AvatarBackground::Themed),
            ),
            (
                "rocket-themed",
                AvatarOptions::new(AvatarKind::Rocket, AvatarBackground::Themed),
            ),
            (
                "mushroom-themed",
                AvatarOptions::new(AvatarKind::Mushroom, AvatarBackground::Themed),
            ),
            (
                "cactus-themed",
                AvatarOptions::new(AvatarKind::Cactus, AvatarBackground::Themed),
            ),
            (
                "frog-themed",
                AvatarOptions::new(AvatarKind::Frog, AvatarBackground::Themed),
            ),
            (
                "panda-themed",
                AvatarOptions::new(AvatarKind::Panda, AvatarBackground::Themed),
            ),
            (
                "cupcake-themed",
                AvatarOptions::new(AvatarKind::Cupcake, AvatarBackground::Themed),
            ),
            (
                "pizza-themed",
                AvatarOptions::new(AvatarKind::Pizza, AvatarBackground::Themed),
            ),
            (
                "icecream-themed",
                AvatarOptions::new(AvatarKind::Icecream, AvatarBackground::Themed),
            ),
            (
                "octopus-themed",
                AvatarOptions::new(AvatarKind::Octopus, AvatarBackground::Themed),
            ),
            (
                "knight-themed",
                AvatarOptions::new(AvatarKind::Knight, AvatarBackground::Themed),
            ),
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
