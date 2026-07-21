//! Security-oriented facade for canonical Hashavatar rendering.
//!
//! `2.0.0-alpha.4` is a source-only development release. It adds bounded typed
//! layered styles to the canonical catalog renderer.
//! The maintained crates.io line remains `1.3.x`.
//!
//! # Example
//!
//! ```
//! use hashavatar::{AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle};
//!
//! let style = AvatarStyle::new(
//!     AvatarKind::Robot,
//!     AvatarBackground::Ocean,
//!     AvatarShape::Circle,
//! );
//! let prepared = AvatarRequest::new(256, 256, 0, b"user@example.invalid", style)?.prepare()?;
//! let rgba = prepared.render_rgba()?;
//! let svg = prepared.render_svg()?;
//!
//! assert_eq!(rgba.dimensions(), (256, 256));
//! assert!(svg.starts_with("<svg"));
//! # Ok::<(), hashavatar::CatError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]

pub use hashavatar_core::{
    AVATAR_FAMILY_CAPABILITIES, AccessoryLayoutDecision, AccessoryStack, AvatarAccessory,
    AvatarAccessorySlot, AvatarAnchorPoint, AvatarAnchorSet, AvatarBackground, AvatarColorRoles,
    AvatarError, AvatarExpression, AvatarFamilyCapabilities, AvatarFamilyCapabilityEntry,
    AvatarKind, AvatarPalette, AvatarRequest, AvatarRgb, AvatarShape, AvatarStyle,
    AvatarTraitVector, AvatarZBand, CATALOG_CONTRACT_ID, CanonicalRgbaImage, CatError, CatRequest,
    CatTraitVector, ExpressionLayoutDecision, IdentityComponent, LayoutDisposition, LayoutReport,
    MAX_ACCESSORY_LAYERS, MAX_DIMENSION, MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES,
    MIN_DIMENSION, PIXEL_CONTRACT_ID, PixelDigest, PreparedAvatar, PreparedCat,
    RGBA8_BYTES_PER_PIXEL, ResolvedStyle, RgbaSurfaceMut, SceneReport, StyleResolutionPolicy,
    SvgMode, SvgOptions,
};

/// Common alpha.4 facade imports.
pub mod prelude {
    pub use crate::{
        AccessoryStack, AvatarAccessory, AvatarBackground, AvatarError, AvatarExpression,
        AvatarKind, AvatarPalette, AvatarRequest, AvatarShape, AvatarStyle, CanonicalRgbaImage,
        PreparedAvatar, RgbaSurfaceMut, StyleResolutionPolicy, SvgOptions,
    };
}
