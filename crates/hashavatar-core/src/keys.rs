use alloc::string::String;
use core::fmt::Write;

use sanitization::Secret;
use sanitization_crypto_interop::sha2::SanitizedSha512;

use crate::{
    AvatarError, AvatarIdentity, CATALOG_CONTRACT_ID, PIXEL_CONTRACT_ID, RENDER_CONTRACT_ID,
    ResolvedStyle,
};

const IDENTITY_CACHE_KEY_DOMAIN: &[u8] = b"hashavatar/identity-cache-key/v2/sha512/v1";
const AVATAR_ASSET_KEY_DOMAIN: &[u8] = b"hashavatar/avatar-asset-key/v2/sha512/v1";

/// Identifier for the active built-in catalog contract.
#[must_use = "bind catalog versions into persistent request metadata"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CatalogVersion;

impl CatalogVersion {
    /// Active catalog version.
    pub const CURRENT: Self = Self;

    /// Returns the immutable catalog contract identifier.
    pub const fn as_str(self) -> &'static str {
        CATALOG_CONTRACT_ID
    }
}

/// Identifier for the active canonical renderer contract.
#[must_use = "bind render contracts into persistent request metadata"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderContractId;

impl RenderContractId {
    /// Active canonical render contract.
    pub const CURRENT: Self = Self;

    /// Returns the immutable render contract identifier.
    pub const fn as_str(self) -> &'static str {
        RENDER_CONTRACT_ID
    }
}

macro_rules! define_public_key {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[must_use = "asset keys identify cache and storage entries"]
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
    IdentityCacheKey,
    "Public domain-separated cache key for one validated avatar identity."
);
define_public_key!(
    AvatarAssetKey,
    "Public domain-separated key for one complete canonical avatar render tuple."
);

pub(crate) fn identity_cache_key(
    identity: &AvatarIdentity,
) -> Result<IdentityCacheKey, AvatarError> {
    identity.with_digest(|digest| {
        derive_key(IDENTITY_CACHE_KEY_DOMAIN, &[b"sha512", digest]).map(IdentityCacheKey)
    })
}

pub(crate) fn avatar_asset_key(
    identity: &AvatarIdentity,
    width: u32,
    height: u32,
    style_seed: u64,
    style: ResolvedStyle,
) -> Result<AvatarAssetKey, AvatarError> {
    let identity_key = identity_cache_key(identity)?;
    let width = width.to_le_bytes();
    let height = height.to_le_bytes();
    let seed = style_seed.to_le_bytes();
    let kind = style.kind().catalog_id().to_le_bytes();
    let background = style.background().catalog_id().to_le_bytes();
    let shape = style.shape().catalog_id().to_le_bytes();
    let palette = [style.palette().catalog_id()];
    let expression = [style.expression().catalog_id()];
    let mut hasher = SanitizedSha512::new();
    for component in [
        AVATAR_ASSET_KEY_DOMAIN,
        identity_key.as_bytes(),
        CatalogVersion::CURRENT.as_str().as_bytes(),
        RenderContractId::CURRENT.as_str().as_bytes(),
        PIXEL_CONTRACT_ID.as_bytes(),
        &width,
        &height,
        &seed,
        &kind,
        &background,
        &shape,
        &palette,
        &expression,
    ] {
        update_component(&mut hasher, component)?;
    }
    for accessory in style.accessories().iter() {
        update_component(&mut hasher, &accessory.catalog_id().to_le_bytes())?;
    }
    let digest = Secret::new(hasher.finalize());
    Ok(AvatarAssetKey(digest.with_secret(first_32)))
}

fn derive_key(domain: &[u8], components: &[&[u8]]) -> Result<[u8; 32], AvatarError> {
    let mut hasher = SanitizedSha512::new();
    update_component(&mut hasher, domain)?;
    for component in components {
        update_component(&mut hasher, component)?;
    }
    let digest = Secret::new(hasher.finalize());
    Ok(digest.with_secret(first_32))
}

fn update_component(hasher: &mut SanitizedSha512, bytes: &[u8]) -> Result<(), AvatarError> {
    let length = u64::try_from(bytes.len()).map_err(|_| AvatarError::NumericRange)?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn first_32(digest: &[u8; 64]) -> [u8; 32] {
    let mut output = [0_u8; 32];
    for (destination, source) in output.iter_mut().zip(digest.iter()) {
        *destination = *source;
    }
    output
}
