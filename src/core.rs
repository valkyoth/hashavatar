use super::*;

/// Input parameters for a generated avatar image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpec {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) seed: u64,
}

impl AvatarSpec {
    /// Creates a validated avatar specification.
    ///
    /// The `seed` is a caller-controlled style variant mixed into the
    /// identity-derived renderer RNG. It does not replace identity hashing:
    /// the same `(namespace, identity, style, size, seed)` tuple is stable, and
    /// changing only `seed` deliberately produces a different visual variant.
    pub const fn new(width: u32, height: u32, seed: u64) -> Result<Self, AvatarSpecError> {
        if Self::dimensions_are_supported(width, height) {
            Ok(Self {
                width,
                height,
                seed,
            })
        } else {
            Err(AvatarSpecError { width, height })
        }
    }

    pub(crate) const fn new_unchecked(width: u32, height: u32, seed: u64) -> Self {
        Self {
            width,
            height,
            seed,
        }
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }

    pub const fn seed(self) -> u64 {
        self.seed
    }

    pub const fn pixel_count(self) -> usize {
        (self.width as usize).saturating_mul(self.height as usize)
    }

    pub const fn rgba_buffer_len(self) -> usize {
        self.pixel_count()
            .saturating_mul(AVATAR_RGBA_BYTES_PER_PIXEL)
    }

    pub const fn render_resource_budget(
        self,
        concurrent_renders: usize,
    ) -> AvatarRenderResourceBudget {
        AvatarRenderResourceBudget::new(self, concurrent_renders)
    }

    pub const fn is_supported(self) -> bool {
        Self::dimensions_are_supported(self.width, self.height)
    }

    pub(crate) const fn dimensions_are_supported(width: u32, height: u32) -> bool {
        width >= MIN_AVATAR_DIMENSION
            && height >= MIN_AVATAR_DIMENSION
            && width <= MAX_AVATAR_DIMENSION
            && height <= MAX_AVATAR_DIMENSION
    }

    pub fn validate(self) -> Result<(), AvatarSpecError> {
        if self.is_supported() {
            Ok(())
        } else {
            Err(AvatarSpecError {
                width: self.width,
                height: self.height,
            })
        }
    }
}

impl Default for AvatarSpec {
    fn default() -> Self {
        Self::new_unchecked(256, 256, 1)
    }
}

/// Resource budget estimate for raster rendering.
///
/// This type intentionally models the raw RGBA buffer only. Encoders may need
/// additional temporary memory, so services should leave headroom above this
/// estimate when sizing request concurrency limits.
#[must_use = "use AvatarRenderResourceBudget to size service-level render concurrency limits"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarRenderResourceBudget {
    spec: AvatarSpec,
    concurrent_renders: usize,
}

impl AvatarRenderResourceBudget {
    pub const fn new(spec: AvatarSpec, concurrent_renders: usize) -> Self {
        Self {
            spec,
            concurrent_renders,
        }
    }

    pub const fn spec(self) -> AvatarSpec {
        self.spec
    }

    pub const fn concurrent_renders(self) -> usize {
        self.concurrent_renders
    }

    pub const fn raw_rgba_bytes_per_render(self) -> usize {
        self.spec.rgba_buffer_len()
    }

    pub const fn raw_rgba_bytes_for_concurrent_renders(self) -> usize {
        self.raw_rgba_bytes_per_render()
            .saturating_mul(self.concurrent_renders)
    }

    pub const fn max_supported_raw_rgba_bytes_for_concurrent_renders(
        concurrent_renders: usize,
    ) -> usize {
        MAX_AVATAR_RGBA_BYTES.saturating_mul(concurrent_renders)
    }

    pub const fn max_concurrent_renders_for_memory_budget(
        spec: AvatarSpec,
        memory_budget_bytes: usize,
    ) -> usize {
        let per_render = spec.rgba_buffer_len();
        match memory_budget_bytes.checked_div(per_render) {
            Some(value) => value,
            None => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpecError {
    width: u32,
    height: u32,
}

impl AvatarSpecError {
    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

impl std::fmt::Display for AvatarSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "avatar dimensions must be between {MIN_AVATAR_DIMENSION} and {MAX_AVATAR_DIMENSION} pixels per side, got {}x{}",
            self.width, self.height
        )
    }
}

impl std::error::Error for AvatarSpecError {}

pub(crate) fn validate_image_avatar_spec(spec: AvatarSpec) -> ImageResult<()> {
    spec.validate().map_err(avatar_spec_error_to_image_error)
}

pub(crate) fn avatar_spec_error_to_image_error(_: AvatarSpecError) -> ImageError {
    ImageError::Limits(LimitError::from_kind(LimitErrorKind::DimensionError))
}

pub(crate) fn avatar_identity_error_to_image_error(error: AvatarIdentityError) -> ImageError {
    ImageError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
}

pub(crate) fn avatar_render_error_to_image_error(error: AvatarRenderError) -> ImageError {
    match error {
        AvatarRenderError::Spec(error) => avatar_spec_error_to_image_error(error),
        AvatarRenderError::Identity(error) => avatar_identity_error_to_image_error(error),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvatarIdentityComponent {
    Input,
    Tenant,
    StyleVersion,
}

impl AvatarIdentityComponent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "identity input",
            Self::Tenant => "namespace tenant",
            Self::StyleVersion => "namespace style version",
        }
    }
}

