mod compiler;

use self::compiler::compile_scene;
use crate::{
    CanonicalRgbaImage, CatError, MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES,
    RgbaSurfaceMut, SceneReport, SvgOptions,
    identity::TraitDeriver,
    raster::{render_scene, render_scene_into},
    scene::Scene,
    svg::{render_scene_svg, render_scene_svg_with, write_scene_svg},
};

const DEFAULT_TENANT: &[u8] = b"public";
const DEFAULT_STYLE: &[u8] = b"v2-alpha2";

/// Borrowed inputs for one canonical alpha.2 Cat avatar.
///
/// Construction validates public resource bounds. [`Self::prepare`] derives
/// traits and compiles the request into a validated private scene without
/// retaining the identity or namespace bytes.
#[must_use = "prepare and render the validated Cat request"]
pub struct CatRequest<'a> {
    width: u32,
    height: u32,
    style_seed: u64,
    tenant: &'a [u8],
    style_version: &'a [u8],
    input: &'a [u8],
}

impl<'a> CatRequest<'a> {
    /// Creates a request in Hashavatar's public alpha.2 namespace.
    pub fn new(
        width: u32,
        height: u32,
        style_seed: u64,
        input: &'a [u8],
    ) -> Result<Self, CatError> {
        Self::with_namespace(
            width,
            height,
            style_seed,
            DEFAULT_TENANT,
            DEFAULT_STYLE,
            input,
        )
    }

    /// Creates a request with explicit tenant and style-version namespaces.
    pub fn with_namespace(
        width: u32,
        height: u32,
        style_seed: u64,
        tenant: &'a [u8],
        style_version: &'a [u8],
        input: &'a [u8],
    ) -> Result<Self, CatError> {
        validate_request(width, height, tenant, style_version, input)?;
        Ok(Self {
            width,
            height,
            style_seed,
            tenant,
            style_version,
            input,
        })
    }

    /// Derives stateless traits and compiles one validated Cat scene.
    pub fn prepare(self) -> Result<PreparedCat, CatError> {
        let deriver = TraitDeriver::with_namespace(
            self.tenant,
            self.style_version,
            self.input,
            self.style_seed,
        )?;
        let traits = CatTraitVector::derive(&deriver)?;
        let scene = compile_scene(self.width, self.height, traits)?;
        let report = scene.validate()?;
        Ok(PreparedCat {
            scene,
            traits,
            report,
        })
    }
}

/// Stable named trait samples used by the alpha.2 Cat compiler.
///
/// Each value is independently derived from a domain-separated label. Values
/// are exposed for reproducibility and diagnostics; the identity digest is not.
#[must_use = "trait vectors describe deterministic Cat appearance"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatTraitVector {
    head_width: u16,
    head_height: u16,
    head_drop: u16,
    ear_width: u16,
    ear_height: u16,
    eye_spacing: u16,
    eye_size: u16,
    background_hue: u16,
    accent_hue: u16,
    fur_hue: u16,
    eye_hue: u16,
    muzzle_hue: u16,
}

impl CatTraitVector {
    fn derive(deriver: &TraitDeriver) -> Result<Self, CatError> {
        Ok(Self {
            head_width: deriver.sample(b"cat/head-width")?,
            head_height: deriver.sample(b"cat/head-height")?,
            head_drop: deriver.sample(b"cat/head-drop")?,
            ear_width: deriver.sample(b"cat/ear-width")?,
            ear_height: deriver.sample(b"cat/ear-height")?,
            eye_spacing: deriver.sample(b"cat/eye-spacing")?,
            eye_size: deriver.sample(b"cat/eye-size")?,
            background_hue: deriver.sample(b"cat/background-hue")?,
            accent_hue: deriver.sample(b"cat/accent-hue")?,
            fur_hue: deriver.sample(b"cat/fur-hue")?,
            eye_hue: deriver.sample(b"cat/eye-hue")?,
            muzzle_hue: deriver.sample(b"cat/muzzle-hue")?,
        })
    }

