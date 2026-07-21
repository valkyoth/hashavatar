use crate::AvatarOutputFormat;

/// Error owned by the format boundary.
#[derive(Debug)]
#[non_exhaustive]
pub enum FormatError {
    /// Canonical preparation or rendering failed before encoding.
    Core(hashavatar_core::AvatarError),
    /// The selected format feature is not enabled.
    FormatDisabled {
        /// Disabled format.
        format: AvatarOutputFormat,
    },
    /// Hashavatar-owned output or conversion allocation failed.
    Allocation,
    /// The caller-provided writer failed and may contain a partial prefix.
    Write(std::io::Error),
    /// The admitted image codec rejected canonical input or failed internally.
    #[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
    Codec(image::ImageError),
}

impl core::fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Core(error) => write!(formatter, "canonical avatar rendering failed: {error}"),
            Self::FormatDisabled { format } => {
                write!(formatter, "the {format} encoder feature is not enabled")
            }
            Self::Allocation => formatter.write_str("bounded format allocation failed"),
            Self::Write(error) => write!(formatter, "encoded output writer failed: {error}"),
            #[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
            Self::Codec(error) => write!(formatter, "image codec failed: {error}"),
        }
    }
}

impl std::error::Error for FormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Write(error) => Some(error),
            #[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
            Self::Codec(error) => Some(error),
            Self::FormatDisabled { .. } | Self::Allocation => None,
        }
    }
}

impl From<hashavatar_core::AvatarError> for FormatError {
    fn from(error: hashavatar_core::AvatarError) -> Self {
        Self::Core(error)
    }
}

#[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
pub(crate) fn map_image_error(error: image::ImageError) -> FormatError {
    match error {
        image::ImageError::IoError(error) => FormatError::Write(error),
        error => FormatError::Codec(error),
    }
}
