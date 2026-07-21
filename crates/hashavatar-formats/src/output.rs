use crate::{
    AlphaSupport, AvatarOutputFormat, BuildEncodedAssetKey, EncoderBuildId, FormatError,
    SemanticEncodedAssetKey, keys::build_key,
};

/// Per-request format storage information visible to application admission.
#[must_use = "include format overhead in service resource admission"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FormatResourceBudget {
    canonical_rgba_bytes: usize,
    hashavatar_scratch_bytes: usize,
    codec_scratch_is_bounded: bool,
}

impl FormatResourceBudget {
    pub(crate) const fn new(canonical_rgba_bytes: usize, format: AvatarOutputFormat) -> Self {
        Self {
            canonical_rgba_bytes,
            hashavatar_scratch_bytes: if matches!(format, AvatarOutputFormat::Jpeg) {
                canonical_rgba_bytes / 4 * 3
            } else {
                0
            },
            codec_scratch_is_bounded: false,
        }
    }

    /// Returns exact canonical RGBA storage used before encoding.
    pub const fn canonical_rgba_bytes(self) -> usize {
        self.canonical_rgba_bytes
    }

    /// Returns exact additional Hashavatar-owned conversion scratch.
    pub const fn hashavatar_scratch_bytes(self) -> usize {
        self.hashavatar_scratch_bytes
    }

    /// Returns whether the upstream codec publishes a complete scratch bound.
    pub const fn codec_scratch_is_bounded(self) -> bool {
        self.codec_scratch_is_bounded
    }
}

/// Completion metadata for one successfully encoded avatar.
#[must_use = "use metadata for content type, storage keys, and resource accounting"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodedAvatarMetadata {
    format: AvatarOutputFormat,
    encoded_len: usize,
    semantic_key: SemanticEncodedAssetKey,
    resources: FormatResourceBudget,
}

impl EncodedAvatarMetadata {
    pub(crate) const fn new(
        format: AvatarOutputFormat,
        encoded_len: usize,
        semantic_key: SemanticEncodedAssetKey,
        resources: FormatResourceBudget,
    ) -> Self {
        Self {
            format,
            encoded_len,
            semantic_key,
            resources,
        }
    }

    /// Returns the encoded format.
    pub const fn format(self) -> AvatarOutputFormat {
        self.format
    }
    /// Returns the IANA media type.
    pub const fn media_type(self) -> &'static str {
        self.format.media_type()
    }
    /// Returns the conventional filename extension.
    pub const fn extension(self) -> &'static str {
        self.format.extension()
    }
    /// Returns the semantic encoder settings contract.
    pub const fn encoder_contract_id(self) -> &'static str {
        self.format.encoder_contract_id()
    }
    /// Returns the concrete encoder provider.
    pub const fn encoder_provider(self) -> &'static str {
        self.format.encoder_provider()
    }
    /// Returns successfully written encoded bytes.
    pub const fn encoded_len(self) -> usize {
        self.encoded_len
    }
    /// Returns the encoded format's alpha-channel capability.
    pub const fn alpha_support(self) -> AlphaSupport {
        self.format.alpha_support()
    }
    /// Returns the semantic encoded asset key.
    pub const fn semantic_key(self) -> SemanticEncodedAssetKey {
        self.semantic_key
    }
    /// Derives a deployment-bound key from a caller-controlled build ID.
    pub fn build_key(self, build_id: EncoderBuildId) -> Result<BuildEncodedAssetKey, FormatError> {
        build_key(self.semantic_key, build_id)
    }
    /// Returns format resource information.
    pub const fn resource_budget(self) -> FormatResourceBudget {
        self.resources
    }
}

/// Owned successfully encoded avatar and its immutable metadata.
#[must_use = "use or explicitly discard the encoded avatar"]
pub struct EncodedAvatar {
    bytes: Vec<u8>,
    metadata: EncodedAvatarMetadata,
}

impl EncodedAvatar {
    pub(crate) const fn new(bytes: Vec<u8>, metadata: EncodedAvatarMetadata) -> Self {
        Self { bytes, metadata }
    }

    /// Borrows encoded bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns immutable completion metadata.
    pub const fn metadata(&self) -> EncodedAvatarMetadata {
        self.metadata
    }

    /// Transfers encoded bytes to the caller.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl core::fmt::Debug for EncodedAvatar {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EncodedAvatar")
            .field("encoded_len", &self.bytes.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}
