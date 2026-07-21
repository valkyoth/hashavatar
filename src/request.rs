use super::*;

/// Style compatibility policy used while preparing an avatar request.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarCompatibilityMode {
    /// Reject explicit style layers that the selected family cannot render.
    #[default]
    Strict,
    /// Preserve the Hashavatar 1.x behavior of ignoring unsupported layers.
    LegacyV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RequestStyle {
    Explicit(AvatarStyleOptions),
    Automatic,
}

/// Immutable input to the prepared-avatar preview API.
///
/// The request owns an already-derived [`AvatarIdentity`], so it never retains
/// the caller's raw identity input. Call [`AvatarRequest::prepare`] to bind
/// validation, style resolution, resource reporting, keys, and rendering to
/// one immutable tuple.
///
/// # Security
///
/// Cloning a request clones its derived identity. Each clone is independently
/// sanitized on drop; high-assurance callers should keep clones short-lived.
#[derive(Clone)]
pub struct AvatarRequest {
    identity: AvatarIdentity,
    spec: AvatarSpec,
    style: RequestStyle,
    compatibility: AvatarCompatibilityMode,
}

impl std::fmt::Debug for AvatarRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvatarRequest")
            .field("identity", &"[REDACTED]")
            .field("spec", &self.spec)
            .field("style", &self.style)
            .field("compatibility", &self.compatibility)
            .finish()
    }
}

impl AvatarRequest {
    /// Starts a request builder from an already validated identity.
    pub fn builder(identity: AvatarIdentity) -> AvatarRequestBuilder {
        AvatarRequestBuilder::new(identity)
    }

    /// Creates a strict request with an explicit style.
    pub fn new(identity: AvatarIdentity, spec: AvatarSpec, style: AvatarStyleOptions) -> Self {
        Self {
            identity,
            spec,
            style: RequestStyle::Explicit(style),
            compatibility: AvatarCompatibilityMode::Strict,
        }
    }

    /// Creates a request whose style is derived from the identity.
    pub fn automatic(identity: AvatarIdentity, spec: AvatarSpec) -> Self {
        Self {
            identity,
            spec,
            style: RequestStyle::Automatic,
            compatibility: AvatarCompatibilityMode::Strict,
        }
    }

    /// Returns the request's validated image specification.
    pub const fn spec(&self) -> AvatarSpec {
        self.spec
    }

    /// Returns the style compatibility policy applied during preparation.
    pub const fn compatibility_mode(&self) -> AvatarCompatibilityMode {
        self.compatibility
    }

    /// Returns the explicit style, or `None` for automatic derivation.
    pub const fn explicit_style(&self) -> Option<AvatarStyleOptions> {
        match self.style {
            RequestStyle::Explicit(style) => Some(style),
            RequestStyle::Automatic => None,
        }
    }

    /// Prepares and freezes this request.
    pub fn prepare(self) -> Result<PreparedAvatar, AvatarRequestError> {
        PreparedAvatar::from_request(self)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        AvatarIdentity,
        AvatarSpec,
        Option<AvatarStyleOptions>,
        AvatarCompatibilityMode,
    ) {
        let style = match self.style {
            RequestStyle::Explicit(style) => Some(style),
            RequestStyle::Automatic => None,
        };
        (self.identity, self.spec, style, self.compatibility)
    }
}

/// Builder for [`AvatarRequest`].
///
/// Explicit styles are validated strictly by default. Use
/// [`AvatarRequestBuilder::legacy_v1_compatibility`] only when reproducing the
/// 1.x skip-on-unsupported behavior is required during migration.
///
/// Cloning the builder clones its derived identity. Each clone is independently
/// sanitized on drop.
#[derive(Clone)]
pub struct AvatarRequestBuilder {
    identity: AvatarIdentity,
    spec: Result<AvatarSpec, AvatarSpecError>,
    style: RequestStyle,
    compatibility: AvatarCompatibilityMode,
}

impl std::fmt::Debug for AvatarRequestBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvatarRequestBuilder")
            .field("identity", &"[REDACTED]")
            .field("spec", &self.spec)
            .field("style", &self.style)
            .field("compatibility", &self.compatibility)
            .finish()
    }
}

impl AvatarRequestBuilder {
    /// Starts a strict request builder from an already-derived identity.
    pub fn new(identity: AvatarIdentity) -> Self {
        Self {
            identity,
            spec: Ok(AvatarSpec::default()),
            style: RequestStyle::Explicit(AvatarStyleOptions::default()),
            compatibility: AvatarCompatibilityMode::Strict,
        }
    }