    /// Returns the head-width sample.
    pub const fn head_width(self) -> u16 {
        self.head_width
    }

    /// Returns the head-height sample.
    pub const fn head_height(self) -> u16 {
        self.head_height
    }

    /// Returns the vertical head-position sample.
    pub const fn head_drop(self) -> u16 {
        self.head_drop
    }

    /// Returns the ear-width sample.
    pub const fn ear_width(self) -> u16 {
        self.ear_width
    }

    /// Returns the ear-height sample.
    pub const fn ear_height(self) -> u16 {
        self.ear_height
    }

    /// Returns the eye-spacing sample.
    pub const fn eye_spacing(self) -> u16 {
        self.eye_spacing
    }

    /// Returns the eye-size sample.
    pub const fn eye_size(self) -> u16 {
        self.eye_size
    }

    /// Returns the themed-background color sample.
    pub const fn background_hue(self) -> u16 {
        self.background_hue
    }

    /// Returns the background-accent color sample.
    pub const fn accent_hue(self) -> u16 {
        self.accent_hue
    }

    /// Returns the fur color sample.
    pub const fn fur_hue(self) -> u16 {
        self.fur_hue
    }

    /// Returns the eye color sample.
    pub const fn eye_hue(self) -> u16 {
        self.eye_hue
    }

    /// Returns the muzzle color sample.
    pub const fn muzzle_hue(self) -> u16 {
        self.muzzle_hue
    }
}

/// Prepared canonical Cat scene shared by the CPU and SVG executors.
#[must_use = "render or inspect the prepared Cat scene"]
pub struct PreparedCat {
    scene: Scene,
    traits: CatTraitVector,
    report: SceneReport,
}

impl PreparedCat {
    /// Returns the canonical output width.
    pub const fn width(&self) -> u32 {
        self.scene.width()
    }

    /// Returns the canonical output height.
    pub const fn height(&self) -> u32 {
        self.scene.height()
    }

    /// Returns the named stateless trait samples.
    pub const fn trait_vector(&self) -> CatTraitVector {
        self.traits
    }

    /// Returns the validated scene's bounded resource estimate.
    pub const fn scene_report(&self) -> SceneReport {
        self.report
    }

    /// Executes the canonical safe-Rust CPU rasterizer as straight-alpha RGBA8.
    pub fn render_rgba(&self) -> Result<CanonicalRgbaImage, CatError> {
        render_scene(&self.scene)
    }

    /// Executes into a validated caller-owned RGBA8 surface.
    ///
    /// Validation occurs before visible bytes are changed. An execution error
    /// may leave visible pixels partially modified; row padding is preserved.
    pub fn render_into(&self, surface: &mut RgbaSurfaceMut<'_>) -> Result<(), CatError> {
        render_scene_into(&self.scene, surface)
    }

    /// Serializes the same canonical scene as deterministic SVG.
    pub fn render_svg(&self) -> Result<alloc::string::String, CatError> {
        render_scene_svg(&self.scene)
    }

    /// Serializes the scene using validated document or fragment options.
    pub fn render_svg_with(
        &self,
        options: SvgOptions<'_>,
    ) -> Result<alloc::string::String, CatError> {
        render_scene_svg_with(&self.scene, options)
    }

    /// Streams SVG to a [`core::fmt::Write`] destination.
    ///
    /// Writer failure may leave a valid prefix in the destination. Retry with
    /// a fresh destination.
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
    if input.len() > MAX_IDENTITY_BYTES {
        return Err(CatError::IdentityComponentTooLong {
            component: crate::IdentityComponent::Input,
            maximum: MAX_IDENTITY_BYTES,
        });
    }
    for (component, bytes) in [
        (crate::IdentityComponent::Tenant, tenant),
        (crate::IdentityComponent::StyleVersion, style_version),
    ] {
        if bytes.len() > MAX_NAMESPACE_COMPONENT_BYTES {
            return Err(CatError::IdentityComponentTooLong {
                component,
                maximum: MAX_NAMESPACE_COMPONENT_BYTES,
            });
        }
    }
    Ok(())
}
