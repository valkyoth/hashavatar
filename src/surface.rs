use super::*;

/// Conservative known memory requirements for the 1.x preview adapters.
///
/// Codec-internal allocations are format-dependent and intentionally excluded.
#[must_use = "use ResourceBudget to enforce service-level concurrency policy"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    spec: AvatarSpec,
    rgba_stride: usize,
    rgba_bytes: usize,
}

impl ResourceBudget {
    pub(crate) const fn new(spec: AvatarSpec) -> Self {
        let rgba_stride = (spec.width() as usize).saturating_mul(AVATAR_RGBA_BYTES_PER_PIXEL);
        Self {
            spec,
            rgba_stride,
            rgba_bytes: spec.rgba_buffer_len(),
        }
    }

    /// Returns the specification used for these calculations.
    pub const fn spec(self) -> AvatarSpec {
        self.spec
    }

    /// Returns the minimum RGBA8 bytes required for one row.
    pub const fn minimum_rgba8_stride(self) -> usize {
        self.rgba_stride
    }

    /// Returns the minimum bytes required for a tightly packed RGBA8 surface.
    pub const fn minimum_rgba8_surface_bytes(self) -> usize {
        self.rgba_bytes
    }

    /// Internal temporary RGBA allocation used by the 1.x `render_into` adapter.
    pub const fn render_into_temporary_bytes(self) -> usize {
        self.rgba_bytes
    }

    /// Minimum known bytes across a tight surface and the 1.x temporary image.
    pub const fn minimum_render_into_known_rgba_bytes(self) -> usize {
        self.rgba_bytes.saturating_mul(2)
    }

    /// Known bytes for a declared surface plus the 1.x temporary image.
    ///
    /// This excludes trailing bytes beyond the surface's declared stride and
    /// height because `render_into` never accesses them.
    pub fn render_into_known_rgba_bytes_for(
        self,
        surface: &RasterSurfaceMut<'_>,
    ) -> Result<usize, RasterSurfaceError> {
        if (surface.width(), surface.height()) != (self.spec.width(), self.spec.height()) {
            return Err(RasterSurfaceError::DimensionMismatch {
                expected_width: self.spec.width(),
                expected_height: self.spec.height(),
                actual_width: surface.width(),
                actual_height: surface.height(),
            });
        }
        surface
            .required_len()
            .checked_add(self.rgba_bytes)
            .ok_or(RasterSurfaceError::LengthOverflow)
    }

    /// Image plus the initial returned-`Vec` allocation used by `encode`.
    ///
    /// Codec scratch space, temporary replacement allocations during output
    /// growth, and capacity beyond the initial reserve remain excluded.
    pub const fn encode_vec_known_base_bytes(self) -> Option<usize> {
        self.rgba_bytes.checked_mul(2)
    }

    /// Internal image bytes used before writer and codec-specific allocations.
    pub const fn encode_writer_known_base_bytes(self) -> usize {
        self.rgba_bytes
    }
}

pub(crate) fn copy_rgba_image_into_surface(
    spec: AvatarSpec,
    image: &RgbaImage,
    surface: &mut RasterSurfaceMut<'_>,
) -> Result<(), RasterSurfaceError> {
    if (surface.width(), surface.height()) != (spec.width(), spec.height()) {
        return Err(RasterSurfaceError::DimensionMismatch {
            expected_width: spec.width(),
            expected_height: spec.height(),
            actual_width: surface.width(),
            actual_height: surface.height(),
        });
    }

    let expected_rows =
        usize::try_from(spec.height()).map_err(|_| RasterSurfaceError::LengthOverflow)?;
    let row_bytes = ResourceBudget::new(spec).minimum_rgba8_stride();
    let expected_len = row_bytes
        .checked_mul(expected_rows)
        .ok_or(RasterSurfaceError::LengthOverflow)?;
    if image.dimensions() != (spec.width(), spec.height()) || image.as_raw().len() != expected_len {
        return Err(renderer_output_mismatch(spec, image, expected_len));
    }

    let stride = surface.stride();
    for row in 0..expected_rows {
        let source_start = row
            .checked_mul(row_bytes)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        let source_end = source_start
            .checked_add(row_bytes)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        let destination_start = row
            .checked_mul(stride)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        let destination_end = destination_start
            .checked_add(row_bytes)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        let source = image
            .as_raw()
            .get(source_start..source_end)
            .ok_or_else(|| renderer_output_mismatch(spec, image, expected_len))?;
        let destination = surface
            .pixels_mut()
            .get_mut(destination_start..destination_end)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        destination.copy_from_slice(source);
    }
    Ok(())
}

fn renderer_output_mismatch(
    spec: AvatarSpec,
    image: &RgbaImage,
    expected_len: usize,
) -> RasterSurfaceError {
    RasterSurfaceError::RendererOutputMismatch {
        expected_width: spec.width(),
        expected_height: spec.height(),
        actual_width: image.width(),
        actual_height: image.height(),
        expected_len,
        actual_len: image.as_raw().len(),
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use super::*;

    #[test]
    fn copy_rejects_renderer_dimension_mismatch_before_touching_surface() {
        let spec = AvatarSpec::new(64, 64, 0).expect("valid spec");
        let image = ImageBuffer::from_pixel(64, 63, Rgba([1, 2, 3, 4]));
        let mut pixels = vec![0xa5; spec.rgba_buffer_len()];
        let mut surface =
            RasterSurfaceMut::new_rgba8(&mut pixels, 64, 64, 64 * 4).expect("valid surface");

        assert!(matches!(
            copy_rgba_image_into_surface(spec, &image, &mut surface),
            Err(RasterSurfaceError::RendererOutputMismatch { .. })
        ));
        assert!(surface.pixels().iter().all(|byte| *byte == 0xa5));
    }
}
