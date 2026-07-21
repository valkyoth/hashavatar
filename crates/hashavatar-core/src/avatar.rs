use crate::{
    AvatarKind, AvatarStyle, CanonicalRgbaImage, CatError, LayoutReport, MAX_IDENTITY_BYTES,
    MAX_NAMESPACE_COMPONENT_BYTES, ResolvedStyle, RgbaSurfaceMut, SceneReport, SvgOptions,
    art::compile_avatar_scene,
    identity::TraitDeriver,
    layout::resolve_style,
    raster::{render_scene, render_scene_into},
    scene::Scene,
    svg::{render_scene_svg, render_scene_svg_with, write_scene_svg},
};

const DEFAULT_TENANT: &[u8] = b"public";
const DEFAULT_STYLE_VERSION: &[u8] = b"v2-alpha3";

/// Borrowed inputs for one canonical alpha.4 avatar.
#[must_use = "prepare and render the validated avatar request"]
pub struct AvatarRequest<'a> {
    width: u32,
    height: u32,
    style_seed: u64,
    style: AvatarStyle,
    tenant: &'a [u8],
    style_version: &'a [u8],
    input: &'a [u8],
}

impl<'a> AvatarRequest<'a> {
    /// Creates a request in Hashavatar's canonical baseline namespace.
    pub fn new(
        width: u32,
        height: u32,
        style_seed: u64,
        input: &'a [u8],
        style: AvatarStyle,
    ) -> Result<Self, CatError> {
        Self::with_namespace(
            width,
            height,
            style_seed,
            DEFAULT_TENANT,
            DEFAULT_STYLE_VERSION,
            input,
            style,
        )
    }

    /// Creates a request with explicit tenant and style-version namespaces.
    #[allow(clippy::too_many_arguments)]
    pub fn with_namespace(
        width: u32,
        height: u32,
        style_seed: u64,
        tenant: &'a [u8],
        style_version: &'a [u8],
        input: &'a [u8],
        style: AvatarStyle,
    ) -> Result<Self, CatError> {
        validate_request(width, height, tenant, style_version, input)?;
        Ok(Self {
            width,
            height,
            style_seed,
            style,
            tenant,
            style_version,
            input,
        })
    }

    /// Derives stateless traits and compiles one validated private scene.
    pub fn prepare(self) -> Result<PreparedAvatar, CatError> {
        let deriver = TraitDeriver::with_namespace(
            self.tenant,
            self.style_version,
            self.input,
            self.style_seed,
        )?;
        let traits = AvatarTraitVector::derive(&deriver, self.style.kind())?;
        let (resolved_style, layout_report) = resolve_style(self.style, traits)?;
        let scene = compile_avatar_scene(
            self.width,
            self.height,
            resolved_style,
            &layout_report,
            traits,
        )?;
        let report = scene.validate()?;
        Ok(PreparedAvatar {
            scene,
            requested_style: self.style,
            resolved_style,
            layout_report,
            traits,
            report,
        })
    }
}

/// Named stateless samples used by all alpha.4 family and style compilers.
#[must_use = "trait vectors describe deterministic avatar appearance"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarTraitVector {
    proportion_a: u16,
    proportion_b: u16,
    detail_a: u16,
    detail_b: u16,
    primary_hue: u16,
    secondary_hue: u16,
    accent_hue: u16,
    pattern_seed: u16,
}

impl AvatarTraitVector {
    fn derive(deriver: &TraitDeriver, kind: AvatarKind) -> Result<Self, CatError> {
        let scope = kind.as_str().as_bytes();
        Ok(Self {
            proportion_a: deriver.sample_scoped(scope, b"proportion-a")?,
            proportion_b: deriver.sample_scoped(scope, b"proportion-b")?,
            detail_a: deriver.sample_scoped(scope, b"detail-a")?,
            detail_b: deriver.sample_scoped(scope, b"detail-b")?,
            primary_hue: deriver.sample_scoped(scope, b"primary-hue")?,
            secondary_hue: deriver.sample_scoped(scope, b"secondary-hue")?,
            accent_hue: deriver.sample_scoped(scope, b"accent-hue")?,
            pattern_seed: deriver.sample_scoped(scope, b"pattern-seed")?,
        })
    }

