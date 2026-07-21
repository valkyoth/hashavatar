use std::io::Write;

#[cfg(any(feature = "webp", feature = "png", feature = "jpeg", feature = "gif"))]
use image::{ExtendedColorType, ImageEncoder};
#[cfg(feature = "jpeg")]
use sanitization::wipe;

use crate::{
    AvatarOutputFormat, EncodedAvatar, EncodedAvatarMetadata, FormatError, FormatResourceBudget,
    keys::semantic_key,
    writer::{CountingWriter, SanitizingWriter},
};

/// Renders and encodes an avatar into an owned byte vector.
pub fn encode(
    prepared: &hashavatar_core::PreparedAvatar,
    format: AvatarOutputFormat,
) -> Result<EncodedAvatar, FormatError> {
    let mut output =
        SanitizingWriter::try_with_capacity(prepared.resource_budget().canonical_rgba_bytes())?;
    let metadata = encode_to_writer(prepared, format, &mut output)?;
    Ok(EncodedAvatar::new(output.into_inner(), metadata))
}

/// Renders and streams one encoded image to a caller-owned writer.
///
/// Writer or codec failure may leave a partial encoded prefix. Retry with a
/// fresh destination. The temporary canonical RGBA allocation is sanitized on
/// drop.
pub fn encode_to_writer<W: Write>(
    prepared: &hashavatar_core::PreparedAvatar,
    format: AvatarOutputFormat,
    writer: &mut W,
) -> Result<EncodedAvatarMetadata, FormatError> {
    let mut scratch = hashavatar_core::ReusableRgbaBuffer::new();
    encode_to_writer_with_scratch(prepared, format, &mut scratch, writer)
}

/// Renders and streams using caller-retained reusable canonical RGBA storage.
///
/// On success or failure, `scratch` remains caller-owned and may contain the
/// rendered public pixels. Call [`hashavatar_core::ReusableRgbaBuffer::clear`]
/// when retaining them is undesirable.
pub fn encode_to_writer_with_scratch<W: Write>(
    prepared: &hashavatar_core::PreparedAvatar,
    format: AvatarOutputFormat,
    scratch: &mut hashavatar_core::ReusableRgbaBuffer,
    writer: &mut W,
) -> Result<EncodedAvatarMetadata, FormatError> {
    if !format.is_enabled() {
        return Err(FormatError::FormatDisabled { format });
    }
    prepared.render_reusing(scratch)?;
    let mut counter = CountingWriter::new(writer);
    write_pixels(
        scratch.pixels(),
        prepared.width(),
        prepared.height(),
        format,
        &mut counter,
    )?;
    let semantic_key = semantic_key(prepared.asset_key(), format)?;
    Ok(EncodedAvatarMetadata::new(
        format,
        counter.written(),
        semantic_key,
        FormatResourceBudget::new(prepared.resource_budget().canonical_rgba_bytes(), format),
    ))
}

fn write_pixels<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: AvatarOutputFormat,
    writer: &mut W,
) -> Result<(), FormatError> {
    match format {
        AvatarOutputFormat::WebP => write_webp(pixels, width, height, writer),
        AvatarOutputFormat::Png => write_png(pixels, width, height, writer),
        AvatarOutputFormat::Jpeg => write_jpeg(pixels, width, height, writer),
        AvatarOutputFormat::Gif => write_gif(pixels, width, height, writer),
    }
}

#[cfg(feature = "webp")]
fn write_webp<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    writer: &mut W,
) -> Result<(), FormatError> {
    image::codecs::webp::WebPEncoder::new_lossless(writer)
        .write_image(pixels, width, height, ExtendedColorType::Rgba8)
        .map_err(crate::error::map_image_error)
}

#[cfg(not(feature = "webp"))]
fn write_webp<W: Write>(
    _pixels: &[u8],
    _width: u32,
    _height: u32,
    _writer: &mut W,
) -> Result<(), FormatError> {
    Err(FormatError::FormatDisabled {
        format: AvatarOutputFormat::WebP,
    })
}

#[cfg(feature = "png")]
fn write_png<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    writer: &mut W,
) -> Result<(), FormatError> {
    image::codecs::png::PngEncoder::new_with_quality(
        writer,
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive,
    )
    .write_image(pixels, width, height, ExtendedColorType::Rgba8)
    .map_err(crate::error::map_image_error)
}

#[cfg(not(feature = "png"))]
fn write_png<W: Write>(
    _pixels: &[u8],
    _width: u32,
    _height: u32,
    _writer: &mut W,
) -> Result<(), FormatError> {
    Err(FormatError::FormatDisabled {
        format: AvatarOutputFormat::Png,
    })
}

#[cfg(feature = "jpeg")]
fn write_jpeg<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    writer: &mut W,
) -> Result<(), FormatError> {
    let rgb = WipingRgb::from_rgba_over_white(pixels)?;
    image::codecs::jpeg::JpegEncoder::new_with_quality(writer, 92)
        .write_image(rgb.bytes(), width, height, ExtendedColorType::Rgb8)
        .map_err(crate::error::map_image_error)
}

#[cfg(not(feature = "jpeg"))]
fn write_jpeg<W: Write>(
    _pixels: &[u8],
    _width: u32,
    _height: u32,
    _writer: &mut W,
) -> Result<(), FormatError> {
    Err(FormatError::FormatDisabled {
        format: AvatarOutputFormat::Jpeg,
    })
}

#[cfg(feature = "gif")]
fn write_gif<W: Write>(
    pixels: &[u8],
    width: u32,
    height: u32,
    writer: &mut W,
) -> Result<(), FormatError> {
    image::codecs::gif::GifEncoder::new(writer)
        .write_image(pixels, width, height, ExtendedColorType::Rgba8)
        .map_err(crate::error::map_image_error)
}

#[cfg(not(feature = "gif"))]
fn write_gif<W: Write>(
    _pixels: &[u8],
    _width: u32,
    _height: u32,
    _writer: &mut W,
) -> Result<(), FormatError> {
    Err(FormatError::FormatDisabled {
        format: AvatarOutputFormat::Gif,
    })
}

#[cfg(feature = "jpeg")]
struct WipingRgb {
    bytes: Vec<u8>,
}

#[cfg(feature = "jpeg")]
impl WipingRgb {
    fn from_rgba_over_white(rgba: &[u8]) -> Result<Self, FormatError> {
        let pixel_count = rgba.len() / 4;
        let required = pixel_count.checked_mul(3).ok_or(FormatError::Allocation)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(required)
            .map_err(|_| FormatError::Allocation)?;
        for pixel in rgba.chunks_exact(4) {
            let red = pixel.first().copied().unwrap_or_default();
            let green = pixel.get(1).copied().unwrap_or_default();
            let blue = pixel.get(2).copied().unwrap_or_default();
            let alpha = u32::from(pixel.get(3).copied().unwrap_or_default());
            let inverse = 255_u32.saturating_sub(alpha);
            for channel in [red, green, blue] {
                let blended = (u32::from(channel) * alpha + 255_u32 * inverse + 127_u32) / 255_u32;
                bytes.push(u8::try_from(blended).unwrap_or_default());
            }
        }
        Ok(Self { bytes })
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(feature = "jpeg")]
impl Drop for WipingRgb {
    fn drop(&mut self) {
        wipe::vec(&mut self.bytes);
    }
}
