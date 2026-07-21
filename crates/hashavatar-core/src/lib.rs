//! Canonical rendering core for the Hashavatar 2.0 development line.
//!
//! Alpha.1 intentionally exposes only a Cat vertical slice. Identity-derived
//! traits are stateless, geometry uses private fixed-point values, and both CPU
//! RGBA8 and SVG output execute the same validated private scene.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod cat;
mod error;
mod fixed;
mod identity;
mod raster;
mod scene;
mod svg;

#[cfg(kani)]
mod kani_proofs;

pub use self::cat::{CatRequest, CatTraitVector, PreparedCat};
pub use self::error::{CatError, IdentityComponent};
pub use self::raster::CanonicalRgbaImage;
pub use self::scene::SceneReport;

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