impl std::fmt::Display for AvatarIdentityComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityError {
    pub(crate) component: AvatarIdentityComponent,
    pub(crate) length: usize,
    pub(crate) max: usize,
}

impl AvatarIdentityError {
    pub const fn component(self) -> AvatarIdentityComponent {
        self.component
    }

    pub const fn length(self) -> usize {
        self.length
    }

    pub const fn max(self) -> usize {
        self.max
    }
}

impl std::fmt::Display for AvatarIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} exceeds the maximum allowed size of {} bytes",
            self.component, self.max
        )
    }
}

impl std::error::Error for AvatarIdentityError {}

#[derive(Debug)]
pub enum AvatarRenderError {
    Spec(AvatarSpecError),
    Identity(AvatarIdentityError),
}

impl From<AvatarSpecError> for AvatarRenderError {
    fn from(error: AvatarSpecError) -> Self {
        Self::Spec(error)
    }
}

impl From<AvatarIdentityError> for AvatarRenderError {
    fn from(error: AvatarIdentityError) -> Self {
        Self::Identity(error)
    }
}

impl std::fmt::Display for AvatarRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spec(error) => error.fmt(f),
            Self::Identity(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for AvatarRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spec(error) => Some(error),
            Self::Identity(error) => Some(error),
        }
    }
}

/// Unified error type for high-level avatar APIs.
///
/// Lower-level functions keep their more specific error types for existing
/// callers. New convenience APIs such as [`AvatarBuilder`] return
/// `AvatarError` so `?` works across identity validation, dimension validation,
/// rendering, and encoding.
#[derive(Debug)]
pub enum AvatarError {
    Spec(AvatarSpecError),
    Identity(AvatarIdentityError),
    Render(AvatarRenderError),
    Image(ImageError),
}

impl From<AvatarSpecError> for AvatarError {
    fn from(error: AvatarSpecError) -> Self {
        Self::Spec(error)
    }
}

impl From<AvatarIdentityError> for AvatarError {
    fn from(error: AvatarIdentityError) -> Self {
        Self::Identity(error)
    }
}

impl From<AvatarRenderError> for AvatarError {
    fn from(error: AvatarRenderError) -> Self {
        Self::Render(error)
    }
}

impl From<ImageError> for AvatarError {
    fn from(error: ImageError) -> Self {
        Self::Image(error)
    }
}

impl std::fmt::Display for AvatarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spec(error) => error.fmt(f),
            Self::Identity(error) => error.fmt(f),
            Self::Render(error) => error.fmt(f),
            Self::Image(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for AvatarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spec(error) => Some(error),
            Self::Identity(error) => Some(error),
            Self::Render(error) => Some(error),
            Self::Image(error) => Some(error),
        }
    }
}

/// Options for deriving a stable avatar identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityOptions<'a> {
    namespace: AvatarNamespace<'a>,
}

impl<'a> AvatarIdentityOptions<'a> {
    pub const fn new(namespace: AvatarNamespace<'a>) -> Self {
        Self { namespace }
    }

    pub const fn namespace(self) -> AvatarNamespace<'a> {
        self.namespace
    }
}

impl Default for AvatarIdentityOptions<'static> {
    fn default() -> Self {
        Self::new(AvatarNamespace::DEFAULT)
    }
}

/// A stable avatar identity derived from a fixed 64-byte digest.
///
/// This is intended for Robohash-style uniqueness: the same input always maps
/// to the same visual genome, while different inputs produce different shape
/// and palette parameters with negligible collision risk.
///
/// # Security
///
/// `AvatarIdentity` implements `Clone`. Each clone is independently sanitized
/// on drop. Callers operating in high-assurance environments should keep clones
/// as short-lived as possible to reduce the window during which digest bytes
/// exist in multiple memory locations.
#[derive(Clone, Eq)]
pub struct AvatarIdentity {
    pub(crate) digest: [u8; 64],
}

