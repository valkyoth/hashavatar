#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::vec::Vec;
use core::str::FromStr;

use sha2::{Digest, Sha512};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

pub const AVATAR_STYLE_VERSION: u32 = 2;
pub const MIN_AVATAR_DIMENSION: u32 = 64;
pub const MAX_AVATAR_DIMENSION: u32 = 2048;
pub const AVATAR_RGBA_BYTES_PER_PIXEL: usize = 4;
pub const MAX_AVATAR_PIXELS: usize =
    (MAX_AVATAR_DIMENSION as usize) * (MAX_AVATAR_DIMENSION as usize);
pub const MAX_AVATAR_RGBA_BYTES: usize = MAX_AVATAR_PIXELS * AVATAR_RGBA_BYTES_PER_PIXEL;
pub const MAX_AVATAR_ID_BYTES: usize = 1024;
pub const MAX_AVATAR_NAMESPACE_COMPONENT_BYTES: usize = 128;

const HASH_DOMAIN: &[u8] = b"hashavatar";
const HASH_DOMAIN_ALGORITHM_COMPONENT: &[u8] = b"identity-hash";
#[cfg(feature = "xxh3")]
const HASH_XOF_CHUNK_COMPONENT: &[u8] = b"xxh3-128-xof-chunk";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarSpec {
    width: u32,
    height: u32,
    seed: u64,
}

impl AvatarSpec {
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
        (self.width as usize) * (self.height as usize)
    }

    pub const fn rgba_buffer_len(self) -> usize {
        self.pixel_count() * AVATAR_RGBA_BYTES_PER_PIXEL
    }

    pub const fn is_supported(self) -> bool {
        Self::dimensions_are_supported(self.width, self.height)
    }

    const fn dimensions_are_supported(width: u32, height: u32) -> bool {
        width >= MIN_AVATAR_DIMENSION
            && height >= MIN_AVATAR_DIMENSION
            && width <= MAX_AVATAR_DIMENSION
            && height <= MAX_AVATAR_DIMENSION
    }
}

impl Default for AvatarSpec {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            seed: 1,
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

impl core::fmt::Display for AvatarSpecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "avatar dimensions must be between {MIN_AVATAR_DIMENSION} and {MAX_AVATAR_DIMENSION} pixels per side, got {}x{}",
            self.width, self.height
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AvatarSpecError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AvatarIdentityComponent {
    Input,
    Tenant,
    StyleVersion,
}

impl AvatarIdentityComponent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Tenant => "tenant",
            Self::StyleVersion => "style_version",
        }
    }
}

impl core::fmt::Display for AvatarIdentityComponent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityError {
    component: AvatarIdentityComponent,
    length: usize,
    max: usize,
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

impl core::fmt::Display for AvatarIdentityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "avatar identity {} length must be at most {} bytes, got {} bytes",
            self.component, self.max, self.length
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AvatarIdentityError {}

#[derive(Clone, Debug, Eq, PartialEq)]
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

impl core::fmt::Display for AvatarRenderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Spec(error) => error.fmt(f),
            Self::Identity(error) => error.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AvatarRenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spec(error) => Some(error),
            Self::Identity(error) => Some(error),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarHashAlgorithm {
    #[default]
    Sha512,
    #[cfg(feature = "blake3")]
    Blake3,
    #[cfg(feature = "xxh3")]
    Xxh3_128,
}

impl AvatarHashAlgorithm {
    #[cfg(all(feature = "blake3", feature = "xxh3"))]
    pub const ALL: &'static [Self] = &[Self::Sha512, Self::Blake3, Self::Xxh3_128];

    #[cfg(all(feature = "blake3", not(feature = "xxh3")))]
    pub const ALL: &'static [Self] = &[Self::Sha512, Self::Blake3];

    #[cfg(all(not(feature = "blake3"), feature = "xxh3"))]
    pub const ALL: &'static [Self] = &[Self::Sha512, Self::Xxh3_128];

    #[cfg(all(not(feature = "blake3"), not(feature = "xxh3")))]
    pub const ALL: &'static [Self] = &[Self::Sha512];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha512 => "sha512",
            #[cfg(feature = "blake3")]
            Self::Blake3 => "blake3",
            #[cfg(feature = "xxh3")]
            Self::Xxh3_128 => "xxh3-128",
        }
    }

    const fn domain_label(self) -> &'static [u8] {
        match self {
            Self::Sha512 => b"sha512",
            #[cfg(feature = "blake3")]
            Self::Blake3 => b"blake3",
            #[cfg(feature = "xxh3")]
            Self::Xxh3_128 => b"xxh3-128",
        }
    }
}

