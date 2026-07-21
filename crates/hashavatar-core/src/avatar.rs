use crate::{
    AvatarAssetKey, AvatarError, AvatarIdentity, AvatarKind, AvatarStyle, CanonicalRgbaImage,
    CatalogVersion, IdentityCacheKey, LayoutReport, RenderContractId, ResolvedStyle,
    ResourceBudget, ReusableRgbaBuffer, RgbaSurfaceMut, SceneReport, SvgOptions,
    art::compile_avatar_scene,
    identity::TraitDeriver,
    layout::resolve_style,
    raster::{render_scene, render_scene_into},
    scene::Scene,
    svg::{render_scene_svg, render_scene_svg_with, write_scene_svg},
};

/// Owned validated inputs for one canonical alpha.5 avatar.
#[must_use = "prepare and render the validated avatar request"]
pub struct AvatarRequest {
    width: u32,
    height: u32,
    style_seed: u64,
    style: AvatarStyle,
    identity: AvatarIdentity,
}

impl AvatarRequest {
    /// Creates a request in Hashavatar's canonical baseline namespace.
    pub fn new(
        width: u32,
        height: u32,
        style_seed: u64,
        input: impl AsRef<[u8]>,
        style: AvatarStyle,
    ) -> Result<Self, AvatarError> {
        Self::from_identity(
            width,
            height,
            style_seed,
            AvatarIdentity::new(input)?,
            style,
        )
    }

    /// Creates a request with explicit tenant and style-version namespaces.
    #[allow(clippy::too_many_arguments)]
    pub fn with_namespace(
        width: u32,
        height: u32,
        style_seed: u64,
        tenant: impl AsRef<[u8]>,
        style_version: impl AsRef<[u8]>,
        input: impl AsRef<[u8]>,
        style: AvatarStyle,
    ) -> Result<Self, AvatarError> {
        Self::from_identity(
            width,
            height,
            style_seed,
            AvatarIdentity::with_namespace(tenant, style_version, input)?,
            style,
        )
    }

    /// Creates a request from an already validated identity.
    pub fn from_identity(
        width: u32,
        height: u32,
        style_seed: u64,
        identity: AvatarIdentity,
        style: AvatarStyle,
    ) -> Result<Self, AvatarError> {
        let _ = Scene::new(width, height)?;
        Ok(Self {
            width,
            height,
            style_seed,
            style,
            identity,
        })
    }

    /// Starts the recommended request builder from a validated identity.
    pub fn builder(identity: AvatarIdentity) -> AvatarRequestBuilder {
        AvatarRequestBuilder::new(identity)
    }

    /// Derives stateless traits and compiles one validated private scene.
    pub fn prepare(self) -> Result<PreparedAvatar, AvatarError> {
        let deriver = TraitDeriver::new(self.identity, self.style_seed);
        let traits = AvatarTraitVector::derive(&deriver, self.style.kind())?;
        let (resolved_style, layout_report) = resolve_style(self.style, traits)?;
        let identity_cache_key = deriver.identity().cache_key()?;
        let asset_key = crate::keys::avatar_asset_key(
            deriver.identity(),
            self.width,
            self.height,
            self.style_seed,
            resolved_style,
        )?;
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
            identity_cache_key,
            asset_key,
        })
    }
}

impl core::fmt::Debug for AvatarRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("AvatarRequest")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("style_seed", &self.style_seed)
            .field("style", &self.style)
            .field("identity", &"[REDACTED]")
            .finish()
    }
}

/// Builder for one validated owned [`AvatarRequest`].
#[must_use = "call build or prepare"]
pub struct AvatarRequestBuilder {
    identity: AvatarIdentity,
    width: u32,
    height: u32,
    style_seed: u64,
    style: AvatarStyle,
}

impl AvatarRequestBuilder {
    /// Starts a 256x256 Cat request in the themed square style.
    pub fn new(identity: AvatarIdentity) -> Self {
        Self {
            identity,
            width: 256,
            height: 256,
            style_seed: 0,
            style: AvatarStyle::default(),
        }
    }

