//! Canonical rendering core for the Hashavatar 2.0 development line.
//!
//! Alpha.4 adds bounded typed palettes, expressions, and multi-accessory
//! composition to the complete catalog. Identity-derived traits are stateless,
//! geometry and scene layouts stay private, and CPU RGBA8 and SVG output execute
//! the same validated scene.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod art;
mod avatar;
mod cat;
mod catalog;
mod error;
mod fixed;
mod geometry;
mod identity;
mod layout;
mod paint;
mod raster;
mod rasterize;
mod scene;
mod style;
mod svg;

#[cfg(kani)]
mod kani_proofs;

pub use self::cat::{CatRequest, CatTraitVector, PreparedCat};
pub use self::catalog::{
    AVATAR_FAMILY_CAPABILITIES, AvatarBackground, AvatarFamilyCapabilities,
    AvatarFamilyCapabilityEntry, AvatarKind, AvatarShape,
};
pub use self::error::{CatError, IdentityComponent};
pub use self::layout::{
    AccessoryLayoutDecision, AvatarAnchorPoint, AvatarAnchorSet, AvatarZBand,
    ExpressionLayoutDecision, LayoutDisposition, LayoutReport, ResolvedStyle,
};
pub use self::raster::{CanonicalRgbaImage, PixelDigest, RgbaSurfaceMut};
pub use self::scene::SceneReport;
pub use self::style::{
    AccessoryStack, AvatarAccessory, AvatarAccessorySlot, AvatarColorRoles, AvatarExpression,
    AvatarPalette, AvatarRgb, AvatarStyle, MAX_ACCESSORY_LAYERS, StyleResolutionPolicy,
};
pub use self::svg::{SvgMode, SvgOptions};

/// Error returned by canonical avatar preparation and execution.
pub type AvatarError = CatError;

/// Smallest supported canonical output dimension.
pub const MIN_DIMENSION: u32 = 64;

/// Largest supported canonical output dimension.
pub const MAX_DIMENSION: u32 = 2048;

/// Largest accepted raw identity input in bytes.
pub const MAX_IDENTITY_BYTES: usize = 1024;

/// Largest accepted tenant or style-version component in bytes.
pub const MAX_NAMESPACE_COMPONENT_BYTES: usize = 128;

/// Number of bytes in one external straight-alpha RGBA8 pixel.
pub const RGBA8_BYTES_PER_PIXEL: usize = 4;

/// Versioned contract identifier included in canonical pixel digests.
pub const PIXEL_CONTRACT_ID: &str = "hashavatar/rgba8-straight-srgb/v1";

/// Source-only alpha.4 catalog contract identifier.
pub const CATALOG_CONTRACT_ID: &str = "hashavatar/catalog/v2-alpha4";
pub use self::avatar::{AvatarRequest, AvatarTraitVector, PreparedAvatar};
