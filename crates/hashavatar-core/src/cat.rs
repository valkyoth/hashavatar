use crate::{
    CanonicalRgbaImage, CatError, MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES, SceneReport,
    fixed::Fixed,
    identity::TraitDeriver,
    raster::render_scene,
    scene::{Color, Command, Point, Scene},
    svg::render_scene_svg,
};

const DEFAULT_TENANT: &[u8] = b"public";
const DEFAULT_STYLE: &[u8] = b"v2-alpha1";

/// Borrowed inputs for one canonical alpha.1 Cat avatar.
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
    /// Creates a request in Hashavatar's public alpha.1 namespace.
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

/// Stable named trait samples used by the alpha.1 Cat compiler.
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

    /// Serializes the same canonical scene as deterministic SVG.
    pub fn render_svg(&self) -> Result<alloc::string::String, CatError> {
        render_scene_svg(&self.scene)
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

fn compile_scene(width: u32, height: u32, traits: CatTraitVector) -> Result<Scene, CatError> {
    let width = Fixed::from_integer(i32::try_from(width).map_err(|_| CatError::NumericRange)?)?;
    let height = Fixed::from_integer(i32::try_from(height).map_err(|_| CatError::NumericRange)?)?;
    let center_x = scale(width, 50, 100)?;
    let center_y = scale(height, 54, 100)?.checked_add(vary(height, 0, 4, traits.head_drop)?)?;
    let head_rx = scale(width, 27, 100)?.checked_add(vary(width, 0, 5, traits.head_width)?)?;
    let head_ry = scale(height, 23, 100)?.checked_add(vary(height, 0, 5, traits.head_height)?)?;
    let ear_half = scale(width, 11, 100)?.checked_add(vary(width, 0, 3, traits.ear_width)?)?;
    let ear_height = scale(height, 18, 100)?.checked_add(vary(height, 0, 5, traits.ear_height)?)?;
    let eye_offset = scale(width, 12, 100)?.checked_add(vary(width, 0, 3, traits.eye_spacing)?)?;
    let eye_rx = scale(width, 4, 100)?.checked_add(vary(width, 0, 2, traits.eye_size)?)?;
    let eye_ry = scale(height, 6, 100)?.checked_add(vary(height, 0, 2, traits.eye_size)?)?;
    let eye_y = center_y.checked_sub(scale(head_ry, 18, 100)?)?;
    let ear_base_y = center_y.checked_sub(scale(head_ry, 72, 100)?)?;
    let ear_tip_y = ear_base_y.checked_sub(ear_height)?;
    let left_ear_x = center_x.checked_sub(scale(head_rx, 58, 100)?)?;
    let right_ear_x = center_x.checked_add(scale(head_rx, 58, 100)?)?;

    let background = themed_color(traits.background_hue, 36, 84, 72);
    let accent = themed_color(traits.accent_hue, 48, 94, 86);
    let fur = themed_color(traits.fur_hue, 54, 210, 188);
    let muzzle = themed_color(traits.muzzle_hue, 168, 246, 222);
    let iris = themed_color(traits.eye_hue, 36, 224, 170);
    let ink = Color::rgb(24, 27, 32);

    let mut scene = Scene::new(
        u32::try_from(width.floor()?).map_err(|_| CatError::NumericRange)?,
        u32::try_from(height.floor()?).map_err(|_| CatError::NumericRange)?,
    )?;
    scene.push(Command::Fill(background))?;
    scene.push(Command::Ellipse {
        center: Point::new(scale(width, 82, 100)?, scale(height, 18, 100)?),
        radius_x: scale(width, 18, 100)?,
        radius_y: scale(height, 18, 100)?,
        color: accent,
    })?;
    scene.push(Command::Triangle {
        points: [
            Point::new(left_ear_x.checked_sub(ear_half)?, ear_base_y),
            Point::new(left_ear_x, ear_tip_y),
            Point::new(left_ear_x.checked_add(ear_half)?, ear_base_y),
        ],
        color: fur,
    })?;
    scene.push(Command::Triangle {
        points: [
            Point::new(right_ear_x.checked_sub(ear_half)?, ear_base_y),
            Point::new(right_ear_x, ear_tip_y),
            Point::new(right_ear_x.checked_add(ear_half)?, ear_base_y),
        ],
        color: fur,
    })?;
    scene.push(Command::Ellipse {
        center: Point::new(center_x, center_y),
        radius_x: head_rx,
        radius_y: head_ry,
        color: fur,
    })?;
    scene.push(Command::Ellipse {
        center: Point::new(center_x, center_y.checked_add(scale(head_ry, 35, 100)?)?),
        radius_x: scale(head_rx, 38, 100)?,
        radius_y: scale(head_ry, 28, 100)?,
        color: muzzle,
    })?;
    for direction in [-1_i32, 1_i32] {
        let offset = if direction < 0 {
            Fixed::ZERO.checked_sub(eye_offset)?
        } else {
            eye_offset
        };
        let eye_x = center_x.checked_add(offset)?;
        scene.push(Command::Ellipse {
            center: Point::new(eye_x, eye_y),
            radius_x: eye_rx,
            radius_y: eye_ry,
            color: Color::rgb(248, 250, 252),
        })?;
        scene.push(Command::Ellipse {
            center: Point::new(eye_x, eye_y),
            radius_x: scale(eye_rx, 48, 100)?,
            radius_y: scale(eye_ry, 62, 100)?,
            color: iris,
        })?;
        scene.push(Command::Ellipse {
            center: Point::new(eye_x, eye_y),
            radius_x: scale(eye_rx, 18, 100)?,
            radius_y: scale(eye_ry, 44, 100)?,
            color: ink,
        })?;
    }
    let nose_y = center_y.checked_add(scale(head_ry, 31, 100)?)?;
    scene.push(Command::Triangle {
        points: [
            Point::new(center_x.checked_sub(scale(width, 3, 100)?)?, nose_y),
            Point::new(center_x.checked_add(scale(width, 3, 100)?)?, nose_y),
            Point::new(center_x, nose_y.checked_add(scale(height, 3, 100)?)?),
        ],
        color: ink,
    })?;
    Ok(scene)
}

fn scale(value: Fixed, numerator: i32, denominator: i32) -> Result<Fixed, CatError> {
    value.checked_mul(Fixed::from_ratio(numerator, denominator)?)
}

fn vary(
    value: Fixed,
    minimum_percent: i32,
    maximum_percent: i32,
    sample: u16,
) -> Result<Fixed, CatError> {
    let minimum = scale(value, minimum_percent, 100)?;
    let maximum = scale(value, maximum_percent, 100)?;
    Fixed::lerp(minimum, maximum, sample)
}

fn themed_color(sample: u16, floor: u8, ceiling: u8, phase: u8) -> Color {
    let span = u16::from(ceiling.saturating_sub(floor));
    let channel = |shift: u16| {
        let mixed = sample.rotate_left(u32::from(shift % 16));
        let scaled = u32::from(mixed)
            .saturating_mul(u32::from(span))
            .checked_div(u32::from(u16::MAX))
            .unwrap_or(0);
        let bounded = u8::try_from(scaled).unwrap_or(u8::MAX);
        floor.saturating_add(bounded)
    };
    Color::rgb(channel(0), channel(u16::from(phase % 13)), channel(11))
}