impl FromStr for AvatarHashAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match normalize_ascii(s).as_slice() {
            b"sha512" | b"sha-512" => Ok(Self::Sha512),
            #[cfg(feature = "blake3")]
            b"blake3" => Ok(Self::Blake3),
            #[cfg(feature = "xxh3")]
            b"xxh3" | b"xxh3-128" | b"xxh3_128" => Ok(Self::Xxh3_128),
            _ => Err("unsupported avatar hash algorithm"),
        }
    }
}

impl core::fmt::Display for AvatarHashAlgorithm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarIdentityOptions<'a> {
    namespace: AvatarNamespace<'a>,
    algorithm: AvatarHashAlgorithm,
}

impl<'a> AvatarIdentityOptions<'a> {
    pub const fn new(namespace: AvatarNamespace<'a>, algorithm: AvatarHashAlgorithm) -> Self {
        Self {
            namespace,
            algorithm,
        }
    }

    pub const fn namespace(self) -> AvatarNamespace<'a> {
        self.namespace
    }

    pub const fn algorithm(self) -> AvatarHashAlgorithm {
        self.algorithm
    }
}

impl Default for AvatarIdentityOptions<'static> {
    fn default() -> Self {
        Self::new(AvatarNamespace::DEFAULT, AvatarHashAlgorithm::Sha512)
    }
}

#[derive(Clone, Debug, Eq)]
pub struct AvatarIdentity {
    digest: [u8; 64],
}

impl AvatarIdentity {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_options(
            AvatarIdentityOptions::new(namespace, AvatarHashAlgorithm::Sha512),
            input,
        )
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
        Ok(Self {
            digest: derive_identity_digest(options, input),
        })
    }

    pub const fn from_digest(digest: [u8; 64]) -> Self {
        Self { digest }
    }

    pub const fn as_digest(&self) -> &[u8; 64] {
        &self.digest
    }

    pub fn rng_seed(&self) -> [u8; 32] {
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&self.digest[32..64]);
        seed
    }

    pub fn seed(&self) -> u64 {
        let mut seed = [0u8; 8];
        seed.copy_from_slice(&self.digest[..8]);
        u64::from_le_bytes(seed)
    }

    pub fn byte(&self, index: usize) -> u8 {
        self.digest[index]
    }

    pub fn unit_f32(&self, index: usize) -> f32 {
        self.byte(index) as f32 / 255.0
    }
}

impl Zeroize for AvatarIdentity {
    fn zeroize(&mut self) {
        self.digest.zeroize();
    }
}

impl PartialEq for AvatarIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.digest.ct_eq(&other.digest).into()
    }
}

impl Drop for AvatarIdentity {
    fn drop(&mut self) {
        self.zeroize();
    }
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

    pub const fn new_unchecked(tenant: &'a str, style_version: &'a str) -> Self {
        Self {
            tenant,
            style_version,
        }
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarOutputFormat {
    #[default]
    WebP,
    Png,
    Jpeg,
    Gif,
}

impl AvatarOutputFormat {
    pub const ALL: &'static [Self] = &[Self::WebP, Self::Png, Self::Jpeg, Self::Gif];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
        }
    }
}

impl FromStr for AvatarOutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match normalize_ascii(s).as_slice() {
            b"webp" => Ok(Self::WebP),
            b"png" => Ok(Self::Png),
            b"jpg" | b"jpeg" => Ok(Self::Jpeg),
            b"gif" => Ok(Self::Gif),
            _ => Err("unsupported avatar output format"),
        }
    }
}

impl core::fmt::Display for AvatarOutputFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarKind {
    #[default]
    Cat,
    Dog,
    Robot,
    Fox,
    Alien,
    Monster,
    Ghost,
    Slime,
    Bird,
    Wizard,
    Skull,
    Paws,
    Planet,
    Rocket,
    Mushroom,
    Cactus,
    Frog,
    Panda,
    Cupcake,
    Pizza,
    Icecream,
    Octopus,
    Knight,
}

impl AvatarKind {
    pub const ALL: &'static [Self] = &[
        Self::Cat,
        Self::Dog,
        Self::Robot,
        Self::Fox,
        Self::Alien,
        Self::Monster,
        Self::Ghost,
        Self::Slime,
        Self::Bird,
        Self::Wizard,
        Self::Skull,
        Self::Paws,
        Self::Planet,
        Self::Rocket,
        Self::Mushroom,
        Self::Cactus,
        Self::Frog,
        Self::Panda,
        Self::Cupcake,
        Self::Pizza,
        Self::Icecream,
        Self::Octopus,
        Self::Knight,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Robot => "robot",
            Self::Fox => "fox",
            Self::Alien => "alien",
            Self::Monster => "monster",
            Self::Ghost => "ghost",
            Self::Slime => "slime",
            Self::Bird => "bird",
            Self::Wizard => "wizard",
            Self::Skull => "skull",
            Self::Paws => "paws",
            Self::Planet => "planet",
            Self::Rocket => "rocket",
            Self::Mushroom => "mushroom",
            Self::Cactus => "cactus",
            Self::Frog => "frog",
            Self::Panda => "panda",
            Self::Cupcake => "cupcake",
            Self::Pizza => "pizza",
            Self::Icecream => "icecream",
            Self::Octopus => "octopus",
            Self::Knight => "knight",
        }
    }
}

