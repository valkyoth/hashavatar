use std::io::Write;

use super::*;

/// Requested and effective style bound to a prepared avatar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedStyle {
    requested: AvatarStyleOptions,
    effective: AvatarStyleOptions,
    automatically_derived: bool,
}

impl ResolvedStyle {
    /// Returns the style requested before family compatibility handling.
    pub const fn requested(self) -> AvatarStyleOptions {
        self.requested
    }

    /// Returns the canonical style actually used for keys and rendering.
    pub const fn effective(self) -> AvatarStyleOptions {
        self.effective
    }

    /// Reports whether the requested style was derived from the identity.
    pub const fn is_automatically_derived(self) -> bool {
        self.automatically_derived
    }

    /// Reports whether any unsupported legacy face layer was ignored.
    pub fn applied_legacy_fallbacks(self) -> bool {
        self.requested.accessory != self.effective.accessory
            || self.requested.expression != self.effective.expression
    }

    /// Reports whether the requested accessory was ignored.
    pub fn ignored_accessory(self) -> bool {
        self.requested.accessory != self.effective.accessory
    }

    /// Reports whether the requested expression was ignored.
    pub fn ignored_expression(self) -> bool {
        self.requested.expression != self.effective.expression
    }
}

/// Stable preparation report for the effective 1.x layout contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutReport {
    spec: AvatarSpec,
    style: ResolvedStyle,
    capabilities: AvatarFamilyCapabilities,
}

impl LayoutReport {
    /// Returns the prepared image specification.
    pub const fn spec(self) -> AvatarSpec {
        self.spec
    }

    /// Returns requested and effective style information.
    pub const fn resolved_style(self) -> ResolvedStyle {
        self.style
    }

    /// Returns the selected family's declared layer capabilities.
    pub const fn family_capabilities(self) -> AvatarFamilyCapabilities {
        self.capabilities
    }

    /// Returns the catalog contract used for style derivation and keys.
    pub const fn catalog_version(self) -> CatalogVersion {
        CatalogVersion::CURRENT
    }

    /// Returns the renderer contract used for output and keys.
    pub const fn render_contract_id(self) -> RenderContractId {
        RenderContractId::CURRENT
    }
}

/// Mutable caller-owned RGBA8 surface with an explicit row stride.
pub struct RasterSurfaceMut<'a> {
    pixels: &'a mut [u8],
    width: u32,
    height: u32,
    stride: usize,
    required_len: usize,
}

impl std::fmt::Debug for RasterSurfaceMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RasterSurfaceMut")
            .field("pixels", &format_args!("[{} bytes]", self.pixels.len()))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("required_len", &self.required_len)
            .finish()
    }
}

impl<'a> RasterSurfaceMut<'a> {
    /// Validates a caller-owned RGBA8 surface and its explicit row stride.
    ///
    /// `pixels` may contain trailing bytes or row padding. The required length
    /// is `stride * height`; arithmetic overflow returns an error.
    pub fn new_rgba8(
        pixels: &'a mut [u8],
        width: u32,
        height: u32,
        stride: usize,
    ) -> Result<Self, RasterSurfaceError> {
        if width == 0 || height == 0 {
            return Err(RasterSurfaceError::ZeroDimension { width, height });
        }
        let width_usize = usize::try_from(width).map_err(|_| RasterSurfaceError::LengthOverflow)?;
        let height_usize =
            usize::try_from(height).map_err(|_| RasterSurfaceError::LengthOverflow)?;
        let minimum_stride = width_usize
            .checked_mul(AVATAR_RGBA_BYTES_PER_PIXEL)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        if stride < minimum_stride {
            return Err(RasterSurfaceError::StrideTooSmall {
                minimum: minimum_stride,
                actual: stride,
            });
        }
        let required = stride
            .checked_mul(height_usize)
            .ok_or(RasterSurfaceError::LengthOverflow)?;
        if pixels.len() < required {
            return Err(RasterSurfaceError::BufferTooSmall {
                required,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            pixels,
            width,
            height,
            stride,
            required_len: required,
        })
    }

    /// Returns the surface width in pixels.
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the surface height in pixels.
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns the distance in bytes between consecutive rows.
    pub const fn stride(&self) -> usize {
        self.stride
    }

    /// Returns `stride * height`, the declared bytes used by this surface.
    pub const fn required_len(&self) -> usize {
        self.required_len
    }

    /// Returns the complete caller-provided slice length, including trailing bytes.
    pub const fn provided_len(&self) -> usize {
        self.pixels.len()
    }

    /// Borrows the complete caller-provided pixel slice.
    pub fn pixels(&self) -> &[u8] {
        self.pixels
    }

    /// Mutably borrows the complete caller-provided pixel slice.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        self.pixels
    }
}

