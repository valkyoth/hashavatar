//! Security-oriented facade for canonical Hashavatar rendering.
//!
//! `2.0.0-alpha.5` is a source-only development release. It adds the isolated
//! established-format boundary and recommended prepared-request workflow.
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
//! # Ok::<(), hashavatar::AvatarError>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]

pub use hashavatar_core::{
    AVATAR_FAMILY_CAPABILITIES, AccessoryLayoutDecision, AccessoryStack, AvatarAccessory,
    AvatarAccessorySlot, AvatarAnchorPoint, AvatarAnchorSet, AvatarAssetKey, AvatarBackground,
    AvatarColorRoles, AvatarError, AvatarExpression, AvatarFamilyCapabilities,
    AvatarFamilyCapabilityEntry, AvatarIdentity, AvatarKind, AvatarPalette, AvatarRequest,
    AvatarRequestBuilder, AvatarRgb, AvatarShape, AvatarStyle, AvatarTraitVector, AvatarZBand,
    CATALOG_CONTRACT_ID, CanonicalRgbaImage, CatError, CatRequest, CatTraitVector, CatalogVersion,
    DEFAULT_STYLE_VERSION, DEFAULT_TENANT, ExpressionLayoutDecision, IdentityCacheKey,
    IdentityComponent, LayoutDisposition, LayoutReport, MAX_ACCESSORY_LAYERS, MAX_DIMENSION,
    MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES, MAX_SVG_OUTPUT_BYTES, MIN_DIMENSION,
    PIXEL_CONTRACT_ID, PixelDigest, PreparedAvatar, PreparedCat, RENDER_CONTRACT_ID,
    RGBA8_BYTES_PER_PIXEL, RenderContractId, ResolvedStyle, ResourceBudget, ReusableRgbaBuffer,
    RgbaSurfaceMut, SceneReport, StyleResolutionPolicy, SvgMode, SvgOptions,
};

/// Feature-gated established image encoding APIs.
#[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
pub mod formats {
    pub use hashavatar_formats::*;
}

/// Common alpha.5 facade imports.
pub mod prelude {
    pub use crate::{
        AccessoryStack, AvatarAccessory, AvatarBackground, AvatarError, AvatarExpression,
        AvatarIdentity, AvatarKind, AvatarPalette, AvatarRequest, AvatarShape, AvatarStyle,
        CanonicalRgbaImage, PreparedAvatar, ReusableRgbaBuffer, RgbaSurfaceMut,
        StyleResolutionPolicy, SvgOptions,
    };

    #[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
    pub use crate::formats::{AvatarOutputFormat, EncodedAvatar, FormatError, encode};
}