    /// Replaces the request's validated image specification.
    pub fn spec(mut self, spec: AvatarSpec) -> Self {
        self.spec = Ok(spec);
        self
    }

    /// Validates and sets image dimensions while retaining the current seed.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        let seed = self.spec.ok().map(AvatarSpec::seed).unwrap_or_default();
        self.spec = AvatarSpec::new(width, height, seed);
        self
    }

    /// Sets the caller-controlled deterministic style-variant seed.
    pub fn style_variant(mut self, seed: u64) -> Self {
        self.spec = match self.spec {
            Ok(spec) => AvatarSpec::new(spec.width(), spec.height(), seed),
            Err(error) => Err(error),
        };
        self
    }

    /// Alias for [`AvatarRequestBuilder::style_variant`].
    pub fn seed(self, seed: u64) -> Self {
        self.style_variant(seed)
    }

    /// Derives the complete style from the identity during preparation.
    pub fn automatic_style(mut self) -> Self {
        self.style = RequestStyle::Automatic;
        self
    }

    /// Uses one complete explicit style.
    pub fn style(mut self, style: AvatarStyleOptions) -> Self {
        self.style = RequestStyle::Explicit(style);
        self
    }

    /// Uses legacy kind/background options with neutral extra layers.
    pub fn options(self, options: AvatarOptions) -> Self {
        self.style(AvatarStyleOptions::from_options(options))
    }

    /// Sets the explicit avatar family.
    pub fn kind(self, kind: AvatarKind) -> Self {
        self.with_style(|style| style.kind = kind)
    }

    /// Sets the explicit background.
    pub fn background(self, background: AvatarBackground) -> Self {
        self.with_style(|style| style.background = background)
    }

    /// Sets the explicit accessory.
    pub fn accessory(self, accessory: AvatarAccessory) -> Self {
        self.with_style(|style| style.accessory = accessory)
    }

    /// Sets the explicit accent palette.
    pub fn color(self, color: AvatarColor) -> Self {
        self.with_style(|style| style.color = color)
    }

    /// Sets the explicit expression.
    pub fn expression(self, expression: AvatarExpression) -> Self {
        self.with_style(|style| style.expression = expression)
    }

    /// Sets the explicit frame shape.
    pub fn shape(self, shape: AvatarShape) -> Self {
        self.with_style(|style| style.shape = shape)
    }

    /// Rejects explicit layers unsupported by the selected family.
    pub fn strict_style_validation(mut self) -> Self {
        self.compatibility = AvatarCompatibilityMode::Strict;
        self
    }

    /// Reproduces the 1.x skip-on-unsupported style behavior.
    pub fn legacy_v1_compatibility(mut self) -> Self {
        self.compatibility = AvatarCompatibilityMode::LegacyV1;
        self
    }

    /// Builds the immutable request, returning any deferred spec error.
    pub fn build(self) -> Result<AvatarRequest, AvatarRequestError> {
        Ok(AvatarRequest {
            identity: self.identity,
            spec: self.spec?,
            style: self.style,
            compatibility: self.compatibility,
        })
    }

    /// Builds, validates, resolves, and freezes the request.
    pub fn prepare(self) -> Result<PreparedAvatar, AvatarRequestError> {
        self.build()?.prepare()
    }

    fn with_style(mut self, update: impl FnOnce(&mut AvatarStyleOptions)) -> Self {
        let mut style = match self.style {
            RequestStyle::Explicit(style) => style,
            RequestStyle::Automatic => AvatarStyleOptions::default(),
        };
        update(&mut style);
        self.style = RequestStyle::Explicit(style);
        self
    }
}

/// Error returned when request preparation fails.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvatarRequestError {
    /// The image specification is outside the supported bounds.
    Spec(AvatarSpecError),
    /// An explicit style requests a layer unsupported by its family.
    Style(AvatarStyleValidationError),
}

impl From<AvatarSpecError> for AvatarRequestError {
    fn from(error: AvatarSpecError) -> Self {
        Self::Spec(error)
    }
}

impl From<AvatarStyleValidationError> for AvatarRequestError {
    fn from(error: AvatarStyleValidationError) -> Self {
        Self::Style(error)
    }
}

impl std::fmt::Display for AvatarRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spec(error) => error.fmt(f),
            Self::Style(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for AvatarRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spec(error) => Some(error),
            Self::Style(error) => Some(error),
        }
    }
}
