//! Alpha.5 feature, metadata, writer, and decode-to-canonical format evidence.

use std::io::Write;

use hashavatar_core::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle, PreparedAvatar,
    ReusableRgbaBuffer,
};
use hashavatar_formats::{
    AvatarOutputFormat, EncoderBuildId, FormatError, encode, encode_to_writer,
    encode_to_writer_with_scratch,
};

fn prepared(background: AvatarBackground) -> Result<PreparedAvatar, hashavatar_core::AvatarError> {
    let style = AvatarStyle::new(AvatarKind::Dragon, background, AvatarShape::Squircle);
    AvatarRequest::new(96, 80, 11, b"format-fixture", style)?.prepare()
}

#[test]
fn format_metadata_and_keys_are_typed_and_deterministic() -> Result<(), Box<dyn std::error::Error>>
{
    let prepared = prepared(AvatarBackground::Ocean)?;
    for format in AvatarOutputFormat::ALL
        .iter()
        .copied()
        .filter(|format| format.is_enabled())
    {
        let first = encode(&prepared, format)?;
        let second = encode(&prepared, format)?;
        assert_eq!(first.bytes(), second.bytes());
        assert_eq!(first.metadata(), second.metadata());
        assert_eq!(first.metadata().encoded_len(), first.bytes().len());
        assert_eq!(first.metadata().media_type(), format.media_type());
        assert_eq!(first.metadata().extension(), format.extension());
        let build_a = first
            .metadata()
            .build_key(EncoderBuildId::from_bytes([1_u8; 32]))?;
        let build_b = first
            .metadata()
            .build_key(EncoderBuildId::from_bytes([2_u8; 32]))?;
        assert_ne!(build_a, build_b);
    }
    Ok(())
}

#[test]
fn disabled_formats_fail_before_rendering_or_writing() -> Result<(), Box<dyn std::error::Error>> {
    let prepared = prepared(AvatarBackground::Ocean)?;
    for format in AvatarOutputFormat::ALL
        .iter()
        .copied()
        .filter(|format| !format.is_enabled())
    {
        let mut output = Vec::new();
        assert!(matches!(
            encode_to_writer(&prepared, format, &mut output),
            Err(FormatError::FormatDisabled { format: rejected }) if rejected == format
        ));
        assert!(output.is_empty());
    }
    Ok(())
}

#[test]
fn reusable_format_scratch_preserves_encoded_output() -> Result<(), Box<dyn std::error::Error>> {
    let prepared = prepared(AvatarBackground::Ocean)?;
    let Some(format) = AvatarOutputFormat::ALL
        .iter()
        .copied()
        .find(|format| format.is_enabled())
    else {
        return Ok(());
    };
    let expected = encode(&prepared, format)?;
    let mut scratch = ReusableRgbaBuffer::new();
    let mut output = Vec::new();
    let metadata = encode_to_writer_with_scratch(&prepared, format, &mut scratch, &mut output)?;
    assert_eq!(output, expected.bytes());
    assert_eq!(metadata, expected.metadata());
    assert_eq!(scratch.dimensions(), (96, 80));
    Ok(())
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("intentional writer failure"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn writer_failure_is_owned_by_the_format_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let prepared = prepared(AvatarBackground::Ocean)?;
    let Some(format) = AvatarOutputFormat::ALL
        .iter()
        .copied()
        .find(|format| format.is_enabled())
    else {
        return Ok(());
    };
    assert!(matches!(
        encode_to_writer(&prepared, format, &mut FailingWriter),
        Err(FormatError::Write(_))
    ));
    Ok(())
}

#[cfg(feature = "webp")]
#[test]
fn webp_decodes_to_exact_canonical_pixels() -> Result<(), Box<dyn std::error::Error>> {
    lossless_round_trip(AvatarOutputFormat::WebP, image::ImageFormat::WebP)
}

#[cfg(feature = "png")]
#[test]
fn png_decodes_to_exact_canonical_pixels() -> Result<(), Box<dyn std::error::Error>> {
    lossless_round_trip(AvatarOutputFormat::Png, image::ImageFormat::Png)
}

#[cfg(any(feature = "webp", feature = "png"))]
fn lossless_round_trip(
    format: AvatarOutputFormat,
    image_format: image::ImageFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let prepared = prepared(AvatarBackground::Transparent)?;
    let canonical = prepared.render_rgba()?;
    let encoded = encode(&prepared, format)?;
    let decoded = image::load_from_memory_with_format(encoded.bytes(), image_format)?.to_rgba8();
    assert_eq!(decoded.dimensions(), canonical.dimensions());
    assert_eq!(decoded.as_raw(), canonical.pixels());
    Ok(())
}

#[cfg(feature = "jpeg")]
#[test]
fn jpeg_decodes_within_frozen_lossy_evidence_bound() -> Result<(), Box<dyn std::error::Error>> {
    let prepared = prepared(AvatarBackground::Transparent)?;
    let canonical = prepared.render_rgba()?;
    let encoded = encode(&prepared, AvatarOutputFormat::Jpeg)?;
    let decoded =
        image::load_from_memory_with_format(encoded.bytes(), image::ImageFormat::Jpeg)?.to_rgb8();
    let expected = rgba_over_white(canonical.pixels());
    assert_eq!(decoded.dimensions(), canonical.dimensions());
    assert!(mean_absolute_error(decoded.as_raw(), &expected) <= 10);
    Ok(())
}

#[cfg(feature = "gif")]
#[test]
fn gif_decodes_within_frozen_quantization_evidence_bound() -> Result<(), Box<dyn std::error::Error>>
{
    let prepared = prepared(AvatarBackground::Light)?;
    let canonical = prepared.render_rgba()?;
    let encoded = encode(&prepared, AvatarOutputFormat::Gif)?;
    let decoded =
        image::load_from_memory_with_format(encoded.bytes(), image::ImageFormat::Gif)?.to_rgba8();
    assert_eq!(decoded.dimensions(), canonical.dimensions());
    assert!(mean_absolute_error(decoded.as_raw(), canonical.pixels()) <= 12);
    Ok(())
}

#[cfg(feature = "jpeg")]
fn rgba_over_white(rgba: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    for pixel in rgba.chunks_exact(4) {
        let alpha = u32::from(pixel.get(3).copied().unwrap_or_default());
        for channel in pixel.iter().take(3).copied() {
            let value = (u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255;
            output.push(u8::try_from(value).unwrap_or_default());
        }
    }
    output
}

#[cfg(any(feature = "jpeg", feature = "gif"))]
fn mean_absolute_error(first: &[u8], second: &[u8]) -> u64 {
    let total = first
        .iter()
        .zip(second.iter())
        .fold(0_u64, |sum, (first, second)| {
            sum.saturating_add(u64::from(first.abs_diff(*second)))
        });
    total / u64::try_from(first.len().max(1)).unwrap_or(1)
}
