use alloc::vec::Vec;

use sanitization_crypto_interop::sha2::SanitizedSha512;

use crate::{
    AvatarError, PIXEL_CONTRACT_ID, RGBA8_BYTES_PER_PIXEL,
    paint::{Color, div_255_round, source_over},
    rasterize::{clip_contains, draw_ellipse, draw_line, draw_path, draw_rect, draw_triangle},
    scene::{Clip, Command, MAX_STACK_DEPTH, Scene, validate_dimensions},
};

/// Tightly packed canonical straight-alpha RGBA8 output.
#[must_use = "use or explicitly discard the rendered pixel buffer"]
pub struct CanonicalRgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl CanonicalRgbaImage {
    /// Returns the image width.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the image height.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the image dimensions.
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Borrows the tightly packed straight-alpha RGBA8 pixels.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Calculates the versioned digest of dimensions and visible pixel rows.
    pub fn pixel_digest(&self) -> Result<PixelDigest, AvatarError> {
        digest_rows(self.width, self.height, self.width_stride()?, &self.pixels)
    }

    /// Transfers ownership of the tightly packed RGBA8 pixels.
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    fn width_stride(&self) -> Result<usize, AvatarError> {
        usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(RGBA8_BYTES_PER_PIXEL))
            .ok_or(AvatarError::NumericRange)
    }
}

/// Validated caller-owned straight-alpha RGBA8 surface.
///
/// Rows may contain padding. Canonical execution modifies visible bytes only;
/// row padding and any trailing bytes remain unchanged.
#[must_use = "pass the validated surface to a prepared avatar"]
pub struct RgbaSurfaceMut<'a> {
    pixels: &'a mut [u8],
    width: u32,
    height: u32,
    stride: usize,
    visible_row_bytes: usize,
}

impl<'a> RgbaSurfaceMut<'a> {
    /// Validates dimensions, stride, and required buffer length.
    pub fn new(
        pixels: &'a mut [u8],
        width: u32,
        height: u32,
        stride: usize,
    ) -> Result<Self, AvatarError> {
        validate_dimensions(width, height)?;
        let visible_row_bytes = usize::try_from(width)
            .ok()
            .and_then(|value| value.checked_mul(RGBA8_BYTES_PER_PIXEL))
            .ok_or(AvatarError::NumericRange)?;
        if stride < visible_row_bytes {
            return Err(AvatarError::InvalidSurface);
        }
        let rows_before_last =
            usize::try_from(height.saturating_sub(1)).map_err(|_| AvatarError::NumericRange)?;
        let required = rows_before_last
            .checked_mul(stride)
            .and_then(|value| value.checked_add(visible_row_bytes))
            .ok_or(AvatarError::NumericRange)?;
        if pixels.len() < required {
            return Err(AvatarError::InvalidSurface);
        }
        Ok(Self {
            pixels,
            width,
            height,
            stride,
            visible_row_bytes,
        })
    }

    /// Returns the surface dimensions.
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the row stride in bytes.
    pub const fn stride(&self) -> usize {
        self.stride
    }

    /// Returns the number of visible bytes in each row.
    pub const fn visible_row_bytes(&self) -> usize {
        self.visible_row_bytes
    }

    /// Borrows the complete caller buffer, including padding.
    pub fn as_bytes(&self) -> &[u8] {
        self.pixels
    }

    /// Calculates the canonical digest without including row padding.
    pub fn pixel_digest(&self) -> Result<PixelDigest, AvatarError> {
        digest_rows(self.width, self.height, self.stride, self.pixels)
    }

    fn pixel_mut(&mut self, x: u32, y: u32) -> Result<&mut [u8], AvatarError> {
        if x >= self.width || y >= self.height {
            return Err(AvatarError::InvalidScene);
        }
        let offset = usize::try_from(y)
            .ok()
            .and_then(|row| row.checked_mul(self.stride))
            .and_then(|row| {
                usize::try_from(x)
                    .ok()
                    .and_then(|column| column.checked_mul(RGBA8_BYTES_PER_PIXEL))
                    .and_then(|column| row.checked_add(column))
            })
            .ok_or(AvatarError::NumericRange)?;
        let end = offset
            .checked_add(RGBA8_BYTES_PER_PIXEL)
            .ok_or(AvatarError::NumericRange)?;
        self.pixels
            .get_mut(offset..end)
            .ok_or(AvatarError::InvalidSurface)
    }
}