impl FromStr for AvatarKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match normalize_ascii(s).as_slice() {
            b"cat" => Ok(Self::Cat),
            b"dog" => Ok(Self::Dog),
            b"robot" => Ok(Self::Robot),
            b"fox" => Ok(Self::Fox),
            b"alien" => Ok(Self::Alien),
            b"monster" => Ok(Self::Monster),
            b"ghost" => Ok(Self::Ghost),
            b"slime" => Ok(Self::Slime),
            b"bird" => Ok(Self::Bird),
            b"wizard" => Ok(Self::Wizard),
            b"skull" => Ok(Self::Skull),
            b"paws" => Ok(Self::Paws),
            b"planet" => Ok(Self::Planet),
            b"rocket" => Ok(Self::Rocket),
            b"mushroom" => Ok(Self::Mushroom),
            b"cactus" => Ok(Self::Cactus),
            b"frog" => Ok(Self::Frog),
            b"panda" => Ok(Self::Panda),
            b"cupcake" => Ok(Self::Cupcake),
            b"pizza" => Ok(Self::Pizza),
            b"icecream" | b"ice-cream" | b"ice_cream" => Ok(Self::Icecream),
            b"octopus" => Ok(Self::Octopus),
            b"knight" => Ok(Self::Knight),
            _ => Err("unsupported avatar kind"),
        }
    }
}

impl core::fmt::Display for AvatarKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AvatarBackground {
    #[default]
    Themed,
    White,
    Black,
    Dark,
    Light,
    Transparent,
}

impl AvatarBackground {
    pub const ALL: &'static [Self] = &[
        Self::Themed,
        Self::White,
        Self::Black,
        Self::Dark,
        Self::Light,
        Self::Transparent,
    ];

    pub fn from_byte(value: u8) -> Self {
        Self::ALL[usize::from(value) % Self::ALL.len()]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Themed => "themed",
            Self::White => "white",
            Self::Black => "black",
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Transparent => "transparent",
        }
    }
}

impl FromStr for AvatarBackground {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match normalize_ascii(s).as_slice() {
            b"themed" => Ok(Self::Themed),
            b"white" => Ok(Self::White),
            b"black" => Ok(Self::Black),
            b"dark" => Ok(Self::Dark),
            b"light" => Ok(Self::Light),
            b"transparent" => Ok(Self::Transparent),
            _ => Err("unsupported avatar background"),
        }
    }
}

impl core::fmt::Display for AvatarBackground {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AvatarOptions {
    pub kind: AvatarKind,
    pub background: AvatarBackground,
}

impl AvatarOptions {
    pub const fn new(kind: AvatarKind, background: AvatarBackground) -> Self {
        Self { kind, background }
    }
}

#[derive(Clone, Debug)]
pub struct AvatarRenderPlan {
    spec: AvatarSpec,
    identity: AvatarIdentity,
    options: AvatarOptions,
}

impl AvatarRenderPlan {
    pub fn new<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        identity_options: AvatarIdentityOptions<'_>,
        id: T,
        options: AvatarOptions,
    ) -> Result<Self, AvatarRenderError> {
        if !spec.is_supported() {
            return Err(AvatarSpecError {
                width: spec.width,
                height: spec.height,
            }
            .into());
        }
        let identity = AvatarIdentity::new_with_options(identity_options, id)?;
        Ok(Self {
            spec,
            identity,
            options,
        })
    }

    pub const fn spec(&self) -> AvatarSpec {
        self.spec
    }

    pub const fn identity(&self) -> &AvatarIdentity {
        &self.identity
    }

    pub const fn options(&self) -> AvatarOptions {
        self.options
    }
}

