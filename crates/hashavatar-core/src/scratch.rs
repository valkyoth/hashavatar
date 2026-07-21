use alloc::vec::Vec;

use sanitization::wipe;

use crate::{
    AvatarError, PixelDigest, RGBA8_BYTES_PER_PIXEL, RgbaSurfaceMut, raster::digest_rows,
    scene::validate_dimensions,
};

/// Reusable tightly packed canonical RGBA8 storage.
///
/// Preparation uses fallible allocation, clears prior visible bytes, and does
/// not change the buffer if validation or reservation fails. The allocation is
/// sanitized on drop because Hashavatar owns this scratch container.
#[must_use = "reuse the buffer across canonical renders or call clear"]
pub struct ReusableRgbaBuffer {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl ReusableRgbaBuffer {
    /// Creates empty reusable storage without allocating.
    pub const fn new() -> Self {
        Self {
            pixels: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Returns the currently prepared dimensions, or `(0, 0)` while empty.
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the initialized visible RGBA8 bytes.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns reusable allocation capacity in bytes.
    pub fn capacity(&self) -> usize {
        self.pixels.capacity()
    }

    /// Calculates the canonical digest of the current tightly packed image.
    pub fn pixel_digest(&self) -> Result<PixelDigest, AvatarError> {
        if self.width == 0 || self.height == 0 {
            return Err(AvatarError::InvalidSurface);
        }
        let stride = width_stride(self.width)?;
        digest_rows(self.width, self.height, stride, &self.pixels)
    }

    /// Clears initialized bytes and resets dimensions while retaining capacity.
    pub fn clear(&mut self) {
        wipe::bytes(&mut self.pixels);
        self.pixels.clear();
        self.width = 0;
        self.height = 0;
    }

    pub(crate) fn prepare(&mut self, width: u32, height: u32) -> Result<(), AvatarError> {
        validate_dimensions(width, height)?;
        let required = required_len(width, height)?;
        if required > self.pixels.capacity() {
            let additional = required.saturating_sub(self.pixels.len());
            self.pixels
                .try_reserve_exact(additional)
                .map_err(|_| AvatarError::Allocation)?;
        }
        wipe::bytes(&mut self.pixels);
        self.pixels.resize(required, 0);
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub(crate) fn surface_mut(&mut self) -> Result<RgbaSurfaceMut<'_>, AvatarError> {
        let stride = width_stride(self.width)?;
        RgbaSurfaceMut::new(&mut self.pixels, self.width, self.height, stride)
    }
}

impl Default for ReusableRgbaBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for ReusableRgbaBuffer {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ReusableRgbaBuffer")
            .field("dimensions", &self.dimensions())
            .field("initialized_bytes", &self.pixels.len())
            .field("capacity", &self.pixels.capacity())
            .finish_non_exhaustive()
    }
}

impl Drop for ReusableRgbaBuffer {
    fn drop(&mut self) {
        wipe::vec(&mut self.pixels);
    }
}

fn width_stride(width: u32) -> Result<usize, AvatarError> {
    usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(RGBA8_BYTES_PER_PIXEL))
        .ok_or(AvatarError::NumericRange)
}

fn required_len(width: u32, height: u32) -> Result<usize, AvatarError> {
    width_stride(width)?
        .checked_mul(usize::try_from(height).map_err(|_| AvatarError::NumericRange)?)
        .ok_or(AvatarError::NumericRange)
}
