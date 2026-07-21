//! Security-oriented facade for canonical Hashavatar rendering.
//!
//! `2.0.0-alpha.3` is a source-only development release. It ports the existing
//! family, background, and frame catalog onto the canonical scene renderer.
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
    AVATAR_FAMILY_CAPABILITIES, AvatarBackground, AvatarError, AvatarFamilyCapabilities,
    AvatarFamilyCapabilityEntry, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    AvatarTraitVector, CATALOG_CONTRACT_ID, CanonicalRgbaImage, CatError, CatRequest,
    CatTraitVector, IdentityComponent, MAX_DIMENSION, MAX_IDENTITY_BYTES,
    MAX_NAMESPACE_COMPONENT_BYTES, MIN_DIMENSION, PIXEL_CONTRACT_ID, PixelDigest, PreparedAvatar,
    PreparedCat, RGBA8_BYTES_PER_PIXEL, RgbaSurfaceMut, SceneReport, SvgMode, SvgOptions,
};

/// Common alpha.3 facade imports.
pub mod prelude {
    pub use crate::{
        AvatarBackground, AvatarError, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
        CanonicalRgbaImage, PreparedAvatar, RgbaSurfaceMut, SvgOptions,
    };
}