    /// Returns the first family proportion sample.
    pub const fn proportion_a(self) -> u16 {
        self.proportion_a
    }
    /// Returns the second family proportion sample.
    pub const fn proportion_b(self) -> u16 {
        self.proportion_b
    }
    /// Returns the first family detail sample.
    pub const fn detail_a(self) -> u16 {
        self.detail_a
    }
    /// Returns the second family detail sample.
    pub const fn detail_b(self) -> u16 {
        self.detail_b
    }
    /// Returns the primary color sample.
    pub const fn primary_hue(self) -> u16 {
        self.primary_hue
    }
    /// Returns the secondary color sample.
    pub const fn secondary_hue(self) -> u16 {
        self.secondary_hue
    }
    /// Returns the accent color sample.
    pub const fn accent_hue(self) -> u16 {
        self.accent_hue
    }
    /// Returns the deterministic pattern sample.
    pub const fn pattern_seed(self) -> u16 {
        self.pattern_seed
    }
}

/// Prepared alpha.4 scene shared by CPU and SVG executors.
#[must_use = "render or inspect the prepared avatar scene"]
pub struct PreparedAvatar {
    scene: Scene,
    requested_style: AvatarStyle,
    resolved_style: ResolvedStyle,
    layout_report: LayoutReport,
    traits: AvatarTraitVector,
    report: SceneReport,
}

impl PreparedAvatar {
    /// Returns the canonical output width.
    pub const fn width(&self) -> u32 {
        self.scene.width()
    }
    /// Returns the canonical output height.
    pub const fn height(&self) -> u32 {
        self.scene.height()
    }
    /// Returns the caller-requested style before compatibility resolution.
    pub const fn style(&self) -> AvatarStyle {
        self.requested_style
    }

    /// Returns the immutable effective style consumed by scene compilation.
    pub const fn resolved_style(&self) -> ResolvedStyle {
        self.resolved_style
    }

    /// Returns deterministic compatibility and placement decisions.
    pub const fn layout_report(&self) -> LayoutReport {
        self.layout_report
    }
    /// Returns the named stateless trait samples.
    pub const fn trait_vector(&self) -> AvatarTraitVector {
        self.traits
    }
    /// Returns the validated scene resource estimate.
    pub const fn scene_report(&self) -> SceneReport {
        self.report
    }

    /// Executes the canonical safe-Rust CPU rasterizer.
    pub fn render_rgba(&self) -> Result<CanonicalRgbaImage, CatError> {
        render_scene(&self.scene)
    }

    /// Executes into a validated caller-owned RGBA8 surface.
    pub fn render_into(&self, surface: &mut RgbaSurfaceMut<'_>) -> Result<(), CatError> {
        render_scene_into(&self.scene, surface)
    }

    /// Serializes the canonical scene as deterministic SVG.
    pub fn render_svg(&self) -> Result<alloc::string::String, CatError> {
        render_scene_svg(&self.scene)
    }

    /// Serializes the scene with document or fragment options.
    pub fn render_svg_with(
        &self,
        options: SvgOptions<'_>,
    ) -> Result<alloc::string::String, CatError> {
        render_scene_svg_with(&self.scene, options)
    }

    /// Streams SVG to a [`core::fmt::Write`] destination.
    pub fn write_svg(
        &self,
        writer: &mut impl core::fmt::Write,
        options: SvgOptions<'_>,
    ) -> Result<(), CatError> {
        write_scene_svg(&self.scene, writer, options)
    }
}

fn validate_request(
    width: u32,
    height: u32,
    tenant: &[u8],
    style_version: &[u8],
    input: &[u8],
) -> Result<(), CatError> {
    let _ = Scene::new(width, height)?;
    for (component, bytes, maximum) in [
        (crate::IdentityComponent::Input, input, MAX_IDENTITY_BYTES),
        (
            crate::IdentityComponent::Tenant,
            tenant,
            MAX_NAMESPACE_COMPONENT_BYTES,
        ),
        (
            crate::IdentityComponent::StyleVersion,
            style_version,
            MAX_NAMESPACE_COMPONENT_BYTES,
        ),
    ] {
        if bytes.len() > maximum {
            return Err(CatError::IdentityComponentTooLong { component, maximum });
        }
    }
    Ok(())
}