/// Error returned by caller-surface construction or rendering.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RasterSurfaceError {
    /// A zero width or height was supplied.
    ZeroDimension { width: u32, height: u32 },
    /// The row stride cannot hold one RGBA8 row.
    StrideTooSmall { minimum: usize, actual: usize },
    /// The caller-provided slice cannot hold all declared rows.
    BufferTooSmall { required: usize, actual: usize },
    /// Surface length arithmetic overflowed `usize`.
    LengthOverflow,
    /// The surface dimensions differ from the prepared request.
    DimensionMismatch {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
    /// A renderer returned dimensions or storage inconsistent with its spec.
    RendererOutputMismatch {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
        expected_len: usize,
        actual_len: usize,
    },
    /// The prepared renderer rejected its image specification.
    Render(AvatarSpecError),
}

impl From<AvatarSpecError> for RasterSurfaceError {
    fn from(error: AvatarSpecError) -> Self {
        Self::Render(error)
    }
}

impl std::fmt::Display for RasterSurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroDimension { width, height } => {
                write!(
                    f,
                    "raster surface dimensions must be non-zero, got {width}x{height}"
                )
            }
            Self::StrideTooSmall { minimum, actual } => {
                write!(
                    f,
                    "RGBA8 stride must be at least {minimum} bytes, got {actual}"
                )
            }
            Self::BufferTooSmall { required, actual } => {
                write!(f, "raster surface requires {required} bytes, got {actual}")
            }
            Self::LengthOverflow => f.write_str("raster surface length calculation overflowed"),
            Self::DimensionMismatch {
                expected_width,
                expected_height,
                actual_width,
                actual_height,
            } => write!(
                f,
                "raster surface dimensions must be {expected_width}x{expected_height}, got {actual_width}x{actual_height}"
            ),
            Self::RendererOutputMismatch {
                expected_width,
                expected_height,
                actual_width,
                actual_height,
                expected_len,
                actual_len,
            } => write!(
                f,
                "renderer output must be {expected_width}x{expected_height} with {expected_len} RGBA8 bytes, got {actual_width}x{actual_height} with {actual_len} bytes"
            ),
            Self::Render(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RasterSurfaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            _ => None,
        }
    }
}

/// Immutable prepared 1.x avatar request.
///
/// This preview API binds resolved style, metadata, keys, and every output
/// method to one validated tuple. The current 1.x renderer still allocates an
/// internal `RgbaImage`; the caller-surface and writer methods prepare migration
/// without claiming a zero-allocation renderer.
///
/// # Security
///
/// Cloning a prepared avatar clones its derived identity. Each clone is
/// independently sanitized on drop; high-assurance callers should keep clones
/// short-lived.
#[derive(Clone)]
pub struct PreparedAvatar {
    plan: AvatarRenderPlan,
    resolved_style: ResolvedStyle,
    layout_report: LayoutReport,
    resource_budget: ResourceBudget,
}

impl std::fmt::Debug for PreparedAvatar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedAvatar")
            .field("identity", &"[REDACTED]")
            .field("layout_report", &self.layout_report)
            .field("resource_budget", &self.resource_budget)
            .finish()
    }
}

impl PreparedAvatar {
    pub(crate) fn from_request(request: AvatarRequest) -> Result<Self, AvatarRequestError> {
        let (identity, spec, explicit_style, compatibility) = request.into_parts();
        spec.validate()?;
        let automatically_derived = explicit_style.is_none();
        let requested =
            explicit_style.unwrap_or_else(|| AvatarStyleOptions::from_identity(&identity));
        if explicit_style.is_some() && matches!(compatibility, AvatarCompatibilityMode::Strict) {
            requested.validate_strict()?;
        }
        let effective = requested.canonicalized_for_family();
        let plan = AvatarRenderPlan::from_identity(spec, identity, effective)?;
        Ok(Self::new(plan, requested, effective, automatically_derived))
    }

