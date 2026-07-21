use std::{fmt::Write as _, string::String};

use sanitization::Secret;
use sanitization_crypto_interop::sha2::SanitizedSha512;

use crate::{AvatarOutputFormat, FormatError};

const SEMANTIC_KEY_DOMAIN: &[u8] = b"hashavatar/encoded-semantic-key/v2/sha512/v1";
const BUILD_KEY_DOMAIN: &[u8] = b"hashavatar/encoded-build-key/v2/sha512/v1";

macro_rules! define_public_key {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[must_use = "encoded keys identify cache and storage entries"]
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Borrows the 32 key bytes.
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            /// Serializes this public key as lowercase hexadecimal.
            pub fn to_hex(self) -> String {
                let mut output = String::new();
                for byte in self.0 {
                    let _ = write!(output, "{byte:02x}");
                }
                output
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.to_hex())
                    .finish()
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(&self.to_hex())
            }
        }
    };
}

define_public_key!(
    SemanticEncodedAssetKey,
    "Public key binding a canonical avatar asset to one format/settings contract."
);
define_public_key!(
    BuildEncodedAssetKey,
    "Public encoded-asset key additionally bound to a caller-supplied encoder build ID."
);
define_public_key!(
    EncoderBuildId,
    "Caller-supplied digest identifying one deployed encoder build."
);

impl EncoderBuildId {
    /// Creates an ID from a caller-computed deployment digest.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

pub(crate) fn semantic_key(
    asset_key: hashavatar_core::AvatarAssetKey,
    format: AvatarOutputFormat,
) -> Result<SemanticEncodedAssetKey, FormatError> {
    derive_key(
        SEMANTIC_KEY_DOMAIN,
        &[
            asset_key.as_bytes(),
            format.encoder_contract_id().as_bytes(),
        ],
    )
    .map(SemanticEncodedAssetKey)
}

pub(crate) fn build_key(
    semantic: SemanticEncodedAssetKey,
    build_id: EncoderBuildId,
) -> Result<BuildEncodedAssetKey, FormatError> {
    derive_key(
        BUILD_KEY_DOMAIN,
        &[semantic.as_bytes(), build_id.as_bytes()],
    )
    .map(BuildEncodedAssetKey)
}

fn derive_key(domain: &[u8], components: &[&[u8]]) -> Result<[u8; 32], FormatError> {
    let mut hasher = SanitizedSha512::new();
    update_component(&mut hasher, domain)?;
    for component in components {
        update_component(&mut hasher, component)?;
    }
    let digest = Secret::new(hasher.finalize());
    Ok(digest.with_secret(|digest| {
        let mut key = [0_u8; 32];
        for (destination, source) in key.iter_mut().zip(digest.iter()) {
            *destination = *source;
        }
        key
    }))
}

fn update_component(hasher: &mut SanitizedSha512, bytes: &[u8]) -> Result<(), FormatError> {
    let length = u64::try_from(bytes.len()).map_err(|_| FormatError::Allocation)?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}