/// SHA-512 digest over the frozen pixel contract, dimensions, and visible rows.
#[must_use = "pixel digests are intended for cache and reproducibility checks"]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PixelDigest([u8; 64]);

impl PixelDigest {
    /// Borrows the 64 digest bytes.
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl core::fmt::Debug for PixelDigest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("PixelDigest(")?;
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        formatter.write_str(")")
    }
}

pub(crate) fn render_scene(scene: &Scene) -> Result<CanonicalRgbaImage, AvatarError> {
    let report = scene.validate()?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(report.rgba_bytes())
        .map_err(|_| AvatarError::Allocation)?;
    pixels.resize(report.rgba_bytes(), 0);
    let stride = usize::try_from(scene.width())
        .ok()
        .and_then(|value| value.checked_mul(RGBA8_BYTES_PER_PIXEL))
        .ok_or(AvatarError::NumericRange)?;
    {
        let mut surface = RgbaSurfaceMut::new(&mut pixels, scene.width(), scene.height(), stride)?;
        render_scene_into(scene, &mut surface)?;
    }
    Ok(CanonicalRgbaImage {
        width: scene.width(),
        height: scene.height(),
        pixels,
    })
}

pub(crate) fn render_scene_into(
    scene: &Scene,
    surface: &mut RgbaSurfaceMut<'_>,
) -> Result<(), AvatarError> {
    let _ = scene.validate()?;
    if surface.dimensions() != (scene.width(), scene.height()) {
        return Err(AvatarError::InvalidSurface);
    }
    let mut writer = SurfaceWriter::new(surface, scene);
    for command in scene.commands()? {
        match *command {
            Command::Empty => return Err(AvatarError::InvalidScene),
            Command::Fill(paint) => writer.fill_background(paint)?,
            Command::Rect { rect, paint } => draw_rect(&mut writer, rect, paint)?,
            Command::Ellipse {
                center,
                radius_x,
                radius_y,
                paint,
            } => draw_ellipse(&mut writer, center, radius_x, radius_y, paint)?,
            Command::Triangle { points, paint } => draw_triangle(&mut writer, points, paint)?,
            Command::Line { start, end, stroke } => draw_line(&mut writer, start, end, stroke)?,
            Command::Path {
                path_index,
                fill_rule,
                fill,
                stroke,
            } => draw_path(
                &mut writer,
                scene.path(path_index)?,
                fill_rule,
                fill,
                stroke,
            )?,
            Command::PushClip(rect) => writer.push_clip(rect)?,
            Command::PopClip => writer.pop_clip()?,
            Command::PushOpacity(opacity) => writer.push_opacity(opacity)?,
            Command::PopOpacity => writer.pop_opacity()?,
        }
    }
    Ok(())
}

pub(crate) struct SurfaceWriter<'scene, 'surface, 'pixels> {
    scene: &'scene Scene,
    surface: &'surface mut RgbaSurfaceMut<'pixels>,
    clips: [Option<Clip>; MAX_STACK_DEPTH],
    clip_depth: usize,
    opacities: [u8; MAX_STACK_DEPTH],
    opacity_depth: usize,
    opacity: u8,
}

impl<'scene, 'surface, 'pixels> SurfaceWriter<'scene, 'surface, 'pixels> {
    fn new(surface: &'surface mut RgbaSurfaceMut<'pixels>, scene: &'scene Scene) -> Self {
        Self {
            scene,
            surface,
            clips: [None; MAX_STACK_DEPTH],
            clip_depth: 0,
            opacities: [u8::MAX; MAX_STACK_DEPTH],
            opacity_depth: 0,
            opacity: u8::MAX,
        }
    }

    pub(crate) const fn width(&self) -> u32 {
        self.surface.width
    }

    pub(crate) const fn height(&self) -> u32 {
        self.surface.height
    }