    pub(crate) fn from_legacy_plan(
        mut plan: AvatarRenderPlan,
        automatically_derived: bool,
    ) -> Self {
        let requested = plan.style();
        let effective = requested.canonicalized_for_family();
        plan.set_style(effective);
        Self::new(plan, requested, effective, automatically_derived)
    }

    fn new(
        plan: AvatarRenderPlan,
        requested: AvatarStyleOptions,
        effective: AvatarStyleOptions,
        automatically_derived: bool,
    ) -> Self {
        let spec = plan.spec();
        let resolved_style = ResolvedStyle {
            requested,
            effective,
            automatically_derived,
        };
        let layout_report = LayoutReport {
            spec,
            style: resolved_style,
            capabilities: effective.kind.capabilities(),
        };
        Self {
            plan,
            resolved_style,
            layout_report,
            resource_budget: ResourceBudget::new(spec),
        }
    }

    /// Returns the prepared image specification.
    pub const fn spec(&self) -> AvatarSpec {
        self.layout_report.spec
    }

    /// Returns requested and effective style information.
    pub const fn resolved_style(&self) -> ResolvedStyle {
        self.resolved_style
    }

    /// Returns stable layout and contract metadata.
    pub const fn layout_report(&self) -> LayoutReport {
        self.layout_report
    }

    /// Returns known RGBA memory requirements for this request.
    pub const fn resource_budget(&self) -> ResourceBudget {
        self.resource_budget
    }

    /// Returns the typed identity-only cache key.
    pub fn identity_cache_key(&self) -> IdentityCacheKey {
        self.plan.identity_cache_key()
    }

    /// Returns the typed key for the complete effective raster/SVG tuple.
    pub fn avatar_asset_key(&self) -> AvatarAssetKey {
        self.plan.avatar_asset_key()
    }

    /// Returns the semantic encoded key for one output format.
    pub fn encoded_asset_key(&self, format: AvatarOutputFormat) -> SemanticEncodedAssetKey {
        self.plan.encoded_asset_key(format)
    }

    /// Returns an encoded key bound to a caller-supplied deployment build.
    pub fn encoded_asset_key_for_build(
        &self,
        format: AvatarOutputFormat,
        build_id: EncoderBuildId,
    ) -> BuildEncodedAssetKey {
        self.plan.encoded_asset_key_for_build(format, build_id)
    }

    /// Renders a new caller-owned RGBA8 image.
    pub fn render(&self) -> Result<RgbaImage, AvatarSpecError> {
        self.plan.render_rgba()
    }

    /// Renders into a validated caller-owned RGBA8 surface.
    ///
    /// The 1.x adapter uses one sanitized internal `RgbaImage` before copying
    /// rows and leaves caller-owned padding bytes unchanged.
    pub fn render_into(
        &self,
        surface: &mut RasterSurfaceMut<'_>,
    ) -> Result<(), RasterSurfaceError> {
        let spec = self.spec();
        let image = SanitizingRgbaImage::new(self.plan.render_rgba()?);
        copy_rgba_image_into_surface(spec, image.as_image(), surface)
    }

    /// Renders SVG into a newly allocated `String`.
    pub fn render_svg(&self) -> String {
        self.plan.render_svg()
    }

    /// Writes SVG bytes to a caller-owned writer.
    ///
    /// The 1.x SVG implementation still builds one temporary `String` before
    /// writing. Partial bytes remain caller-owned if the writer fails.
    pub fn write_svg<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.plan.render_svg().as_bytes())
    }

    /// Encodes into a newly allocated caller-owned byte vector.
    pub fn encode(&self, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
        let image = self
            .plan
            .render_rgba()
            .map_err(avatar_spec_error_to_image_error)?;
        encode_owned_rgba_image(image, format)
    }

    /// Encodes into a caller-owned writer without allocating a returned output `Vec`.
    ///
    /// The 1.x renderer and codecs may still allocate internal raster, scratch,
    /// or quantization buffers. Partial encoded bytes remain caller-owned if
    /// writing or encoding fails.
    pub fn encode_to_writer<W: Write>(
        &self,
        format: AvatarOutputFormat,
        writer: &mut W,
    ) -> ImageResult<()> {
        let image = SanitizingRgbaImage::new(
            self.plan
                .render_rgba()
                .map_err(avatar_spec_error_to_image_error)?,
        );
        encode_into_writer(image.as_image(), format, writer)
    }
}
