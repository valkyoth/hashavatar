//! Canonical rendering core for the Hashavatar 2.0 development line.
//!
//! Alpha.5 adds owned prepared requests, public cache keys, resource budgets,
//! and reusable caller-controlled raster storage to the complete canonical
//! renderer. Identity-derived traits are stateless, geometry and scene layouts
//! stay private, and CPU RGBA8 and SVG execute the same validated scene.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod art;
mod avatar;
mod budget;
mod cat;
mod catalog;
mod error;
mod fixed;
mod geometry;
mod identity;
mod keys;
mod layout;
mod paint;
mod raster;
mod rasterize;
mod scene;
mod scratch;
mod style;
mod svg;

#[cfg(kani)]
mod kani_proofs;

pub use self::budget::ResourceBudget;
pub use self::cat::{CatRequest, CatTraitVector, PreparedCat};
pub use self::catalog::{
    AVATAR_FAMILY_CAPABILITIES, AvatarBackground, AvatarFamilyCapabilities,
    AvatarFamilyCapabilityEntry, AvatarKind, AvatarShape,
};
pub use self::error::{AvatarError, CatError, IdentityComponent};
pub use self::identity::AvatarIdentity;
pub use self::keys::{AvatarAssetKey, CatalogVersion, IdentityCacheKey, RenderContractId};
pub use self::layout::{
    AccessoryLayoutDecision, AvatarAnchorPoint, AvatarAnchorSet, AvatarZBand,
    ExpressionLayoutDecision, LayoutDisposition, LayoutReport, ResolvedStyle,
};
pub use self::raster::{CanonicalRgbaImage, PixelDigest, RgbaSurfaceMut};
pub use self::scene::SceneReport;
pub use self::scratch::ReusableRgbaBuffer;
pub use self::style::{
    AccessoryStack, AvatarAccessory, AvatarAccessorySlot, AvatarColorRoles, AvatarExpression,
    AvatarPalette, AvatarRgb, AvatarStyle, MAX_ACCESSORY_LAYERS, StyleResolutionPolicy,
};
pub use self::svg::{SvgMode, SvgOptions};

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

/// Maximum owned SVG output reservation and emitted length.
pub const MAX_SVG_OUTPUT_BYTES: usize = 64 * 1024;

/// Versioned contract identifier included in canonical pixel digests.
pub const PIXEL_CONTRACT_ID: &str = "hashavatar/rgba8-straight-srgb/v1";

/// Canonical CPU/SVG render contract used by alpha.5.
pub const RENDER_CONTRACT_ID: &str = "hashavatar/render/v2-alpha3";

/// Source-only catalog contract identifier retained from alpha.4.
pub const CATALOG_CONTRACT_ID: &str = "hashavatar/catalog/v2-alpha4";
/// Default public tenant namespace.
pub const DEFAULT_TENANT: &[u8] = b"public";
/// Default style namespace retained to preserve layer-free alpha.3 pixels.
pub const DEFAULT_STYLE_VERSION: &[u8] = b"v2-alpha3";
pub use self::avatar::{AvatarRequest, AvatarRequestBuilder, AvatarTraitVector, PreparedAvatar};
