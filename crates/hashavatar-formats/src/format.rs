/// Alpha-channel capability of one encoded format contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlphaSupport {
    /// Alpha is deterministically flattened and not encoded.
    None,
    /// Only palette/binary transparency is preserved.
    Binary,
    /// Full per-pixel alpha is preserved.
    Full,
}

/// Established encoded image format.
///
/// Variants are always available for configuration and schema code. Encoding
/// returns a typed disabled-format error unless the corresponding Cargo feature
/// is enabled.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AvatarOutputFormat {
    /// Lossless WebP; enabled by the default feature.
    WebP,
    /// Lossless PNG with best/adaptive image-rs settings.
    Png,
    /// Quality-92 JPEG after deterministic alpha flattening over white.
    Jpeg,
    /// Single-frame GIF with image-rs speed 1 quantization.
    Gif,
}

impl AvatarOutputFormat {
    /// Complete established-format catalog in stable order.
    pub const ALL: [Self; 4] = [Self::WebP, Self::Png, Self::Jpeg, Self::Gif];

    /// Returns the stable format identifier.
    pub const fn catalog_id(self) -> u8 {
        self as u8
    }

    /// Returns whether this format was compiled into the current build.
    pub const fn is_enabled(self) -> bool {
        match self {
            Self::WebP => cfg!(feature = "webp"),
            Self::Png => cfg!(feature = "png"),
            Self::Jpeg => cfg!(feature = "jpeg"),
            Self::Gif => cfg!(feature = "gif"),
        }
    }

    /// Returns the IANA media type.
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::WebP => "image/webp",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
        }
    }

    /// Returns the conventional extension without a leading dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
        }
    }

    /// Returns the frozen semantic encoder settings contract.
    pub const fn encoder_contract_id(self) -> &'static str {
        match self {
            Self::WebP => "hashavatar/webp-lossless/v1",
            Self::Png => "hashavatar/png-best-adaptive/v1",
            Self::Jpeg => "hashavatar/jpeg-q92-white/v1",
            Self::Gif => "hashavatar/gif-single-speed1/v1",
        }
    }

    /// Returns the concrete encoder provider used by alpha.5.
    pub const fn encoder_provider(self) -> &'static str {
        "image-rs/image/0.25.10"
    }

    /// Returns whether decoded RGB and alpha are lossless under this contract.
    pub const fn is_lossless(self) -> bool {
        matches!(self, Self::WebP | Self::Png)
    }

    /// Returns the format's alpha-channel capability.
    pub const fn alpha_support(self) -> AlphaSupport {
        match self {
            Self::WebP | Self::Png => AlphaSupport::Full,
            Self::Gif => AlphaSupport::Binary,
            Self::Jpeg => AlphaSupport::None,
        }
    }
}

impl core::fmt::Display for AvatarOutputFormat {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Gif => "gif",
        })
    }
}