fn validate_identity_component(
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

fn derive_identity_digest(options: AvatarIdentityOptions<'_>, input: &[u8]) -> [u8; 64] {
    let mut preimage = identity_hash_preimage(options, input);
    let digest = match options.algorithm {
        AvatarHashAlgorithm::Sha512 => sha512_digest(&preimage),
        #[cfg(feature = "blake3")]
        AvatarHashAlgorithm::Blake3 => blake3_digest(&preimage),
        #[cfg(feature = "xxh3")]
        AvatarHashAlgorithm::Xxh3_128 => xxh3_128_digest(&preimage),
    };
    preimage.zeroize();
    digest
}

fn identity_hash_preimage(options: AvatarIdentityOptions<'_>, input: &[u8]) -> Vec<u8> {
    let algorithm_overhead = if options.algorithm == AvatarHashAlgorithm::Sha512 {
        0
    } else {
        length_prefixed_component_size(HASH_DOMAIN_ALGORITHM_COMPONENT)
            + length_prefixed_component_size(options.algorithm.domain_label())
    };
    let mut preimage = Vec::with_capacity(
        length_prefixed_component_size(HASH_DOMAIN)
            + algorithm_overhead
            + length_prefixed_component_size(options.namespace.tenant.as_bytes())
            + length_prefixed_component_size(options.namespace.style_version.as_bytes())
            + length_prefixed_component_size(input),
    );

    update_hash_input_component(&mut preimage, HASH_DOMAIN);
    if options.algorithm != AvatarHashAlgorithm::Sha512 {
        update_hash_input_component(&mut preimage, HASH_DOMAIN_ALGORITHM_COMPONENT);
        update_hash_input_component(&mut preimage, options.algorithm.domain_label());
    }
    update_hash_input_component(&mut preimage, options.namespace.tenant.as_bytes());
    update_hash_input_component(&mut preimage, options.namespace.style_version.as_bytes());
    update_hash_input_component(&mut preimage, input);
    preimage
}

const fn length_prefixed_component_size(bytes: &[u8]) -> usize {
    core::mem::size_of::<u64>() + bytes.len()
}

fn update_hash_input_component(preimage: &mut Vec<u8>, bytes: &[u8]) {
    preimage.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    preimage.extend_from_slice(bytes);
}

fn sha512_digest(preimage: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(preimage);
    hasher.finalize().into()
}

#[cfg(feature = "blake3")]
fn blake3_digest(preimage: &[u8]) -> [u8; 64] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(preimage);
    let mut digest = [0u8; 64];
    hasher.finalize_xof().fill(&mut digest);
    digest
}

#[cfg(feature = "xxh3")]
fn xxh3_128_digest(preimage: &[u8]) -> [u8; 64] {
    let mut digest = [0u8; 64];
    for chunk in 0..4 {
        let mut chunk_input = Vec::with_capacity(
            preimage.len()
                + length_prefixed_component_size(HASH_XOF_CHUNK_COMPONENT)
                + length_prefixed_component_size(&[chunk as u8]),
        );
        chunk_input.extend_from_slice(preimage);
        update_hash_input_component(&mut chunk_input, HASH_XOF_CHUNK_COMPONENT);
        update_hash_input_component(&mut chunk_input, &[chunk as u8]);
        let mut chunk_digest = xxhash_rust::xxh3::xxh3_128(&chunk_input).to_le_bytes();
        let offset = chunk * chunk_digest.len();
        digest[offset..offset + chunk_digest.len()].copy_from_slice(&chunk_digest);
        chunk_digest.zeroize();
        chunk_input.zeroize();
    }
    digest
}

fn normalize_ascii(input: &str) -> Vec<u8> {
    input
        .trim()
        .bytes()
        .map(|byte| byte.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_builds_identity_and_render_plan_without_std_types() {
        let spec = AvatarSpec::new(128, 128, 0).expect("spec should be valid");
        let options = AvatarIdentityOptions::new(
            AvatarNamespace::new("tenant", "v2").expect("namespace should be valid"),
            AvatarHashAlgorithm::Sha512,
        );
        let plan = AvatarRenderPlan::new(
            spec,
            options,
            b"alice@example.com",
            AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
        )
        .expect("plan should be valid");

        assert_eq!(plan.spec().rgba_buffer_len(), 128 * 128 * 4);
        assert_eq!(plan.options().kind, AvatarKind::Robot);
        assert_ne!(plan.identity().as_digest(), &[0u8; 64]);
    }

    #[test]
    fn core_parser_round_trips_public_enums() {
        for &kind in AvatarKind::ALL {
            assert_eq!(kind.as_str().parse::<AvatarKind>().ok(), Some(kind));
        }
        for &background in AvatarBackground::ALL {
            assert_eq!(
                background.as_str().parse::<AvatarBackground>().ok(),
                Some(background)
            );
        }
        for &format in AvatarOutputFormat::ALL {
            assert_eq!(
                format.as_str().parse::<AvatarOutputFormat>().ok(),
                Some(format)
            );
        }
        for &algorithm in AvatarHashAlgorithm::ALL {
            assert_eq!(
                algorithm.as_str().parse::<AvatarHashAlgorithm>().ok(),
                Some(algorithm)
            );
        }
    }
}