    /// Sets dimensions, validated when the request is built.
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the deterministic style-variant seed.
    pub const fn style_variant(mut self, style_seed: u64) -> Self {
        self.style_seed = style_seed;
        self
    }

    /// Sets one complete explicit or automatic style.
    pub const fn style(mut self, style: AvatarStyle) -> Self {
        self.style = style;
        self
    }

    /// Builds the validated owned request.
    pub fn build(self) -> Result<AvatarRequest, AvatarError> {
        AvatarRequest::from_identity(
            self.width,
            self.height,
            self.style_seed,
            self.identity,
            self.style,
        )
    }

    /// Builds, resolves, and compiles the request transactionally.
    pub fn prepare(self) -> Result<PreparedAvatar, AvatarError> {
        self.build()?.prepare()
    }
}

/// Named stateless samples used by all canonical family and style compilers.
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
    fn derive(deriver: &TraitDeriver, kind: AvatarKind) -> Result<Self, AvatarError> {
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

/// Prepared alpha.5 scene shared by CPU, SVG, and format executors.
#[must_use = "render or inspect the prepared avatar scene"]
pub struct PreparedAvatar {
    scene: Scene,
    requested_style: AvatarStyle,
    resolved_style: ResolvedStyle,
    layout_report: LayoutReport,
    traits: AvatarTraitVector,
    report: SceneReport,
    identity_cache_key: IdentityCacheKey,
    asset_key: AvatarAssetKey,
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

    /// Returns conservative scene, CPU, scratch, and owned-output limits.
    pub const fn resource_budget(&self) -> ResourceBudget {
        ResourceBudget::new(self.report)
    }

    /// Returns the public correlatable identity cache key bound at preparation.
    pub const fn identity_cache_key(&self) -> IdentityCacheKey {
        self.identity_cache_key
    }

    /// Returns the public key for this complete canonical render tuple.
    pub const fn asset_key(&self) -> AvatarAssetKey {
        self.asset_key
    }

    /// Returns the catalog contract bound into style resolution and asset keys.
    pub const fn catalog_version(&self) -> CatalogVersion {
        CatalogVersion::CURRENT
    }

    /// Returns the canonical renderer contract bound into asset keys.
    pub const fn render_contract_id(&self) -> RenderContractId {
        RenderContractId::CURRENT
    }

    /// Executes the canonical safe-Rust CPU rasterizer.
    pub fn render_rgba(&self) -> Result<CanonicalRgbaImage, AvatarError> {
        render_scene(&self.scene)
    }

    /// Executes into a validated caller-owned RGBA8 surface.
    pub fn render_into(&self, surface: &mut RgbaSurfaceMut<'_>) -> Result<(), AvatarError> {
        render_scene_into(&self.scene, surface)
    }

    /// Executes into allocation-reusing Hashavatar-owned RGBA8 storage.
    pub fn render_reusing(&self, scratch: &mut ReusableRgbaBuffer) -> Result<(), AvatarError> {
        scratch.prepare(self.width(), self.height())?;
        self.render_into(&mut scratch.surface_mut()?)
    }

    /// Serializes the canonical scene as deterministic SVG.
    pub fn render_svg(&self) -> Result<alloc::string::String, AvatarError> {
        render_scene_svg(&self.scene)
    }

    /// Serializes the scene with document or fragment options.
    pub fn render_svg_with(
        &self,
        options: SvgOptions<'_>,
    ) -> Result<alloc::string::String, AvatarError> {
        render_scene_svg_with(&self.scene, options)
    }

    /// Streams SVG to a [`core::fmt::Write`] destination.
    pub fn write_svg(
        &self,
        writer: &mut impl core::fmt::Write,
        options: SvgOptions<'_>,
    ) -> Result<(), AvatarError> {
        write_scene_svg(&self.scene, writer, options)
    }
}
