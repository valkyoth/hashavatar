use super::*;

const IDENTITY_CACHE_KEY_DOMAIN: &[u8] = b"hashavatar-identity-cache-key-v1";
const AVATAR_ASSET_KEY_DOMAIN: &[u8] = b"hashavatar-avatar-asset-key-v1";
const ENCODED_ASSET_KEY_DOMAIN: &[u8] = b"hashavatar-encoded-asset-key-v1";
const ENCODED_BUILD_ASSET_KEY_DOMAIN: &[u8] = b"hashavatar-encoded-build-asset-key-v1";

macro_rules! define_asset_key {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Eq, Hash, PartialEq)]
        pub struct $name([u8; 32]);

        impl $name {
            pub const fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            pub fn to_hex(self) -> String {
                hex_lower(&self.0)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.to_hex())
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.to_hex())
            }
        }
    };
}

define_asset_key!(
    IdentityCacheKey,
    "Domain-separated public cache key for one derived avatar identity."
);
define_asset_key!(
    AvatarAssetKey,
    "Domain-separated key for one complete unencoded avatar render tuple."
);
define_asset_key!(
    EncodedAssetKey,
    "Domain-separated key for one encoded avatar asset tuple."
);
define_asset_key!(
    EncoderBuildId,
    "Caller-supplied digest identifying one deployed encoder build."
);

impl EncoderBuildId {
    /// Creates an encoder build identifier from a caller-computed digest.
    ///
    /// Applications should derive this value from deployment inputs that can
    /// affect encoded bytes, such as the resolved lockfile, target, relevant
    /// build flags, and application build revision.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

fn derive_key(domain: &[u8], components: &[&[u8]]) -> [u8; 32] {
    let expected_capacity = components
        .iter()
        .fold(key_component_size(domain), |capacity, component| {
            capacity.saturating_add(key_component_size(component))
        });
    let mut preimage = SecretVec::with_capacity(expected_capacity);
    update_hash_input_component(&mut preimage, domain);
    for component in components {
        update_hash_input_component(&mut preimage, component);
    }
    debug_assert_eq!(
        (preimage.capacity(), preimage.len()),
        (expected_capacity, expected_capacity),
        "asset-key preimage size accounting drifted"
    );

    let digest = Secret::new(preimage.with_secret(sanitized_sha512_digest));
    digest.with_secret(|digest| {
        let mut key = [0_u8; 32];
        key.copy_from_slice(&digest[..32]);
        key
    })
}

const fn key_component_size(bytes: &[u8]) -> usize {
    std::mem::size_of::<u64>().saturating_add(bytes.len())
}

impl AvatarIdentity {
    /// Returns a typed cache key that includes the active hash mode.
    ///
    /// Like [`AvatarIdentity::cache_key`], this value is public and
    /// correlatable. It is not an authentication secret.
    pub fn identity_cache_key(&self) -> IdentityCacheKey {
        IdentityCacheKey(derive_key(
            IDENTITY_CACHE_KEY_DOMAIN,
            &[ACTIVE_HASH_ALGORITHM_LABEL, &self.digest],
        ))
    }

    /// Derives a complete key for an unencoded avatar render.
    pub fn avatar_asset_key(&self, spec: AvatarSpec, style: AvatarStyleOptions) -> AvatarAssetKey {
        let style = style.canonicalized_for_family();
        let identity_key = self.identity_cache_key();
        let width = spec.width().to_le_bytes();
        let height = spec.height().to_le_bytes();
        let seed = spec.seed().to_le_bytes();
        let kind = style.kind.legacy_catalog_id().to_le_bytes();
        let background = style.background.legacy_catalog_id().to_le_bytes();
        let accessory = style.accessory.legacy_catalog_id().to_le_bytes();
        let color = style.color.legacy_catalog_id().to_le_bytes();
        let expression = style.expression.legacy_catalog_id().to_le_bytes();
        let shape = style.shape.legacy_catalog_id().to_le_bytes();
        AvatarAssetKey(derive_key(
            AVATAR_ASSET_KEY_DOMAIN,
            &[
                identity_key.as_bytes(),
                CatalogVersion::CURRENT.as_str().as_bytes(),
                RenderContractId::CURRENT.as_str().as_bytes(),
                &width,
                &height,
                &seed,
                &kind,
                &background,
                &accessory,
                &color,
                &expression,
                &shape,
            ],
        ))
    }
}

impl AvatarOutputFormat {
    /// Frozen ID for this encoder configuration.
    pub const fn encoder_contract_id(self) -> &'static str {
        match self {
            Self::WebP => "hashavatar-webp-lossless-v1",
            #[cfg(feature = "png")]
            Self::Png => "hashavatar-png-best-adaptive-v1",
            #[cfg(feature = "jpeg")]
            Self::Jpeg => "hashavatar-jpeg-q92-white-v1",
            #[cfg(feature = "gif")]
            Self::Gif => "hashavatar-gif-default-v1",
        }
    }
}

impl AvatarAssetKey {
    /// Derives a semantic request key for an encoded avatar.
    ///
    /// This binds the format and Hashavatar's fixed encoder settings, but not
    /// dependency versions, compilation target, or deployment build. Use
    /// [`AvatarAssetKey::encoded_for_build`] when encoded bytes from different
    /// deployments must never share a cache entry.
    pub fn encoded(self, format: AvatarOutputFormat) -> EncodedAssetKey {
        EncodedAssetKey(derive_key(
            ENCODED_ASSET_KEY_DOMAIN,
            &[self.as_bytes(), format.encoder_contract_id().as_bytes()],
        ))
    }

    /// Derives a deployment-specific encoded asset key.
    ///
    /// The caller controls `build_id` and must change it whenever any encoder
    /// implementation detail that can affect output bytes changes. For true
    /// content-addressable storage, hash the encoded bytes themselves after
    /// encoding instead of treating this predictive key as a content digest.
    pub fn encoded_for_build(
        self,
        format: AvatarOutputFormat,
        build_id: EncoderBuildId,
    ) -> EncodedAssetKey {
        EncodedAssetKey(derive_key(
            ENCODED_BUILD_ASSET_KEY_DOMAIN,
            &[
                self.as_bytes(),
                format.encoder_contract_id().as_bytes(),
                build_id.as_bytes(),
            ],
        ))
    }
}