impl AvatarIdentity {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_options(AvatarIdentityOptions::new(namespace), input)
    }

    pub fn new_with_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        let input = input.as_ref();
        validate_identity_component(
            AvatarIdentityComponent::Input,
            input.len(),
            MAX_AVATAR_ID_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::Tenant,
            options.namespace.tenant.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::StyleVersion,
            options.namespace.style_version.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        Ok(Self::new_unchecked(options, input))
    }

    pub(crate) fn new_unchecked(options: AvatarIdentityOptions<'_>, input: &[u8]) -> Self {
        Self {
            digest: derive_identity_digest(options, input),
        }
    }

    /// Returns an opaque, display-safe cache key for this identity.
    ///
    /// The returned string is a 64-character lowercase hexadecimal key derived
    /// by hashing the internal identity digest under a cache-key domain. It is
    /// stable for the same identity and active hash mode, but it does not expose
    /// the raw 64-byte identity digest.
    ///
    /// # Security
    ///
    /// Cache keys still enable correlation: the same identity produces the same
    /// cache key. They also do not prevent offline dictionary enumeration of
    /// low-entropy inputs such as email addresses or usernames. Treat cache keys
    /// as public identifiers for cache lookup, not as authentication secrets.
    /// Applications with sensitive, guessable identifiers should first map them
    /// through a keyed pseudonymization boundary with a separately managed
    /// tenant/domain key, then pass only the pseudonym to `hashavatar`.
    pub fn cache_key(&self) -> String {
        let expected_capacity = length_prefixed_component_size(CACHE_KEY_DOMAIN)
            + length_prefixed_component_size(&self.digest);
        let mut preimage = SecretVec::with_capacity(expected_capacity);
        update_hash_input_component(&mut preimage, CACHE_KEY_DOMAIN);
        update_hash_input_component(&mut preimage, &self.digest);
        debug_assert_eq!(
            (preimage.capacity(), preimage.len()),
            (expected_capacity, expected_capacity),
            "cache-key preimage size accounting drifted"
        );

        let digest = Secret::new(preimage.with_secret(sanitized_sha512_digest));
        digest.with_secret(|digest| hex_lower(&digest[..32]))
    }

    pub(crate) fn rng_seed(&self) -> Secret<[u8; 32]> {
        let mut seed = Secret::new([0u8; 32]);
        seed.with_secret_mut(|seed| seed.copy_from_slice(&self.digest[32..64]));
        seed
    }

    pub(crate) fn byte(&self, index: usize) -> u8 {
        debug_assert!(
            index < self.digest.len(),
            "identity digest byte index {index} out of range"
        );
        // Keep future renderer additions non-panicking if a digest offset is
        // miscomputed; tests still cover the currently used offset range.
        match self.digest.get(index) {
            Some(byte) => *byte,
            None => 0,
        }
    }

    pub(crate) fn unit_f32(&self, index: usize) -> f32 {
        self.byte(index) as f32 / 255.0
    }
}

impl SecureSanitize for AvatarIdentity {
    fn secure_sanitize(&mut self) {
        self.digest.secure_sanitize();
    }
}

impl std::fmt::Debug for AvatarIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AvatarIdentity")
            .field("digest", &"[REDACTED]")
            .finish()
    }
}

impl PartialEq for AvatarIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.digest.ct_eq(&other.digest).into()
    }
}

impl Drop for AvatarIdentity {
    fn drop(&mut self) {
        self.secure_sanitize();
    }
}

pub(crate) fn validate_identity_component(
    component: AvatarIdentityComponent,
    length: usize,
    max: usize,
) -> Result<(), AvatarIdentityError> {
    if length <= max {
        Ok(())
    } else {
        Err(AvatarIdentityError {
            component,
            length,
            max,
        })
    }
}

pub(crate) fn derive_identity_digest(options: AvatarIdentityOptions<'_>, input: &[u8]) -> [u8; 64] {
    let preimage = identity_hash_preimage(options, input);
    let digest = Secret::new(preimage.with_secret(active_identity_digest));
    digest.with_secret(|digest| *digest)
}