    fn fill_background(&mut self, paint: crate::paint::Paint) -> Result<(), AvatarError> {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let sample = crate::geometry::Point::new(
                    crate::fixed::Fixed::pixel_center(x)?,
                    crate::fixed::Fixed::pixel_center(y)?,
                );
                let color = paint.sample(sample)?;
                self.surface
                    .pixel_mut(x, y)?
                    .copy_from_slice(&color.channels());
            }
        }
        Ok(())
    }

    pub(crate) fn paint_pixel(&mut self, x: u32, y: u32, source: Color) -> Result<(), AvatarError> {
        let sample = crate::geometry::Point::new(
            crate::fixed::Fixed::pixel_center(x)?,
            crate::fixed::Fixed::pixel_center(y)?,
        );
        for clip in self
            .clips
            .get(..self.clip_depth)
            .ok_or(AvatarError::InvalidScene)?
        {
            if !clip_contains(self.scene, clip.ok_or(AvatarError::InvalidScene)?, sample)? {
                return Ok(());
            }
        }
        let source = source.with_opacity(self.opacity);
        let pixel = self.surface.pixel_mut(x, y)?;
        let destination = <[u8; 4]>::try_from(&*pixel).map_err(|_| AvatarError::InvalidSurface)?;
        pixel.copy_from_slice(&source_over(destination, source));
        Ok(())
    }

    fn push_clip(&mut self, clip: Clip) -> Result<(), AvatarError> {
        let slot = self
            .clips
            .get_mut(self.clip_depth)
            .ok_or(AvatarError::InvalidScene)?;
        *slot = Some(clip);
        self.clip_depth = self
            .clip_depth
            .checked_add(1)
            .ok_or(AvatarError::NumericRange)?;
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<(), AvatarError> {
        self.clip_depth = self
            .clip_depth
            .checked_sub(1)
            .ok_or(AvatarError::InvalidScene)?;
        let slot = self
            .clips
            .get_mut(self.clip_depth)
            .ok_or(AvatarError::InvalidScene)?;
        *slot = None;
        Ok(())
    }

    fn push_opacity(&mut self, opacity: u8) -> Result<(), AvatarError> {
        let slot = self
            .opacities
            .get_mut(self.opacity_depth)
            .ok_or(AvatarError::InvalidScene)?;
        *slot = self.opacity;
        self.opacity_depth = self
            .opacity_depth
            .checked_add(1)
            .ok_or(AvatarError::NumericRange)?;
        self.opacity = u8::try_from(div_255_round(u32::from(self.opacity) * u32::from(opacity)))
            .map_err(|_| AvatarError::NumericRange)?;
        Ok(())
    }

    fn pop_opacity(&mut self) -> Result<(), AvatarError> {
        self.opacity_depth = self
            .opacity_depth
            .checked_sub(1)
            .ok_or(AvatarError::InvalidScene)?;
        self.opacity = *self
            .opacities
            .get(self.opacity_depth)
            .ok_or(AvatarError::InvalidScene)?;
        Ok(())
    }
}

pub(crate) fn digest_rows(
    width: u32,
    height: u32,
    stride: usize,
    pixels: &[u8],
) -> Result<PixelDigest, AvatarError> {
    let visible = usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(RGBA8_BYTES_PER_PIXEL))
        .ok_or(AvatarError::NumericRange)?;

    let mut hasher = SanitizedSha512::new();
    hasher.update(PIXEL_CONTRACT_ID.as_bytes());
    hasher.update(&width.to_le_bytes());
    hasher.update(&height.to_le_bytes());
    for row in 0..height {
        let start = usize::try_from(row)
            .ok()
            .and_then(|value| value.checked_mul(stride))
            .ok_or(AvatarError::NumericRange)?;
        let end = start
            .checked_add(visible)
            .ok_or(AvatarError::NumericRange)?;
        hasher.update(pixels.get(start..end).ok_or(AvatarError::InvalidSurface)?);
    }
    Ok(PixelDigest(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surfaces_reject_short_stride_and_short_buffers() {
        let mut pixels = [0_u8; 64 * 64 * 4];
        assert!(matches!(
            RgbaSurfaceMut::new(&mut pixels, 64, 64, 255),
            Err(AvatarError::InvalidSurface)
        ));
        assert!(matches!(
            RgbaSurfaceMut::new(&mut pixels[..100], 64, 64, 256),
            Err(AvatarError::InvalidSurface)
        ));
    }
}
