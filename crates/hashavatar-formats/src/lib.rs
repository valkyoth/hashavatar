//! Feature-gated established image encoders for canonical Hashavatar output.
//!
//! The crate owns codec dependencies, settings, metadata, and encoded keys.
//! Canonical rendering remains in `hashavatar-core`.

#![forbid(unsafe_code)]

mod encode;
mod error;
mod format;
mod keys;
mod output;
mod writer;

pub use self::{
    encode::{encode, encode_to_writer, encode_to_writer_with_scratch},
    error::FormatError,
    format::{AlphaSupport, AvatarOutputFormat},
    keys::{BuildEncodedAssetKey, EncoderBuildId, SemanticEncodedAssetKey},
    output::{EncodedAvatar, EncodedAvatarMetadata, FormatResourceBudget},
};
