//! Procedural, asset-free avatar generation driven by stable identity hashes.
//!
//! The crate produces deterministic avatar images from an input identifier
//! without shipping image packs, sprites, or third-party artwork. All visual
//! output is drawn from code using geometric primitives.
//!
//! Typical usage:
//! ```no_run
//! use hashavatar::prelude::*;
//!
//! let bytes = AvatarBuilder::for_id("robot@hashavatar.app")
//!     .size(256, 256)
//!     .kind(AvatarKind::Robot)
//!     .background(AvatarBackground::Transparent)
//!     .encode(AvatarOutputFormat::WebP)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]

#[cfg(all(feature = "blake3", feature = "xxh3"))]
compile_error!(
    "hashavatar features `blake3` and `xxh3` are mutually exclusive; choose one identity hash mode"
);

#[cfg(all(feature = "fuzzing", not(any(debug_assertions, fuzzing))))]
compile_error!(
    "hashavatar's fuzzing feature exposes internal fuzz harness entry points and must not be enabled in non-fuzzing release builds"
);

use std::mem::swap;
use std::str::FromStr;

#[cfg(feature = "gif")]
use image::codecs::gif::GifEncoder;
#[cfg(feature = "jpeg")]
use image::codecs::jpeg::JpegEncoder;
#[cfg(feature = "png")]
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::error::{LimitError, LimitErrorKind};
use image::{
    ExtendedColorType, ImageBuffer, ImageEncoder, ImageError, ImageResult, Rgba, RgbaImage,
};
use palette::{FromColor, Hsl, Srgb};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use sanitization::unsafe_wipe::volatile_sanitize_vec;
use sanitization::{Secret, SecretVec, SecureSanitize, sanitize_bytes};
#[cfg(feature = "blake3")]
use sanitization_crypto_interop::blake3::blake3_xof_fill;
use sanitization_crypto_interop::sha2::sha512_digest as sanitized_sha512_digest;
use subtle::ConstantTimeEq;

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

/// Number of bytes in one RGBA8 raster pixel.
pub const AVATAR_RGBA_BYTES_PER_PIXEL: usize = 4;

/// Largest supported raster pixel count.
pub const MAX_AVATAR_PIXELS: usize =
    (MAX_AVATAR_DIMENSION as usize) * (MAX_AVATAR_DIMENSION as usize);

/// Largest supported raw RGBA8 image buffer size before encoder overhead.
///
/// Services should combine this bound with their own request concurrency
/// limits. The crate bounds each render request, but it cannot prevent process
/// memory pressure from many concurrent maximum-size renders.
pub const MAX_AVATAR_RGBA_BYTES: usize = MAX_AVATAR_PIXELS * AVATAR_RGBA_BYTES_PER_PIXEL;

/// Largest supported identity input in bytes.
///
/// This prevents applications from accidentally hashing attacker-controlled
/// request bodies or other unbounded byte strings as avatar identities.
pub const MAX_AVATAR_ID_BYTES: usize = 1024;

/// Largest supported namespace component in bytes.
pub const MAX_AVATAR_NAMESPACE_COMPONENT_BYTES: usize = 128;

/// Identity digest byte used for automatic avatar family selection.
pub const AVATAR_STYLE_KIND_BYTE: usize = 0;

/// Identity digest byte used for automatic background selection.
pub const AVATAR_STYLE_BACKGROUND_BYTE: usize = 1;

/// Identity digest byte used for automatic accessory selection.
pub const AVATAR_STYLE_ACCESSORY_BYTE: usize = 2;

/// Identity digest byte used for automatic color-palette selection.
pub const AVATAR_STYLE_COLOR_BYTE: usize = 3;

/// Identity digest byte used for automatic expression selection.
pub const AVATAR_STYLE_EXPRESSION_BYTE: usize = 4;

/// Identity digest byte used for automatic frame-shape selection.
pub const AVATAR_STYLE_SHAPE_BYTE: usize = 5;

/// Common imports for application code using the high-level avatar APIs.
pub mod prelude {
    pub use crate::{
        AvatarAccessory, AvatarBackground, AvatarBuilder, AvatarColor, AvatarError,
        AvatarExpression, AvatarIdentity, AvatarIdentityOptions, AvatarKind, AvatarNamespace,
        AvatarOptions, AvatarOutputFormat, AvatarShape, AvatarSpec, AvatarStyleOptions,
    };
}

const HASH_DOMAIN: &[u8] = b"hashavatar";
const HASH_DOMAIN_ALGORITHM_COMPONENT: &[u8] = b"identity-hash";
const CACHE_KEY_DOMAIN: &[u8] = b"hashavatar-cache-key-v1";
#[cfg(feature = "blake3")]
const ACTIVE_HASH_ALGORITHM_LABEL: &[u8] = b"blake3";
#[cfg(all(not(feature = "blake3"), feature = "xxh3"))]
const ACTIVE_HASH_ALGORITHM_LABEL: &[u8] = b"xxh3-128";
#[cfg(all(not(feature = "blake3"), not(feature = "xxh3")))]
const ACTIVE_HASH_ALGORITHM_LABEL: &[u8] = b"sha512";
#[cfg(feature = "xxh3")]
const HASH_XOF_CHUNK_COMPONENT: &[u8] = b"digest-chunk";

mod api;
mod avatars;
mod backgrounds;
mod cat_support;
mod core;
mod encoding;
mod layers;
mod model;
mod primitives;
mod renderer_types;
mod svg;

#[cfg(kani)]
mod kani_proofs;

#[cfg(test)]
mod tests;

pub(crate) use self::backgrounds::*;
pub(crate) use self::cat_support::*;
pub(crate) use self::encoding::*;
pub(crate) use self::layers::*;
pub(crate) use self::primitives::*;
pub(crate) use self::svg::*;

pub use self::api::*;
pub use self::avatars::*;
pub use self::core::*;
pub use self::model::*;
pub use self::primitives::Color;
#[cfg(feature = "fuzzing")]
pub use self::primitives::fuzz_draw_polygon_rgba;
pub use self::renderer_types::*;
