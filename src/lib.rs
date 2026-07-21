//! Security-oriented facade for canonical Hashavatar rendering.
//!
//! `2.0.0-alpha.1` is a source-only development release. It intentionally
//! exposes one complete Cat vertical slice while the 2.0 contracts are
//! validated. The maintained crates.io line remains `1.3.x`.
//!
//! # Example
//!
//! ```
//! use hashavatar::CatRequest;
//!
//! let prepared = CatRequest::new(256, 256, 0, b"user@example.invalid")?
//!     .prepare()?;
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
    CanonicalRgbaImage, CatError, CatRequest, CatTraitVector, IdentityComponent, MAX_DIMENSION,
    MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES, MIN_DIMENSION, PreparedCat,
    RGBA8_BYTES_PER_PIXEL, SceneReport,
};

/// Common alpha.1 facade imports.
pub mod prelude {
    pub use crate::{CanonicalRgbaImage, CatError, CatRequest, CatTraitVector, PreparedCat};
}