pub(crate) fn identity_hash_preimage(
    options: AvatarIdentityOptions<'_>,
    input: &[u8],
) -> SecretVec {
    let algorithm_overhead = if active_hash_algorithm_is_domain_separated() {
        length_prefixed_component_size(HASH_DOMAIN_ALGORITHM_COMPONENT)
            + length_prefixed_component_size(ACTIVE_HASH_ALGORITHM_LABEL)
    } else {
        0
    };
    let expected_capacity = length_prefixed_component_size(HASH_DOMAIN)
        + algorithm_overhead
        + length_prefixed_component_size(options.namespace.tenant.as_bytes())
        + length_prefixed_component_size(options.namespace.style_version.as_bytes())
        + length_prefixed_component_size(input);
    let mut preimage = SecretVec::with_capacity(expected_capacity);

    update_hash_input_component(&mut preimage, HASH_DOMAIN);
    if active_hash_algorithm_is_domain_separated() {
        update_hash_input_component(&mut preimage, HASH_DOMAIN_ALGORITHM_COMPONENT);
        update_hash_input_component(&mut preimage, ACTIVE_HASH_ALGORITHM_LABEL);
    }
    update_hash_input_component(&mut preimage, options.namespace.tenant.as_bytes());
    update_hash_input_component(&mut preimage, options.namespace.style_version.as_bytes());
    update_hash_input_component(&mut preimage, input);
    debug_assert_eq!(
        (preimage.capacity(), preimage.len()),
        (expected_capacity, expected_capacity),
        "identity preimage size accounting drifted"
    );
    preimage
}

const fn active_hash_algorithm_is_domain_separated() -> bool {
    cfg!(any(feature = "blake3", feature = "xxh3"))
}

#[cfg(feature = "blake3")]
pub(crate) fn active_identity_digest(preimage: &[u8]) -> [u8; 64] {
    blake3_digest(preimage)
}

#[cfg(all(not(feature = "blake3"), feature = "xxh3"))]
pub(crate) fn active_identity_digest(preimage: &[u8]) -> [u8; 64] {
    xxh3_128_digest(preimage)
}

#[cfg(all(not(feature = "blake3"), not(feature = "xxh3")))]
pub(crate) fn active_identity_digest(preimage: &[u8]) -> [u8; 64] {
    sha512_digest(preimage)
}

const fn length_prefixed_component_size(bytes: &[u8]) -> usize {
    std::mem::size_of::<u64>() + bytes.len()
}

pub(crate) fn update_hash_input_component(preimage: &mut SecretVec, bytes: &[u8]) {
    preimage.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    preimage.extend_from_slice(bytes);
}

pub(crate) fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
pub(crate) fn sha512_digest(preimage: &[u8]) -> [u8; 64] {
    sanitized_sha512_digest(preimage)
}

#[cfg(feature = "blake3")]
pub(crate) fn blake3_digest(preimage: &[u8]) -> [u8; 64] {
    let mut digest = Secret::new([0u8; 64]);
    digest.with_secret_mut(|digest| blake3_xof_fill(preimage, digest));
    digest.with_secret(|digest| *digest)
}

#[cfg(feature = "xxh3")]
pub(crate) fn xxh3_128_digest(preimage: &[u8]) -> [u8; 64] {
    let mut digest = Secret::new([0u8; 64]);
    for chunk in 0..4 {
        let expected_capacity = preimage.len()
            + length_prefixed_component_size(HASH_XOF_CHUNK_COMPONENT)
            + length_prefixed_component_size(&[chunk as u8]);
        let mut chunk_input = SecretVec::with_capacity(expected_capacity);
        chunk_input.extend_from_slice(preimage);
        update_hash_input_component(&mut chunk_input, HASH_XOF_CHUNK_COMPONENT);
        update_hash_input_component(&mut chunk_input, &[chunk as u8]);
        debug_assert_eq!(
            (chunk_input.capacity(), chunk_input.len()),
            (expected_capacity, expected_capacity),
            "XXH3 chunk preimage size accounting drifted"
        );
        let mut chunk_digest = chunk_input
            .with_secret(xxhash_rust::xxh3::xxh3_128)
            .to_le_bytes();
        let offset = chunk * chunk_digest.len();
        digest.with_secret_mut(|digest| {
            digest[offset..offset + chunk_digest.len()].copy_from_slice(&chunk_digest);
        });
        chunk_digest.secure_sanitize();
    }
    digest.with_secret(|digest| *digest)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarNamespace<'a> {
    tenant: &'a str,
    style_version: &'a str,
}

impl<'a> AvatarNamespace<'a> {
    pub const DEFAULT: Self = Self {
        tenant: "public",
        style_version: "v2",
    };

    pub fn new(tenant: &'a str, style_version: &'a str) -> Result<Self, AvatarIdentityError> {
        validate_identity_component(
            AvatarIdentityComponent::Tenant,
            tenant.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        validate_identity_component(
            AvatarIdentityComponent::StyleVersion,
            style_version.len(),
            MAX_AVATAR_NAMESPACE_COMPONENT_BYTES,
        )?;
        Ok(Self {
            tenant,
            style_version,
        })
    }

    pub const fn tenant(self) -> &'a str {
        self.tenant
    }

    pub const fn style_version(self) -> &'a str {
        self.style_version
    }
}

impl Default for AvatarNamespace<'_> {
    fn default() -> Self {
        Self::DEFAULT
    }
}
