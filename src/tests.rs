use super::*;
use image::ImageFormat;
use sha2::Sha512 as TestSha512;

fn valid_spec(width: u32, height: u32, seed: u64) -> AvatarSpec {
    AvatarSpec::new(width, height, seed).expect("test avatar spec should be valid")
}

fn valid_namespace<'a>(tenant: &'a str, style_version: &'a str) -> AvatarNamespace<'a> {
    super::AvatarNamespace::new(tenant, style_version).expect("test namespace should be valid")
}

fn valid_identity<T: AsRef<[u8]>>(input: T) -> AvatarIdentity {
    super::AvatarIdentity::new(input).expect("test identity should be valid")
}

fn valid_identity_with_namespace<T: AsRef<[u8]>>(
    namespace: AvatarNamespace<'_>,
    input: T,
) -> AvatarIdentity {
    AvatarIdentity::new_with_namespace(namespace, input).expect("test identity should be valid")
}

fn render_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> RgbaImage {
    super::render_avatar_for_id(spec, id, options).expect("valid avatar spec should render")
}

fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> String {
    super::render_avatar_svg_for_id(spec, id, options)
        .expect("valid avatar spec should render as svg")
}

fn render_avatar_style_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    style: AvatarStyleOptions,
) -> RgbaImage {
    super::render_avatar_style_for_id(spec, id, style).expect("valid avatar style should render")
}

fn render_avatar_svg_style_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    style: AvatarStyleOptions,
) -> String {
    super::render_avatar_svg_style_for_id(spec, id, style)
        .expect("valid avatar style should render as svg")
}

fn assert_svg_is_well_formed(svg: &str) {
    let document = roxmltree::Document::parse(svg).expect("svg should be well-formed xml");
    let root = document.root_element();

    assert_eq!(root.tag_name().name(), "svg");
    assert_eq!(
        root.tag_name().namespace(),
        Some("http://www.w3.org/2000/svg")
    );
    assert!(root.attribute("viewBox").is_some());
}

fn identity_with_digest_byte(index: usize, value: u8) -> AvatarIdentity {
    let mut digest = [0_u8; 64];
    digest[index] = value;
    AvatarIdentity { digest }
}

fn render_cat_avatar(spec: AvatarSpec) -> RgbaImage {
    super::render_cat_avatar(spec).expect("valid avatar spec should render")
}

fn render_cat_avatar_for_identity(spec: AvatarSpec, identity: &AvatarIdentity) -> RgbaImage {
    super::render_cat_avatar_for_identity(spec, identity).expect("valid avatar spec should render")
}

fn render_cat_avatar_for_identity_with_background(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_cat_avatar_for_identity_with_background(spec, identity, background)
        .expect("valid avatar spec should render")
}

fn render_dog_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_dog_avatar_for_identity(spec, identity, background)
        .expect("valid avatar spec should render")
}

fn render_robot_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_robot_avatar_for_identity(spec, identity, background)
        .expect("valid avatar spec should render")
}

fn render_alien_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_alien_avatar_for_identity(spec, identity, background)
        .expect("valid avatar spec should render")
}

fn render_monster_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_monster_avatar_for_identity(spec, identity, background)
        .expect("valid avatar spec should render")
}

fn render_paws_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    super::render_paws_avatar_for_identity(spec, identity, background)
        .expect("valid avatar spec should render")
}

#[test]
fn cat_avatar_is_deterministic_for_a_seed() {
    let spec = valid_spec(256, 256, 42);
    let left = render_cat_avatar(spec);
    let right = render_cat_avatar(spec);

    assert_eq!(left.as_raw(), right.as_raw());
}

#[test]
fn cat_avatar_uses_requested_dimensions() {
    let image = render_cat_avatar(valid_spec(192, 160, 7));

    assert_eq!(image.width(), 192);
    assert_eq!(image.height(), 160);
}

#[test]
fn cat_avatar_has_non_background_pixels() {
    let spec = valid_spec(128, 128, 3);
    let image = render_cat_avatar(spec);
    let background = image.get_pixel(0, 0);

    assert!(image.pixels().any(|pixel| pixel != background));
}

#[test]
fn avatar_identity_uses_sha512_digest() {
    let identity = valid_identity("alice@example.com");

    assert_eq!(identity.digest.len(), 64);
    let rng_seed = identity.rng_seed();
    rng_seed.with_secret(|rng_seed| assert_eq!(&rng_seed[..], &identity.digest[32..64]));
}

#[test]
fn avatar_identity_debug_redacts_digest() {
    let identity = valid_identity("alice@example.com");
    let debug = format!("{identity:?}");

    assert_eq!(debug, r#"AvatarIdentity { digest: "[REDACTED]" }"#);
    for byte in identity.digest {
        assert!(
            !debug.contains(&format!("{byte}")),
            "debug output leaked digest byte {byte}"
        );
    }
}

#[test]
fn avatar_identity_rustdoc_mentions_clone_sanitization() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));
    let before_struct = source
        .split("pub struct AvatarIdentity {")
        .next()
        .expect("AvatarIdentity struct should exist");
    let docs = before_struct
        .rsplit("/// A stable avatar identity")
        .next()
        .expect("AvatarIdentity rustdoc should exist");

    assert!(docs.contains("/// # Security"));
    assert!(docs.contains("`AvatarIdentity` implements `Clone`"));
    assert!(docs.contains("Each clone is independently sanitized"));
    assert!(docs.contains("short-lived as possible"));
    assert!(docs.contains("multiple memory locations"));
}

#[test]
#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn sha512_hasher_state_sanitization_boundary_is_documented() {
    let controls = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/SECURITY_CONTROLS.md"
    ));

    assert!(controls.contains("sanitization-crypto-interop"));
    assert!(controls.contains("upstream `sha2`"));
}

#[test]
fn rng_seed_uses_second_half_of_identity_digest() {
    let identity = valid_identity("alice@example.com");
    let rng_seed = identity.rng_seed();

    rng_seed.with_secret(|rng_seed| {
        assert_eq!(rng_seed.len(), 32);
        assert_eq!(&rng_seed[..], &identity.digest[32..64]);
        assert_ne!(&identity.digest[..32], &rng_seed[..]);
    });
}

#[test]
fn rng_seed_copy_is_sanitizing() {
    let identity = valid_identity("alice@example.com");
    let rng_seed: Secret<[u8; 32]> = identity.rng_seed();

    rng_seed.with_secret(|rng_seed| assert_eq!(rng_seed.len(), 32));
}

#[test]
fn identity_byte_access_debug_asserts_for_out_of_range_indices() {
    let identity = valid_identity("alice@example.com");

    if cfg!(debug_assertions) {
        assert!(std::panic::catch_unwind(|| identity.byte(64)).is_err());
    } else {
        assert_eq!(identity.byte(64), 0);
        assert_eq!(identity.unit_f32(64), 0.0);
    }
}

#[test]
fn renderer_rng_seed_copy_is_sanitized_before_rng_use() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/avatars/common.rs"
    ));
    let helper = source
        .split("fn seeded_renderer_rng")
        .nth(1)
        .expect("seeded renderer rng helper should exist");

    assert!(helper.contains("let rng_seed_value = Secret::new(rng_seed.with_secret"));
    assert!(helper.contains("drop(rng_seed);"));
    assert!(helper.contains("StdRng::from_seed(*rng_seed_value)"));
    assert!(!helper.contains("let mut rng_seed_value = *rng_seed;"));
}

#[test]
fn identity_digest_intermediate_uses_sanitizing_guard() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));
    let helper = source
        .split("fn derive_identity_digest")
        .nth(1)
        .and_then(|after_name| after_name.split("fn identity_hash_preimage").next())
        .expect("identity digest helper should exist");

    assert!(helper.contains("Secret::new(active_identity_digest(&preimage))"));
    assert!(helper.contains("volatile_sanitize_vec(&mut preimage);"));
}

#[test]
fn preimage_builders_assert_exact_capacity_before_sanitization() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));

    let identity_helper = source
        .split("fn identity_hash_preimage")
        .nth(1)
        .and_then(|after_name| after_name.split("const fn active_hash_algorithm").next())
        .expect("identity preimage helper should exist");
    assert!(identity_helper.contains("let expected_capacity"));
    assert!(identity_helper.contains("assert_eq!("));
    assert!(identity_helper.contains("preimage.capacity()"));
    assert!(identity_helper.contains("preimage.len()"));

    let cache_key_helper = source
        .split("pub fn cache_key")
        .nth(1)
        .and_then(|after_name| after_name.split("fn rng_seed").next())
        .expect("cache-key helper should exist");
    assert!(cache_key_helper.contains("let expected_capacity"));
    assert!(cache_key_helper.contains("assert_eq!("));
    assert!(cache_key_helper.contains("preimage.capacity()"));
    assert!(cache_key_helper.contains("preimage.len()"));

    #[cfg(feature = "xxh3")]
    {
        let xxh3_helper = source
            .split("fn xxh3_128_digest")
            .nth(1)
            .and_then(|after_name| after_name.split("#[derive(Clone, Copy").next())
            .expect("XXH3 digest helper should exist");
        assert!(xxh3_helper.contains("let expected_capacity"));
        assert!(xxh3_helper.contains("assert_eq!("));
        assert!(xxh3_helper.contains("chunk_input.capacity()"));
        assert!(xxh3_helper.contains("chunk_input.len()"));
    }
}

#[test]
#[cfg(feature = "blake3")]
fn blake3_digest_uses_crypto_interop_and_secret_buffer() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));
    let helper = source
        .split("fn blake3_digest")
        .nth(1)
        .and_then(|after_name| after_name.split("#[cfg(feature = \"xxh3\")]").next())
        .expect("BLAKE3 digest helper should exist");

    assert!(helper.contains("Secret::new([0u8; 64])"));
    assert!(helper.contains("blake3_xof_fill(preimage, digest)"));
    assert!(!helper.contains("let mut digest = [0u8; 64];"));
}

#[test]
#[cfg(feature = "xxh3")]
fn xxh3_digest_accumulator_uses_sanitizing_guard() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));
    let helper = source
        .split("fn xxh3_128_digest")
        .nth(1)
        .and_then(|after_name| after_name.split("#[derive(Clone, Copy").next())
        .expect("XXH3 digest helper should exist");

    assert!(helper.contains("let mut digest = Secret::new([0u8; 64]);"));
    assert!(helper.contains("digest.with_secret_mut"));
    assert!(!helper.contains("let mut digest = [0u8; 64];"));
}

#[test]
fn avatar_identity_equality_compares_digest_values() {
    let left = valid_identity("alice@example.com");
    let same = valid_identity("alice@example.com");
    let different = valid_identity("bob@example.com");

    assert_eq!(left, same);
    assert_ne!(left, different);
}

#[test]
fn avatar_identity_cache_key_is_stable_and_not_raw_digest_hex() {
    let identity = valid_identity("alice@example.com");
    let same = valid_identity("alice@example.com");
    let different = valid_identity("bob@example.com");

    let key = identity.cache_key();
    assert_eq!(key.len(), 64);
    assert!(key.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(key, same.cache_key());
    assert_ne!(key, different.cache_key());
    assert_ne!(key, hex_lower(&identity.digest[..32]));
}

#[test]
fn default_identity_options_match_namespace_constructor() {
    let namespace = valid_namespace("tenant-a", "v2");
    let default = AvatarIdentity::new_with_namespace(namespace, "alice@example.com")
        .expect("identity should be valid");
    let explicit = AvatarIdentity::new_with_options(
        AvatarIdentityOptions::new(namespace),
        "alice@example.com",
    )
    .expect("explicit identity options should be valid");

    assert_eq!(default.digest, explicit.digest);
}

#[test]
fn active_hash_algorithm_label_matches_enabled_feature() {
    #[cfg(feature = "blake3")]
    assert_eq!(ACTIVE_HASH_ALGORITHM_LABEL, b"blake3");

    #[cfg(feature = "xxh3")]
    assert_eq!(ACTIVE_HASH_ALGORITHM_LABEL, b"xxh3-128");

    #[cfg(not(any(feature = "blake3", feature = "xxh3")))]
    assert_eq!(ACTIVE_HASH_ALGORITHM_LABEL, b"sha512");
}

#[test]
#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn default_sha512_preimage_omits_algorithm_domain_for_legacy_stability() {
    let preimage = identity_hash_preimage(AvatarIdentityOptions::default(), b"alice@example.com");

    assert!(
        !preimage
            .windows(HASH_DOMAIN_ALGORITHM_COMPONENT.len())
            .any(|window| window == HASH_DOMAIN_ALGORITHM_COMPONENT)
    );
    assert!(
        !preimage
            .windows(ACTIVE_HASH_ALGORITHM_LABEL.len())
            .any(|window| window == ACTIVE_HASH_ALGORITHM_LABEL)
    );
}

#[test]
#[cfg(any(feature = "blake3", feature = "xxh3"))]
fn optional_hash_modes_add_algorithm_domain_to_preimage() {
    let preimage = identity_hash_preimage(AvatarIdentityOptions::default(), b"alice@example.com");

    assert!(
        preimage
            .windows(HASH_DOMAIN_ALGORITHM_COMPONENT.len())
            .any(|window| window == HASH_DOMAIN_ALGORITHM_COMPONENT)
    );
    assert!(
        preimage
            .windows(ACTIVE_HASH_ALGORITHM_LABEL.len())
            .any(|window| window == ACTIVE_HASH_ALGORITHM_LABEL)
    );
}

#[test]
fn oversized_identity_is_rejected_for_active_hash_mode() {
    let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
    let error = AvatarIdentity::new_with_options(AvatarIdentityOptions::default(), &too_long)
        .expect_err("oversized identity should fail");

    assert_eq!(error.component(), AvatarIdentityComponent::Input);
    assert_eq!(error.length(), MAX_AVATAR_ID_BYTES + 1);
    assert_eq!(error.max(), MAX_AVATAR_ID_BYTES);
}

#[cfg(feature = "blake3")]
#[test]
fn blake3_identity_mode_renders_avatar() {
    let image = render_avatar_with_identity_options(
        valid_spec(96, 96, 0),
        AvatarIdentityOptions::new(AvatarNamespace::default()),
        "alice@example.com",
        AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
    )
    .expect("blake3-backed avatar should render");

    assert_eq!(image.width(), 96);
    assert_eq!(image.height(), 96);
}

#[cfg(feature = "xxh3")]
#[test]
fn xxh3_identity_mode_renders_avatar() {
    let image = render_avatar_with_identity_options(
        valid_spec(96, 96, 0),
        AvatarIdentityOptions::new(AvatarNamespace::default()),
        "alice@example.com",
        AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed),
    )
    .expect("xxh3-backed avatar should render");

    assert_eq!(image.width(), 96);
    assert_eq!(image.height(), 96);
}

#[test]
fn namespace_changes_identity_digest() {
    let left =
        valid_identity_with_namespace(valid_namespace("tenant-a", "v2"), "alice@example.com");
    let right =
        valid_identity_with_namespace(valid_namespace("tenant-b", "v2"), "alice@example.com");

    assert_ne!(left.digest, right.digest);
}

#[test]
fn namespace_hashing_is_not_ambiguous_with_nul_bytes() {
    let left =
        valid_identity_with_namespace(valid_namespace("tenant\0v2", "v1"), "alice@example.com");
    let right =
        valid_identity_with_namespace(valid_namespace("tenant", "v2\0v1"), "alice@example.com");

    assert_ne!(left.digest, right.digest);
}

#[test]
fn identity_construction_rejects_oversized_input() {
    let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
    let error = AvatarIdentity::new(&too_long).expect_err("oversized identity should fail");

    assert_eq!(error.component(), AvatarIdentityComponent::Input);
    assert_eq!(error.length(), MAX_AVATAR_ID_BYTES + 1);
    assert_eq!(error.max(), MAX_AVATAR_ID_BYTES);
}

#[test]
fn avatar_identity_error_display_omits_exact_rejected_length() {
    let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
    let error = AvatarIdentity::new(&too_long).expect_err("oversized identity should fail");
    let message = error.to_string();

    assert!(message.contains("identity input"));
    assert!(message.contains(&MAX_AVATAR_ID_BYTES.to_string()));
    assert!(!message.contains(&(MAX_AVATAR_ID_BYTES + 1).to_string()));
}

#[test]
fn namespace_construction_rejects_oversized_components() {
    let too_long = "a".repeat(MAX_AVATAR_NAMESPACE_COMPONENT_BYTES + 1);
    let error = AvatarNamespace::new(&too_long, "v2").expect_err("oversized tenant should fail");

    assert_eq!(error.component(), AvatarIdentityComponent::Tenant);
    assert_eq!(error.length(), MAX_AVATAR_NAMESPACE_COMPONENT_BYTES + 1);
    assert_eq!(error.max(), MAX_AVATAR_NAMESPACE_COMPONENT_BYTES);
}

#[test]
fn render_avatar_for_id_rejects_oversized_identity() {
    let too_long = vec![b'a'; MAX_AVATAR_ID_BYTES + 1];
    let error = super::render_avatar_for_id(
        valid_spec(128, 128, 0),
        &too_long,
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
    )
    .expect_err("oversized identity should fail");

    assert!(matches!(
        error,
        AvatarRenderError::Identity(AvatarIdentityError {
            component: AvatarIdentityComponent::Input,
            ..
        })
    ));
}

#[test]
fn hashed_cat_avatar_is_deterministic_for_same_id() {
    let spec = valid_spec(192, 192, 0);
    let left = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));
    let right = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));

    assert_eq!(left.as_raw(), right.as_raw());
}

#[test]
fn hashed_cat_avatar_changes_for_different_ids() {
    let spec = valid_spec(192, 192, 0);
    let left = render_cat_avatar_for_identity(spec, &valid_identity("alice@example.com"));
    let right = render_cat_avatar_for_identity(spec, &valid_identity("bob@example.com"));

    assert_ne!(left.as_raw(), right.as_raw());
}

#[test]
fn cat_avatar_webp_export_round_trips() {
    let bytes = encode_cat_avatar(valid_spec(128, 128, 11), AvatarOutputFormat::WebP)
        .expect("webp encoding should succeed");
    let decoded =
        image::load_from_memory_with_format(&bytes, ImageFormat::WebP).expect("webp should decode");

    assert_eq!(decoded.width(), 128);
    assert_eq!(decoded.height(), 128);
}

#[test]
#[cfg(feature = "png")]
fn cat_avatar_png_export_round_trips() {
    let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Png)
        .expect("png encoding should succeed");
    let decoded =
        image::load_from_memory_with_format(&bytes, ImageFormat::Png).expect("png should decode");

    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 96);
}

#[test]
#[cfg(feature = "jpeg")]
fn cat_avatar_jpeg_export_round_trips() {
    let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Jpeg)
        .expect("jpeg encoding should succeed");
    let decoded =
        image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg).expect("jpeg should decode");

    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 96);
}

#[test]
#[cfg(not(feature = "png"))]
fn png_output_format_is_unavailable_without_feature() {
    assert_eq!(
        "png".parse::<AvatarOutputFormat>(),
        Err("unsupported avatar output format")
    );
    assert!(
        !AvatarOutputFormat::ALL
            .iter()
            .any(|format| format.as_str() == "png")
    );
}

#[test]
#[cfg(not(feature = "jpeg"))]
fn jpeg_output_format_is_unavailable_without_feature() {
    assert_eq!(
        "jpg".parse::<AvatarOutputFormat>(),
        Err("unsupported avatar output format")
    );
    assert_eq!(
        "jpeg".parse::<AvatarOutputFormat>(),
        Err("unsupported avatar output format")
    );
    assert!(
        !AvatarOutputFormat::ALL
            .iter()
            .any(|format| format.as_str() == "jpg")
    );
}

#[test]
#[cfg(feature = "gif")]
fn cat_avatar_gif_export_round_trips() {
    let bytes = encode_cat_avatar(valid_spec(96, 96, 99), AvatarOutputFormat::Gif)
        .expect("gif encoding should succeed");
    let decoded =
        image::load_from_memory_with_format(&bytes, ImageFormat::Gif).expect("gif should decode");

    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 96);
}

#[test]
#[cfg(not(feature = "gif"))]
fn gif_output_format_is_unavailable_without_feature() {
    assert_eq!(
        "gif".parse::<AvatarOutputFormat>(),
        Err("unsupported avatar output format")
    );
    assert!(
        !AvatarOutputFormat::ALL
            .iter()
            .any(|format| format.as_str() == "gif")
    );
}

#[test]
#[cfg(feature = "jpeg")]
fn jpeg_export_flattens_transparency_over_white() {
    let bytes = encode_avatar_for_id(
        valid_spec(96, 96, 0),
        "cat@hashavatar.app",
        AvatarOutputFormat::Jpeg,
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent),
    )
    .expect("jpeg encoding should succeed");
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::Jpeg)
        .expect("jpeg should decode")
        .to_rgb8();

    let corner = decoded.get_pixel(0, 0);
    assert!(corner.0.iter().all(|channel| *channel > 245));
}

#[test]
fn webp_is_the_default_output_format() {
    assert_eq!(AvatarOutputFormat::default(), AvatarOutputFormat::WebP);
}

#[test]
fn hashed_cat_avatar_webp_export_round_trips() {
    let bytes = encode_cat_avatar_for_id(
        valid_spec(128, 128, 0),
        "alice@example.com",
        AvatarOutputFormat::WebP,
    )
    .expect("webp encoding should succeed");
    let decoded =
        image::load_from_memory_with_format(&bytes, ImageFormat::WebP).expect("webp should decode");

    assert_eq!(decoded.width(), 128);
    assert_eq!(decoded.height(), 128);
}

#[test]
fn white_background_mode_renders_white_corner() {
    let image = render_cat_avatar_for_identity_with_background(
        valid_spec(128, 128, 0),
        &valid_identity("alice@example.com"),
        AvatarBackground::White,
    );

    assert_eq!(image.get_pixel(0, 0), &Rgba([255, 255, 255, 255]));
}

#[test]
fn fixed_background_modes_render_expected_corners() {
    for (background, expected) in [
        (AvatarBackground::Black, Rgba([0, 0, 0, 255])),
        (AvatarBackground::Dark, Rgba([17, 24, 39, 255])),
        (AvatarBackground::Light, Rgba([248, 250, 247, 255])),
    ] {
        let image = render_cat_avatar_for_identity_with_background(
            valid_spec(128, 128, 0),
            &valid_identity("cat@hashavatar.app"),
            background,
        );

        assert_eq!(image.get_pixel(0, 0), &expected, "{background}");
    }
}

#[test]
fn transparent_background_mode_renders_clear_corner() {
    let image = render_cat_avatar_for_identity_with_background(
        valid_spec(128, 128, 0),
        &valid_identity("cat@hashavatar.app"),
        AvatarBackground::Transparent,
    );

    assert_eq!(image.get_pixel(0, 0), &Rgba([255, 255, 255, 0]));
}

#[test]
fn decorative_background_modes_render_distinct_raster_canvases() {
    let spec = valid_spec(128, 128, 0);
    let identity = valid_identity("backgrounds@hashavatar.app");
    let mut fingerprints = Vec::new();

    for background in [
        AvatarBackground::PolkaDot,
        AvatarBackground::Striped,
        AvatarBackground::Checkerboard,
        AvatarBackground::Grid,
        AvatarBackground::Sunrise,
        AvatarBackground::Ocean,
        AvatarBackground::Starry,
    ] {
        let image = render_cat_avatar_for_identity_with_background(spec, &identity, background);
        assert_eq!(image.width(), 128);
        assert_eq!(image.height(), 128);
        assert!(
            image.pixels().any(|pixel| pixel.0[3] == 255),
            "{background}"
        );
        fingerprints.push(image_fingerprint(&image));
    }

    fingerprints.sort();
    fingerprints.dedup();
    assert_eq!(fingerprints.len(), 7);
}

#[test]
fn decorative_svg_backgrounds_use_structured_defs() {
    let spec = valid_spec(128, 128, 0);
    for background in [
        AvatarBackground::PolkaDot,
        AvatarBackground::Striped,
        AvatarBackground::Checkerboard,
        AvatarBackground::Grid,
        AvatarBackground::Sunrise,
        AvatarBackground::Ocean,
        AvatarBackground::Starry,
    ] {
        let svg = render_avatar_svg_for_id(
            spec,
            "backgrounds@hashavatar.app",
            AvatarOptions::new(AvatarKind::Robot, background),
        );

        assert_svg_is_well_formed(&svg);
        assert!(svg.contains("hashavatar-bg-"), "{background}");
        assert!(!svg.contains("<script"), "{background}");
    }
}

#[test]
fn dog_and_robot_variants_generate_distinct_images() {
    let spec = valid_spec(128, 128, 0);
    let id = valid_identity("alice@example.com");
    let dog = render_dog_avatar_for_identity(spec, &id, AvatarBackground::Themed);
    let robot = render_robot_avatar_for_identity(spec, &id, AvatarBackground::Themed);

    assert_ne!(dog.as_raw(), robot.as_raw());
}

#[test]
fn monster_variant_is_distinct_from_alien() {
    let spec = valid_spec(128, 128, 0);
    let id = valid_identity("alice@example.com");
    let alien = render_alien_avatar_for_identity(spec, &id, AvatarBackground::Themed);
    let monster = render_monster_avatar_for_identity(spec, &id, AvatarBackground::Themed);

    assert_ne!(alien.as_raw(), monster.as_raw());
}

#[test]
fn paws_variant_is_distinct_from_cat() {
    let spec = valid_spec(128, 128, 0);
    let id = valid_identity("alice@example.com");
    let cat = render_cat_avatar_for_identity_with_background(spec, &id, AvatarBackground::Themed);
    let paws = render_paws_avatar_for_identity(spec, &id, AvatarBackground::Themed);

    assert_ne!(cat.as_raw(), paws.as_raw());
}

#[test]
fn generic_avatar_encoder_supports_robot_and_white_background() {
    let bytes = encode_avatar_for_id(
        valid_spec(96, 96, 0),
        "robot@example.com",
        AvatarOutputFormat::WebP,
        AvatarOptions {
            kind: AvatarKind::Robot,
            background: AvatarBackground::White,
        },
    )
    .expect("robot webp encoding should succeed");
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
        .expect("robot webp should decode");

    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 96);
}

#[test]
fn avatar_builder_renders_svg_and_encoded_webp() {
    let svg = AvatarBuilder::for_id("builder@example.com")
        .size(96, 96)
        .namespace("tenant-a", "v2")
        .kind(AvatarKind::Robot)
        .background(AvatarBackground::Transparent)
        .accessory(AvatarAccessory::Glasses)
        .shape(AvatarShape::Circle)
        .render_svg()
        .expect("builder svg should render");
    assert!(svg.contains("robot avatar"));

    let bytes = AvatarBuilder::for_id("builder@example.com")
        .size(96, 96)
        .kind(AvatarKind::Robot)
        .encode(AvatarOutputFormat::WebP)
        .expect("builder webp should encode");
    let decoded = image::load_from_memory_with_format(&bytes, ImageFormat::WebP)
        .expect("builder webp should decode");
    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 96);
}

#[test]
fn avatar_builder_returns_unified_errors_without_panicking() {
    let error = AvatarBuilder::for_id("builder@example.com")
        .size(1, 256)
        .render_svg()
        .expect_err("invalid size should be rejected");

    assert!(matches!(error, AvatarError::Spec(_)));
}

#[test]
fn avatar_builder_debug_redacts_identity_input() {
    let debug = format!(
        "{:?}",
        AvatarBuilder::for_id("secret@example.com").namespace("private-tenant", "secret-v3")
    );

    assert!(debug.contains("[REDACTED]"));
    assert!(!debug.contains("secret@example.com"));
    assert!(!debug.contains("private-tenant"));
    assert!(!debug.contains("secret-v3"));
}

#[test]
fn avatar_builder_can_use_automatic_style_and_cache_key() {
    let image = AvatarBuilder::for_id("auto@example.com")
        .size(96, 96)
        .automatic_style()
        .render()
        .expect("automatic builder style should render");
    let cache_key = AvatarBuilder::for_id("auto@example.com")
        .cache_key()
        .expect("cache key should be derived");

    assert_eq!(image.width(), 96);
    assert_eq!(image.height(), 96);
    assert_eq!(cache_key.len(), 64);
}

#[test]
fn svg_export_contains_svg_root_and_kind_label() {
    let svg = render_avatar_svg_for_id(
        valid_spec(128, 128, 0),
        "vector@example.com",
        AvatarOptions::new(AvatarKind::Fox, AvatarBackground::White),
    );

    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("fox avatar"));
}

#[test]
fn svg_output_is_minimal_and_safe() {
    let svg = render_avatar_svg_for_id(
        valid_spec(256, 256, 0),
        "ghost@example.com",
        AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
    );

    assert!(!svg.contains("<script"));
    assert!(!svg.contains("onload="));
    assert!(svg.len() < 8_000);
}

#[test]
fn svg_output_is_well_formed_xml_for_all_avatar_kinds() {
    let spec = valid_spec(128, 128, 0);
    let identities: [&[u8]; 4] = [
        b"alice@example.com",
        b"\0\0\0\0",
        b"<not-svg attr=\"x\">&",
        b"0123456789abcdef0123456789abcdef0123456789abcdef",
    ];

    for &kind in AvatarKind::ALL {
        for &background in AvatarBackground::ALL {
            for identity in identities {
                let svg =
                    render_avatar_svg_for_id(spec, identity, AvatarOptions::new(kind, background));
                assert_svg_is_well_formed(&svg);
            }
        }
    }
}

#[test]
fn styled_svg_output_is_well_formed_xml_for_all_layer_options() {
    let spec = valid_spec(96, 96, 0);
    let kinds = [AvatarKind::Robot, AvatarKind::Shield];

    for &kind in &kinds {
        for &accessory in AvatarAccessory::ALL {
            let style = AvatarStyleOptions::new(
                kind,
                AvatarBackground::Themed,
                accessory,
                AvatarColor::Default,
                AvatarExpression::Default,
                AvatarShape::Square,
            );
            assert_svg_is_well_formed(&render_avatar_svg_style_for_id(
                spec,
                "accessory-xml@example.com",
                style,
            ));
        }

        for &color in AvatarColor::ALL {
            let style = AvatarStyleOptions::new(
                kind,
                AvatarBackground::Themed,
                AvatarAccessory::None,
                color,
                AvatarExpression::Default,
                AvatarShape::Square,
            );
            assert_svg_is_well_formed(&render_avatar_svg_style_for_id(
                spec,
                "color-xml@example.com",
                style,
            ));
        }

        for &expression in AvatarExpression::ALL {
            let style = AvatarStyleOptions::new(
                kind,
                AvatarBackground::Themed,
                AvatarAccessory::None,
                AvatarColor::Default,
                expression,
                AvatarShape::Square,
            );
            assert_svg_is_well_formed(&render_avatar_svg_style_for_id(
                spec,
                "expression-xml@example.com",
                style,
            ));
        }

        for &shape in AvatarShape::ALL {
            let style = AvatarStyleOptions::new(
                kind,
                AvatarBackground::Themed,
                AvatarAccessory::None,
                AvatarColor::Default,
                AvatarExpression::Default,
                shape,
            );
            assert_svg_is_well_formed(&render_avatar_svg_style_for_id(
                spec,
                "shape-xml@example.com",
                style,
            ));
        }
    }
}

#[test]
fn transparent_svg_output_has_no_background_rect() {
    let svg = render_avatar_svg_for_id(
        valid_spec(128, 128, 0),
        "cat@hashavatar.app",
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Transparent),
    );

    assert!(!svg.contains(r#"<rect width="100%" height="100%""#));
    assert!(svg.contains("cat avatar"));
}

#[test]
fn svg_radius_attributes_do_not_contain_color_values() {
    let spec = valid_spec(128, 128, 0);
    for &kind in AvatarKind::ALL {
        let svg = render_avatar_svg_for_id(
            spec,
            "svg-radius@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );

        assert!(!svg.contains(r##"rx="#"##), "{kind}");
        assert!(!svg.contains(r##"ry="#"##), "{kind}");
    }
}

#[test]
fn dark_svg_output_has_background_rect() {
    let svg = render_avatar_svg_for_id(
        valid_spec(128, 128, 0),
        "cat@hashavatar.app",
        AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Dark),
    );

    assert!(svg.contains(r##"<rect width="100%" height="100%" fill="#111827"/>"##));
}

#[test]
fn parser_round_trip_supports_public_enums() {
    for &kind in AvatarKind::ALL {
        assert_eq!(kind.as_str().parse::<AvatarKind>().ok(), Some(kind));
        assert_eq!(kind.to_string(), kind.as_str());
    }
    for &background in AvatarBackground::ALL {
        assert_eq!(
            background.as_str().parse::<AvatarBackground>().ok(),
            Some(background)
        );
        assert_eq!(background.to_string(), background.as_str());
    }
    for &format in AvatarOutputFormat::ALL {
        assert_eq!(
            format.as_str().parse::<AvatarOutputFormat>().ok(),
            Some(format)
        );
        assert_eq!(format.to_string(), format.as_str());
    }
    for &accessory in AvatarAccessory::ALL {
        assert_eq!(
            accessory.as_str().parse::<AvatarAccessory>().ok(),
            Some(accessory)
        );
        assert_eq!(accessory.to_string(), accessory.as_str());
    }
    for &color in AvatarColor::ALL {
        assert_eq!(color.as_str().parse::<AvatarColor>().ok(), Some(color));
        assert_eq!(color.to_string(), color.as_str());
    }
    for &expression in AvatarExpression::ALL {
        assert_eq!(
            expression.as_str().parse::<AvatarExpression>().ok(),
            Some(expression)
        );
        assert_eq!(expression.to_string(), expression.as_str());
    }
    for &shape in AvatarShape::ALL {
        assert_eq!(shape.as_str().parse::<AvatarShape>().ok(), Some(shape));
        assert_eq!(shape.to_string(), shape.as_str());
    }
}

#[test]
fn public_enum_variant_lists_match_documented_labels() {
    let kind_labels: Vec<_> = AvatarKind::ALL.iter().map(|kind| kind.as_str()).collect();
    assert_eq!(
        kind_labels,
        [
            "cat",
            "dog",
            "robot",
            "fox",
            "alien",
            "monster",
            "ghost",
            "slime",
            "bird",
            "wizard",
            "skull",
            "paws",
            "planet",
            "rocket",
            "mushroom",
            "cactus",
            "frog",
            "panda",
            "cupcake",
            "pizza",
            "icecream",
            "octopus",
            "knight",
            "bear",
            "penguin",
            "dragon",
            "ninja",
            "astronaut",
            "diamond",
            "coffee-cup",
            "shield",
        ]
    );

    let background_labels: Vec<_> = AvatarBackground::ALL
        .iter()
        .map(|background| background.as_str())
        .collect();
    assert_eq!(
        background_labels,
        [
            "themed",
            "white",
            "black",
            "dark",
            "light",
            "transparent",
            "polka-dot",
            "striped",
            "checkerboard",
            "grid",
            "sunrise",
            "ocean",
            "starry",
        ]
    );

    let format_labels: Vec<_> = AvatarOutputFormat::ALL
        .iter()
        .map(|format| format.as_str())
        .collect();
    assert_eq!(
        format_labels,
        [
            "webp",
            #[cfg(feature = "png")]
            "png",
            #[cfg(feature = "jpeg")]
            "jpg",
            #[cfg(feature = "gif")]
            "gif",
        ]
    );

    let accessory_labels: Vec<_> = AvatarAccessory::ALL
        .iter()
        .map(|accessory| accessory.as_str())
        .collect();
    assert_eq!(
        accessory_labels,
        [
            "none",
            "glasses",
            "hat",
            "headphones",
            "crown",
            "bowtie",
            "eyepatch",
            "scarf",
            "halo",
            "horns",
        ]
    );

    let color_labels: Vec<_> = AvatarColor::ALL
        .iter()
        .map(|color| color.as_str())
        .collect();
    assert_eq!(
        color_labels,
        [
            "default",
            "neon-mint",
            "pastel-pink",
            "crimson",
            "gold",
            "deep-sea-blue",
        ]
    );

    let expression_labels: Vec<_> = AvatarExpression::ALL
        .iter()
        .map(|expression| expression.as_str())
        .collect();
    assert_eq!(
        expression_labels,
        [
            "default",
            "happy",
            "grumpy",
            "surprised",
            "sleepy",
            "winking",
            "cool",
            "crying",
        ]
    );

    let shape_labels: Vec<_> = AvatarShape::ALL
        .iter()
        .map(|shape| shape.as_str())
        .collect();
    assert_eq!(
        shape_labels,
        ["square", "circle", "squircle", "hexagon", "octagon"]
    );
}

#[test]
fn byte_to_public_enum_helpers_use_variant_lists() {
    for (index, &kind) in AvatarKind::ALL.iter().enumerate() {
        assert_eq!(AvatarKind::from_byte(index as u8), kind);
    }
    assert_eq!(
        AvatarKind::from_byte(AvatarKind::ALL.len() as u8),
        AvatarKind::ALL[0]
    );

    for (index, &background) in AvatarBackground::ALL.iter().enumerate() {
        assert_eq!(AvatarBackground::from_byte(index as u8), background);
    }
    assert_eq!(
        AvatarBackground::from_byte(AvatarBackground::ALL.len() as u8),
        AvatarBackground::ALL[0]
    );

    for (index, &format) in AvatarOutputFormat::ALL.iter().enumerate() {
        assert_eq!(AvatarOutputFormat::from_byte(index as u8), format);
    }
    assert_eq!(
        AvatarOutputFormat::from_byte(AvatarOutputFormat::ALL.len() as u8),
        AvatarOutputFormat::ALL[0]
    );

    for (index, &accessory) in AvatarAccessory::ALL.iter().enumerate() {
        assert_eq!(AvatarAccessory::from_byte(index as u8), accessory);
    }
    assert_eq!(
        AvatarAccessory::from_byte(AvatarAccessory::ALL.len() as u8),
        AvatarAccessory::ALL[0]
    );

    for (index, &color) in AvatarColor::ALL.iter().enumerate() {
        assert_eq!(AvatarColor::from_byte(index as u8), color);
    }
    assert_eq!(
        AvatarColor::from_byte(AvatarColor::ALL.len() as u8),
        AvatarColor::ALL[0]
    );

    for (index, &expression) in AvatarExpression::ALL.iter().enumerate() {
        assert_eq!(AvatarExpression::from_byte(index as u8), expression);
    }
    assert_eq!(
        AvatarExpression::from_byte(AvatarExpression::ALL.len() as u8),
        AvatarExpression::ALL[0]
    );

    for (index, &shape) in AvatarShape::ALL.iter().enumerate() {
        assert_eq!(AvatarShape::from_byte(index as u8), shape);
    }
    assert_eq!(
        AvatarShape::from_byte(AvatarShape::ALL.len() as u8),
        AvatarShape::ALL[0]
    );
}

#[test]
#[cfg(feature = "serde")]
fn serde_feature_round_trips_public_style_enums_as_strings() {
    assert_eq!(
        serde_json::to_string(&AvatarKind::Robot).expect("kind should serialize"),
        "\"robot\""
    );
    assert_eq!(
        serde_json::from_str::<AvatarKind>("\"coffee-cup\"").expect("kind should deserialize"),
        AvatarKind::CoffeeCup
    );
    assert_eq!(
        serde_json::from_str::<AvatarBackground>("\"polka-dot\"")
            .expect("background should deserialize"),
        AvatarBackground::PolkaDot
    );
    assert_eq!(
        serde_json::from_str::<AvatarAccessory>("\"eyepatch\"")
            .expect("accessory should deserialize"),
        AvatarAccessory::Eyepatch
    );
    assert_eq!(
        serde_json::from_str::<AvatarColor>("\"deep-sea-blue\"").expect("color should deserialize"),
        AvatarColor::DeepSeaBlue
    );
    assert_eq!(
        serde_json::from_str::<AvatarExpression>("\"winking\"")
            .expect("expression should deserialize"),
        AvatarExpression::Winking
    );
    assert_eq!(
        serde_json::from_str::<AvatarShape>("\"hexagon\"").expect("shape should deserialize"),
        AvatarShape::Hexagon
    );
    let library_source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/core.rs"));
    assert!(!library_source.contains("impl serde::Serialize for AvatarIdentity"));
    assert!(!library_source.contains("impl<'de> serde::Deserialize<'de> for AvatarIdentity"));
}

#[test]
#[cfg(feature = "gif")]
fn gif_variant_has_rustdoc_security_warning() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/model.rs"));
    let variant_docs = source
        .split("Gif,")
        .next()
        .and_then(|before_variant| before_variant.rsplit_once("/// Optional GIF output."))
        .map(|(_, docs)| docs)
        .expect("gif variant docs should be present");

    assert!(variant_docs.contains("# Warning"));
    assert!(variant_docs.contains("256-color quantization"));
    assert!(variant_docs.contains("not sanitized"));
    assert!(variant_docs.contains("high-assurance deployments"));
    assert!(variant_docs.contains("AvatarOutputFormat::WebP"));
    assert!(variant_docs.contains("PNG output"));
}

#[test]
fn internal_render_plan_matches_direct_raster_renderer() {
    let spec = valid_spec(96, 96, 0);
    let options = AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Dark);
    let plan = AvatarRenderPlan::new(
        spec,
        AvatarIdentityOptions::default(),
        "plan@example.com",
        options,
    )
    .expect("render plan should be valid");
    let identity = valid_identity("plan@example.com");
    let direct = render_robot_avatar_for_identity(spec, &identity, AvatarBackground::Dark);

    assert_eq!(
        plan.render_rgba()
            .expect("planned robot render should be valid")
            .as_raw(),
        direct.as_raw()
    );
    assert!(plan.render_svg().contains("robot avatar"));
}

#[test]
fn avatar_style_options_from_legacy_options_is_noop() {
    let spec = valid_spec(128, 128, 0);
    let options = AvatarOptions::new(AvatarKind::Robot, AvatarBackground::Themed);
    let style = AvatarStyleOptions::from_options(options);

    let legacy = render_avatar_for_id(spec, "style@example.com", options);
    let styled = render_avatar_style_for_id(spec, "style@example.com", style);
    assert_eq!(legacy.as_raw(), styled.as_raw());

    let legacy_svg = render_avatar_svg_for_id(spec, "style@example.com", options);
    let styled_svg = render_avatar_svg_style_for_id(spec, "style@example.com", style);
    assert_eq!(legacy_svg, styled_svg);
}

#[test]
fn avatar_style_options_has_human_readable_summary() {
    let style = AvatarStyleOptions::new(
        AvatarKind::Robot,
        AvatarBackground::Ocean,
        AvatarAccessory::Glasses,
        AvatarColor::Gold,
        AvatarExpression::Happy,
        AvatarShape::Circle,
    );

    assert_eq!(
        style.summary(),
        "robot / ocean / glasses / gold / happy / circle"
    );
    assert_eq!(style.summary(), style.to_string());
}

#[test]
fn automatic_style_derivation_uses_distinct_digest_offsets() {
    let base = AvatarStyleOptions::from_identity(&identity_with_digest_byte(63, 99));

    let kind =
        AvatarStyleOptions::from_identity(&identity_with_digest_byte(AVATAR_STYLE_KIND_BYTE, 1));
    assert_ne!(kind.kind, base.kind);
    assert_eq!(kind.background, base.background);
    assert_eq!(kind.accessory, base.accessory);
    assert_eq!(kind.color, base.color);
    assert_eq!(kind.expression, base.expression);
    assert_eq!(kind.shape, base.shape);

    let background = AvatarStyleOptions::from_identity(&identity_with_digest_byte(
        AVATAR_STYLE_BACKGROUND_BYTE,
        1,
    ));
    assert_eq!(background.kind, base.kind);
    assert_ne!(background.background, base.background);
    assert_eq!(background.accessory, base.accessory);
    assert_eq!(background.color, base.color);
    assert_eq!(background.expression, base.expression);
    assert_eq!(background.shape, base.shape);

    let accessory = AvatarStyleOptions::from_identity(&identity_with_digest_byte(
        AVATAR_STYLE_ACCESSORY_BYTE,
        1,
    ));
    assert_eq!(accessory.kind, base.kind);
    assert_eq!(accessory.background, base.background);
    assert_ne!(accessory.accessory, base.accessory);
    assert_eq!(accessory.color, base.color);
    assert_eq!(accessory.expression, base.expression);
    assert_eq!(accessory.shape, base.shape);

    let color =
        AvatarStyleOptions::from_identity(&identity_with_digest_byte(AVATAR_STYLE_COLOR_BYTE, 1));
    assert_eq!(color.kind, base.kind);
    assert_eq!(color.background, base.background);
    assert_eq!(color.accessory, base.accessory);
    assert_ne!(color.color, base.color);
    assert_eq!(color.expression, base.expression);
    assert_eq!(color.shape, base.shape);

    let expression = AvatarStyleOptions::from_identity(&identity_with_digest_byte(
        AVATAR_STYLE_EXPRESSION_BYTE,
        1,
    ));
    assert_eq!(expression.kind, base.kind);
    assert_eq!(expression.background, base.background);
    assert_eq!(expression.accessory, base.accessory);
    assert_eq!(expression.color, base.color);
    assert_ne!(expression.expression, base.expression);
    assert_eq!(expression.shape, base.shape);

    let shape =
        AvatarStyleOptions::from_identity(&identity_with_digest_byte(AVATAR_STYLE_SHAPE_BYTE, 1));
    assert_eq!(shape.kind, base.kind);
    assert_eq!(shape.background, base.background);
    assert_eq!(shape.accessory, base.accessory);
    assert_eq!(shape.color, base.color);
    assert_eq!(shape.expression, base.expression);
    assert_ne!(shape.shape, base.shape);
}

#[test]
fn automatic_style_derivation_is_deterministic() {
    let identity = valid_identity("auto-style@example.com");

    assert_eq!(
        AvatarStyleOptions::from_identity(&identity),
        AvatarStyleOptions::from_identity(&identity)
    );
    assert_eq!(
        super::render_avatar_auto_for_id(valid_spec(96, 96, 0), "auto-style@example.com")
            .expect("automatic style should render")
            .as_raw(),
        super::render_avatar_auto_for_id(valid_spec(96, 96, 0), "auto-style@example.com")
            .expect("automatic style should render")
            .as_raw()
    );
}

#[test]
fn manual_style_selection_changes_raster_and_svg() {
    let spec = valid_spec(128, 128, 0);
    let legacy_options = AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed);
    let layered_style = AvatarStyleOptions::new(
        AvatarKind::Cat,
        AvatarBackground::Themed,
        AvatarAccessory::Glasses,
        AvatarColor::Gold,
        AvatarExpression::Happy,
        AvatarShape::Circle,
    );

    let legacy = render_avatar_for_id(spec, "manual-style@example.com", legacy_options);
    let layered = render_avatar_style_for_id(spec, "manual-style@example.com", layered_style);
    assert_ne!(legacy.as_raw(), layered.as_raw());

    let svg = render_avatar_svg_style_for_id(spec, "manual-style@example.com", layered_style);
    assert!(svg.contains(r#"data-layer="accessory-glasses""#));
    assert!(svg.contains(r#"data-layer="expression-happy""#));
    assert!(svg.contains(r#"data-layer="shape-circle""#));
}

#[test]
fn avatar_kind_face_layer_support_matches_anchor_coverage() {
    let supported = [
        AvatarKind::Cat,
        AvatarKind::Dog,
        AvatarKind::Robot,
        AvatarKind::Fox,
        AvatarKind::Alien,
        AvatarKind::Monster,
        AvatarKind::Ghost,
        AvatarKind::Slime,
        AvatarKind::Bird,
        AvatarKind::Wizard,
        AvatarKind::Skull,
        AvatarKind::Frog,
        AvatarKind::Panda,
        AvatarKind::Octopus,
        AvatarKind::Knight,
        AvatarKind::Bear,
        AvatarKind::Penguin,
        AvatarKind::Dragon,
        AvatarKind::Ninja,
        AvatarKind::Astronaut,
    ];

    for &kind in AvatarKind::ALL {
        assert_eq!(
            kind.supports_face_layers(),
            supported.contains(&kind),
            "{kind}"
        );
        assert_eq!(
            avatar_layer_anchors(kind).is_some(),
            kind.supports_face_layers(),
            "{kind}"
        );
    }
}

#[test]
fn face_layer_families_emit_accessory_and_expression_svg_layers() {
    let spec = valid_spec(96, 96, 0);

    for &kind in AvatarKind::ALL {
        if !kind.supports_face_layers() {
            continue;
        }

        let style = AvatarStyleOptions::new(
            kind,
            AvatarBackground::Themed,
            AvatarAccessory::Eyepatch,
            AvatarColor::Gold,
            AvatarExpression::Winking,
            AvatarShape::Square,
        );
        let image = render_avatar_style_for_id(spec, "face-layer@example.com", style);
        assert_eq!(image.dimensions(), (96, 96), "{kind}");

        let svg = render_avatar_svg_style_for_id(spec, "face-layer@example.com", style);
        assert!(svg.contains(r#"data-layer="accessory-eyepatch""#), "{kind}");
        assert!(svg.contains(r#"data-layer="expression-winking""#), "{kind}");
    }
}

#[test]
fn style_layers_render_for_all_baseline_variants() {
    let spec = valid_spec(96, 96, 0);

    for &accessory in AvatarAccessory::ALL {
        let style = AvatarStyleOptions::new(
            AvatarKind::Robot,
            AvatarBackground::Themed,
            accessory,
            AvatarColor::NeonMint,
            AvatarExpression::Default,
            AvatarShape::Square,
        );
        let image = render_avatar_style_for_id(spec, "accessory@example.com", style);
        assert_eq!(image.width(), 96, "{accessory}");
        assert!(
            render_avatar_svg_style_for_id(spec, "accessory@example.com", style)
                .starts_with("<svg ")
        );
    }

    for &color in AvatarColor::ALL {
        let style = AvatarStyleOptions::new(
            AvatarKind::Robot,
            AvatarBackground::Themed,
            AvatarAccessory::None,
            color,
            AvatarExpression::Default,
            AvatarShape::Square,
        );
        let image = render_avatar_style_for_id(spec, "color@example.com", style);
        assert_eq!(image.height(), 96, "{color}");
        assert!(
            render_avatar_svg_style_for_id(spec, "color@example.com", style).starts_with("<svg ")
        );
    }

    for &expression in AvatarExpression::ALL {
        let style = AvatarStyleOptions::new(
            AvatarKind::Robot,
            AvatarBackground::Themed,
            AvatarAccessory::None,
            AvatarColor::Default,
            expression,
            AvatarShape::Square,
        );
        let image = render_avatar_style_for_id(spec, "expression@example.com", style);
        assert_eq!(image.width(), 96, "{expression}");
        assert!(
            render_avatar_svg_style_for_id(spec, "expression@example.com", style)
                .starts_with("<svg ")
        );
    }

    for &shape in AvatarShape::ALL {
        let style = AvatarStyleOptions::new(
            AvatarKind::Robot,
            AvatarBackground::Themed,
            AvatarAccessory::None,
            AvatarColor::Default,
            AvatarExpression::Default,
            shape,
        );
        let image = render_avatar_style_for_id(spec, "shape@example.com", style);
        assert_eq!(image.height(), 96, "{shape}");
        assert!(
            render_avatar_svg_style_for_id(spec, "shape@example.com", style).starts_with("<svg ")
        );
    }
}

#[test]
fn unsupported_family_accessories_and_expressions_are_skipped() {
    let spec = valid_spec(128, 128, 0);
    for &kind in AvatarKind::ALL {
        if kind.supports_face_layers() {
            continue;
        }

        let baseline_options = AvatarOptions::new(kind, AvatarBackground::Themed);
        let unsupported_style = AvatarStyleOptions::new(
            kind,
            AvatarBackground::Themed,
            AvatarAccessory::Eyepatch,
            AvatarColor::Default,
            AvatarExpression::Winking,
            AvatarShape::Square,
        );

        let baseline =
            render_avatar_for_id(spec, "unsupported-layer@example.com", baseline_options);
        let unsupported =
            render_avatar_style_for_id(spec, "unsupported-layer@example.com", unsupported_style);
        assert_eq!(baseline.as_raw(), unsupported.as_raw(), "{kind}");

        let svg = render_avatar_svg_style_for_id(
            spec,
            "unsupported-layer@example.com",
            unsupported_style,
        );
        assert!(!svg.contains("accessory-eyepatch"), "{kind}");
        assert!(!svg.contains("expression-winking"), "{kind}");
    }
}

#[test]
fn non_square_svg_frame_shapes_clip_content() {
    let spec = valid_spec(128, 128, 0);
    let shaped = AvatarStyleOptions::new(
        AvatarKind::Robot,
        AvatarBackground::White,
        AvatarAccessory::None,
        AvatarColor::Default,
        AvatarExpression::Default,
        AvatarShape::Hexagon,
    );
    let svg = render_avatar_svg_style_for_id(spec, "shape-clip@example.com", shaped);

    assert!(svg.contains(r#"<defs><clipPath id="hashavatar-frame-clip">"#));
    assert!(svg.contains(r#"<g clip-path="url(#hashavatar-frame-clip)">"#));
    assert!(svg.contains(r#"data-layer="shape-hexagon""#));

    let square = render_avatar_svg_style_for_id(
        spec,
        "shape-clip@example.com",
        AvatarStyleOptions::from_options(AvatarOptions::new(
            AvatarKind::Robot,
            AvatarBackground::White,
        )),
    );
    assert!(!square.contains("clipPath"));
    assert!(!square.contains("shape-hexagon"));
}

#[test]
fn avatar_spec_validation_rejects_resource_extremes() {
    assert!(AvatarSpec::new(MIN_AVATAR_DIMENSION, MIN_AVATAR_DIMENSION, 0).is_ok());
    assert!(AvatarSpec::new(MAX_AVATAR_DIMENSION, MAX_AVATAR_DIMENSION, 0).is_ok());

    let too_small = AvatarSpec::new(MIN_AVATAR_DIMENSION - 1, 256, 0)
        .expect_err("undersized width should be rejected");
    let too_large = AvatarSpec::new(256, MAX_AVATAR_DIMENSION + 1, 0)
        .expect_err("oversized height should be rejected");

    assert_eq!(too_small.width(), MIN_AVATAR_DIMENSION - 1);
    assert_eq!(too_large.height(), MAX_AVATAR_DIMENSION + 1);
}

#[test]
fn avatar_spec_reports_raw_rgba_buffer_budget() {
    let spec = valid_spec(MAX_AVATAR_DIMENSION, MAX_AVATAR_DIMENSION, 0);

    assert_eq!(spec.pixel_count(), MAX_AVATAR_PIXELS);
    assert_eq!(spec.rgba_buffer_len(), MAX_AVATAR_RGBA_BYTES);
    assert_eq!(
        MAX_AVATAR_RGBA_BYTES,
        2048_usize * 2048_usize * AVATAR_RGBA_BYTES_PER_PIXEL
    );
}

#[test]
fn avatar_spec_size_helpers_saturate_for_unchecked_future_values() {
    let spec = AvatarSpec::new_unchecked(u32::MAX, u32::MAX, 0);

    assert_eq!(
        spec.pixel_count(),
        (u32::MAX as usize).saturating_mul(u32::MAX as usize)
    );
    assert_eq!(spec.rgba_buffer_len(), usize::MAX);
}

#[test]
fn render_resource_budget_makes_concurrency_memory_math_explicit() {
    let spec = valid_spec(256, 128, 0);
    let budget = spec.render_resource_budget(8);

    assert_eq!(budget.spec(), spec);
    assert_eq!(budget.concurrent_renders(), 8);
    assert_eq!(budget.raw_rgba_bytes_per_render(), 256 * 128 * 4);
    assert_eq!(
        budget.raw_rgba_bytes_for_concurrent_renders(),
        8 * 256 * 128 * 4
    );
    assert_eq!(
        AvatarRenderResourceBudget::max_concurrent_renders_for_memory_budget(
            spec,
            16 * 256 * 128 * 4
        ),
        16
    );
}

#[test]
fn render_resource_budget_saturates_concurrent_byte_estimates() {
    let spec = valid_spec(MAX_AVATAR_DIMENSION, MAX_AVATAR_DIMENSION, 0);
    let budget = spec.render_resource_budget(usize::MAX);

    assert_eq!(budget.raw_rgba_bytes_for_concurrent_renders(), usize::MAX);
    assert_eq!(
        AvatarRenderResourceBudget::max_supported_raw_rgba_bytes_for_concurrent_renders(usize::MAX),
        usize::MAX
    );
}

#[test]
fn avatar_spec_default_is_fixed_and_supported() {
    let default = AvatarSpec::default();
    let explicit = valid_spec(256, 256, 1);

    assert_eq!(default, explicit);
    assert!(default.is_supported());
    assert_eq!(default.width(), 256);
    assert_eq!(default.height(), 256);
    assert_eq!(default.seed(), 1);
}

#[test]
fn rect_edges_saturate_on_extreme_coordinates() {
    let rect = Rect {
        left: i32::MAX,
        top: i32::MAX,
        width: 64,
        height: 64,
    };

    assert_eq!(rect.right(), i32::MAX);
    assert_eq!(rect.bottom(), i32::MAX);
}

#[test]
fn rect_intersection_size_saturates_on_extreme_coordinates() {
    let rect = Rect {
        left: i32::MIN,
        top: i32::MIN,
        width: u32::MAX,
        height: u32::MAX,
    };

    let intersection = rect
        .intersect(rect)
        .expect("extreme rectangles should intersect");

    assert_eq!(intersection.left(), i32::MIN);
    assert_eq!(intersection.top(), i32::MIN);
    assert_eq!(intersection.width(), i32::MAX as u32);
    assert_eq!(intersection.height(), i32::MAX as u32);
}

#[test]
fn rect_size_builder_clamps_zero_dimensions() {
    let rect = Rect::at(4, 8).of_size(0, 0);

    assert_eq!(rect.width(), 1);
    assert_eq!(rect.height(), 1);
}

#[test]
fn minimum_size_renders_all_avatar_families_without_artifact_panics() {
    let spec = valid_spec(MIN_AVATAR_DIMENSION, MIN_AVATAR_DIMENSION, 0);
    for kind in AvatarKind::ALL {
        let image = render_avatar_for_id(
            spec,
            "minimum-size@example.com",
            AvatarOptions::new(*kind, AvatarBackground::Themed),
        );

        assert_eq!(image.width(), MIN_AVATAR_DIMENSION, "{kind}");
        assert_eq!(image.height(), MIN_AVATAR_DIMENSION, "{kind}");
        assert!(image.pixels().any(|pixel| pixel.0[3] != 0), "{kind}");
    }
}

#[test]
fn avatar_identity_implements_secure_sanitize() {
    fn assert_secure_sanitize<T: SecureSanitize>() {}

    assert_secure_sanitize::<AvatarIdentity>();
}

#[test]
fn antialiased_zero_length_line_draws_single_pixel() {
    let mut image = RgbaImage::new(4, 4);

    draw_antialiased_line_segment_mut(
        &mut image,
        (1, 1),
        (1, 1),
        Rgba([10, 20, 30, 255]),
        interpolate,
    );

    assert_eq!(image.get_pixel(1, 1), &Rgba([10, 20, 30, 255]));
}

#[test]
fn polygon_rasterizer_skips_unpaired_intersections() {
    let mut image = RgbaImage::new(32, 32);
    let triangle_with_horizontal_base = [Point::new(0, 0), Point::new(16, 16), Point::new(31, 0)];

    draw_polygon_mut(
        &mut image,
        &triangle_with_horizontal_base,
        Rgba([255, 0, 0, 255]),
    );

    assert!(image.pixels().any(|pixel| pixel.0[3] == 255));
}

#[test]
fn polygon_rasterizer_skips_zero_sized_images() {
    let mut zero_width = RgbaImage::new(0, 8);
    let mut zero_height = RgbaImage::new(8, 0);
    let color = Rgba([255, 0, 0, 255]);
    let poly = [Point::new(0, 0), Point::new(4, 0), Point::new(0, 4)];

    draw_polygon_mut(&mut zero_width, &poly, color);
    draw_polygon_mut(&mut zero_height, &poly, color);

    assert!(zero_width.is_empty());
    assert!(zero_height.is_empty());
}

#[test]
fn ellipse_rasterizer_handles_max_supported_radius() {
    let mut image = RgbaImage::new(1, 1);
    let mut render_calls = 0;

    draw_ellipse(
        |_, _, _, _, _| render_calls += 1,
        &mut image,
        (
            MAX_AVATAR_DIMENSION as i32 / 2,
            MAX_AVATAR_DIMENSION as i32 / 2,
        ),
        MAX_AVATAR_DIMENSION as i32 / 2,
        MAX_AVATAR_DIMENSION as i32 / 2,
    );

    assert!(render_calls > 0);
}

#[test]
fn jpeg_alpha_flattening_uses_wide_intermediates() {
    let image = RgbaImage::from_vec(
        3,
        1,
        vec![
            0, 0, 0, 0, // transparent black over white
            0, 0, 0, 128, // half alpha black over white
            10, 20, 30, 255, // opaque color
        ],
    )
    .expect("test image should be valid");

    let rgb = rgba_to_rgb_over_white(&image);

    assert_eq!(
        rgb,
        vec![
            255, 255, 255, // transparent becomes white
            127, 127, 127, // rounded half-alpha black over white
            10, 20, 30,
        ]
    );
}

#[test]
fn starry_background_pattern_depends_on_identity() {
    let mut left = RgbaImage::new(128, 128);
    let mut right = RgbaImage::new(128, 128);

    draw_starry_background(&mut left, &valid_identity("alice@example.com"));
    draw_starry_background(&mut right, &valid_identity("bob@example.com"));

    assert_ne!(left.as_raw(), right.as_raw());
}

#[test]
fn lerp_channel_clamps_position_to_prevent_underflow() {
    assert_eq!(lerp_channel_u32(0, 255, 10, 5), 255);
    assert_eq!(lerp_channel_u32(255, 0, 10, 5), 0);
}

#[test]
fn rgba_pixel_sanitizer_scrubs_owned_render_buffers() {
    let mut image = RgbaImage::from_vec(
        2,
        1,
        vec![
            10, 20, 30, 255, // opaque pixel
            40, 50, 60, 128, // translucent pixel
        ],
    )
    .expect("test image should be valid");

    sanitize_rgba_pixels(&mut image);

    assert!(image.as_raw().iter().all(|byte| *byte == 0));
}

#[test]
fn encoded_output_buffer_is_sanitizing_until_success() {
    let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/encoding.rs"));
    let helper = source
        .split("fn encode_rgba_image")
        .nth(1)
        .and_then(|after_name| after_name.split("fn encode_owned_rgba_image").next())
        .expect("encode helper should exist");

    assert!(helper.contains("SanitizingVec::with_capacity"));
    assert!(helper.contains("encode_into_writer(image, format, cursor)"));
    assert!(helper.contains("Ok(()) => Ok(bytes.into_inner())"));
    assert!(helper.contains("Err(error) => Err(error)"));
}

#[test]
fn weighted_channel_sum_rejects_invalid_weights() {
    assert_eq!(weighted_channel_sum(255, 0, 0.25, 0.75), 63);
    assert_eq!(weighted_channel_sum(255, 0, 0.0, 0.0), 0);
    assert_eq!(weighted_channel_sum(255, 0, f32::NAN, 1.0), 0);
    assert_eq!(weighted_channel_sum(255, 0, f32::INFINITY, 1.0), 0);
}

#[test]
fn render_avatar_for_id_supports_all_avatar_kinds() {
    let spec = valid_spec(96, 96, 0);
    for &kind in AvatarKind::ALL {
        let image = render_avatar_for_id(
            spec,
            "integration@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );
        assert_eq!(image.width(), 96);
        assert_eq!(image.height(), 96);
    }
}

#[test]
fn render_avatar_svg_for_id_supports_all_avatar_kinds() {
    let spec = valid_spec(96, 96, 0);
    for &kind in AvatarKind::ALL {
        let svg = render_avatar_svg_for_id(
            spec,
            "integration@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );

        assert!(svg.contains("<svg"));
        assert!(svg.contains(&format!("{kind} avatar")));
    }
}

#[test]
fn lower_variation_presets_change_for_different_identities() {
    let spec = valid_spec(128, 128, 0);
    for kind in [
        AvatarKind::Ghost,
        AvatarKind::Slime,
        AvatarKind::Wizard,
        AvatarKind::Skull,
    ] {
        let left = render_avatar_for_id(
            spec,
            "alice@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );
        let right = render_avatar_for_id(
            spec,
            "bob@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );

        assert_ne!(
            image_fingerprint(&left),
            image_fingerprint(&right),
            "{kind}"
        );
    }
}

#[test]
fn lower_variation_svg_presets_change_for_different_identities() {
    let spec = valid_spec(128, 128, 0);
    for kind in [
        AvatarKind::Ghost,
        AvatarKind::Slime,
        AvatarKind::Wizard,
        AvatarKind::Skull,
    ] {
        let left = render_avatar_svg_for_id(
            spec,
            "alice@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );
        let right = render_avatar_svg_for_id(
            spec,
            "bob@example.com",
            AvatarOptions::new(kind, AvatarBackground::Themed),
        );

        assert_ne!(left, right, "{kind}");
    }
}

#[test]
#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn visual_fingerprints_are_stable() {
    for (label, options) in regression_scenarios() {
        let image = render_avatar_for_id(valid_spec(128, 128, 0), "snapshot@example.com", options);
        let fingerprint = image_fingerprint(&image);
        let expected =
            regression_fingerprint_for(label).expect("missing golden regression fingerprint");
        assert_eq!(fingerprint, expected, "fingerprint mismatch for {label}");
    }

    for (label, style) in style_regression_scenarios() {
        let image =
            render_avatar_style_for_id(valid_spec(128, 128, 0), "snapshot@example.com", style);
        let fingerprint = image_fingerprint(&image);
        let expected =
            regression_fingerprint_for(label).expect("missing golden regression fingerprint");
        assert_eq!(fingerprint, expected, "fingerprint mismatch for {label}");
    }

    let auto = super::render_avatar_auto_for_id(valid_spec(128, 128, 0), "snapshot@example.com")
        .expect("automatic avatar should render");
    let auto_fingerprint = image_fingerprint(&auto);
    let auto_expected =
        regression_fingerprint_for("auto-layered").expect("missing golden auto fingerprint");
    assert_eq!(
        auto_fingerprint, auto_expected,
        "fingerprint mismatch for auto-layered"
    );
}

#[ignore]
#[test]
#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn print_visual_fingerprints() {
    for (label, options) in [
        (
            "cat-themed",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
        ),
        (
            "cat-white",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White),
        ),
        (
            "dog-themed",
            AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
        ),
        (
            "robot-white",
            AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White),
        ),
        (
            "monster-themed",
            AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed),
        ),
        (
            "ghost-themed",
            AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
        ),
        (
            "slime-white",
            AvatarOptions::new(AvatarKind::Slime, AvatarBackground::White),
        ),
        (
            "bird-themed",
            AvatarOptions::new(AvatarKind::Bird, AvatarBackground::Themed),
        ),
        (
            "wizard-white",
            AvatarOptions::new(AvatarKind::Wizard, AvatarBackground::White),
        ),
        (
            "skull-themed",
            AvatarOptions::new(AvatarKind::Skull, AvatarBackground::Themed),
        ),
        (
            "paws-themed",
            AvatarOptions::new(AvatarKind::Paws, AvatarBackground::Themed),
        ),
        (
            "planet-themed",
            AvatarOptions::new(AvatarKind::Planet, AvatarBackground::Themed),
        ),
        (
            "rocket-themed",
            AvatarOptions::new(AvatarKind::Rocket, AvatarBackground::Themed),
        ),
        (
            "mushroom-themed",
            AvatarOptions::new(AvatarKind::Mushroom, AvatarBackground::Themed),
        ),
        (
            "cactus-themed",
            AvatarOptions::new(AvatarKind::Cactus, AvatarBackground::Themed),
        ),
        (
            "frog-themed",
            AvatarOptions::new(AvatarKind::Frog, AvatarBackground::Themed),
        ),
        (
            "panda-themed",
            AvatarOptions::new(AvatarKind::Panda, AvatarBackground::Themed),
        ),
        (
            "cupcake-themed",
            AvatarOptions::new(AvatarKind::Cupcake, AvatarBackground::Themed),
        ),
        (
            "pizza-themed",
            AvatarOptions::new(AvatarKind::Pizza, AvatarBackground::Themed),
        ),
        (
            "icecream-themed",
            AvatarOptions::new(AvatarKind::Icecream, AvatarBackground::Themed),
        ),
        (
            "octopus-themed",
            AvatarOptions::new(AvatarKind::Octopus, AvatarBackground::Themed),
        ),
        (
            "knight-themed",
            AvatarOptions::new(AvatarKind::Knight, AvatarBackground::Themed),
        ),
        (
            "bear-themed",
            AvatarOptions::new(AvatarKind::Bear, AvatarBackground::Themed),
        ),
        (
            "penguin-themed",
            AvatarOptions::new(AvatarKind::Penguin, AvatarBackground::Themed),
        ),
        (
            "dragon-themed",
            AvatarOptions::new(AvatarKind::Dragon, AvatarBackground::Themed),
        ),
        (
            "ninja-themed",
            AvatarOptions::new(AvatarKind::Ninja, AvatarBackground::Themed),
        ),
        (
            "astronaut-themed",
            AvatarOptions::new(AvatarKind::Astronaut, AvatarBackground::Themed),
        ),
        (
            "diamond-themed",
            AvatarOptions::new(AvatarKind::Diamond, AvatarBackground::Themed),
        ),
        (
            "coffee-cup-themed",
            AvatarOptions::new(AvatarKind::CoffeeCup, AvatarBackground::Themed),
        ),
        (
            "shield-themed",
            AvatarOptions::new(AvatarKind::Shield, AvatarBackground::Themed),
        ),
    ] {
        let image = render_avatar_for_id(valid_spec(128, 128, 0), "snapshot@example.com", options);
        println!("{label}: {}", image_fingerprint(&image));
    }

    for (label, style) in style_regression_scenarios() {
        let image =
            render_avatar_style_for_id(valid_spec(128, 128, 0), "snapshot@example.com", style);
        println!("{label}: {}", image_fingerprint(&image));
    }

    let auto = super::render_avatar_auto_for_id(valid_spec(128, 128, 0), "snapshot@example.com")
        .expect("automatic avatar should render");
    println!("auto-layered: {}", image_fingerprint(&auto));
}

#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn regression_scenarios() -> [(&'static str, AvatarOptions); 30] {
    [
        (
            "cat-themed",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::Themed),
        ),
        (
            "cat-white",
            AvatarOptions::new(AvatarKind::Cat, AvatarBackground::White),
        ),
        (
            "dog-themed",
            AvatarOptions::new(AvatarKind::Dog, AvatarBackground::Themed),
        ),
        (
            "robot-white",
            AvatarOptions::new(AvatarKind::Robot, AvatarBackground::White),
        ),
        (
            "monster-themed",
            AvatarOptions::new(AvatarKind::Monster, AvatarBackground::Themed),
        ),
        (
            "ghost-themed",
            AvatarOptions::new(AvatarKind::Ghost, AvatarBackground::Themed),
        ),
        (
            "slime-white",
            AvatarOptions::new(AvatarKind::Slime, AvatarBackground::White),
        ),
        (
            "bird-themed",
            AvatarOptions::new(AvatarKind::Bird, AvatarBackground::Themed),
        ),
        (
            "wizard-white",
            AvatarOptions::new(AvatarKind::Wizard, AvatarBackground::White),
        ),
        (
            "skull-themed",
            AvatarOptions::new(AvatarKind::Skull, AvatarBackground::Themed),
        ),
        (
            "paws-themed",
            AvatarOptions::new(AvatarKind::Paws, AvatarBackground::Themed),
        ),
        (
            "planet-themed",
            AvatarOptions::new(AvatarKind::Planet, AvatarBackground::Themed),
        ),
        (
            "rocket-themed",
            AvatarOptions::new(AvatarKind::Rocket, AvatarBackground::Themed),
        ),
        (
            "mushroom-themed",
            AvatarOptions::new(AvatarKind::Mushroom, AvatarBackground::Themed),
        ),
        (
            "cactus-themed",
            AvatarOptions::new(AvatarKind::Cactus, AvatarBackground::Themed),
        ),
        (
            "frog-themed",
            AvatarOptions::new(AvatarKind::Frog, AvatarBackground::Themed),
        ),
        (
            "panda-themed",
            AvatarOptions::new(AvatarKind::Panda, AvatarBackground::Themed),
        ),
        (
            "cupcake-themed",
            AvatarOptions::new(AvatarKind::Cupcake, AvatarBackground::Themed),
        ),
        (
            "pizza-themed",
            AvatarOptions::new(AvatarKind::Pizza, AvatarBackground::Themed),
        ),
        (
            "icecream-themed",
            AvatarOptions::new(AvatarKind::Icecream, AvatarBackground::Themed),
        ),
        (
            "octopus-themed",
            AvatarOptions::new(AvatarKind::Octopus, AvatarBackground::Themed),
        ),
        (
            "knight-themed",
            AvatarOptions::new(AvatarKind::Knight, AvatarBackground::Themed),
        ),
        (
            "bear-themed",
            AvatarOptions::new(AvatarKind::Bear, AvatarBackground::Themed),
        ),
        (
            "penguin-themed",
            AvatarOptions::new(AvatarKind::Penguin, AvatarBackground::Themed),
        ),
        (
            "dragon-themed",
            AvatarOptions::new(AvatarKind::Dragon, AvatarBackground::Themed),
        ),
        (
            "ninja-themed",
            AvatarOptions::new(AvatarKind::Ninja, AvatarBackground::Themed),
        ),
        (
            "astronaut-themed",
            AvatarOptions::new(AvatarKind::Astronaut, AvatarBackground::Themed),
        ),
        (
            "diamond-themed",
            AvatarOptions::new(AvatarKind::Diamond, AvatarBackground::Themed),
        ),
        (
            "coffee-cup-themed",
            AvatarOptions::new(AvatarKind::CoffeeCup, AvatarBackground::Themed),
        ),
        (
            "shield-themed",
            AvatarOptions::new(AvatarKind::Shield, AvatarBackground::Themed),
        ),
    ]
}

#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn style_regression_scenarios() -> [(&'static str, AvatarStyleOptions); 4] {
    [
        (
            "style-robot-glasses-gold-happy-circle",
            AvatarStyleOptions::new(
                AvatarKind::Robot,
                AvatarBackground::Themed,
                AvatarAccessory::Glasses,
                AvatarColor::Gold,
                AvatarExpression::Happy,
                AvatarShape::Circle,
            ),
        ),
        (
            "style-fox-halo-neon-cool-squircle",
            AvatarStyleOptions::new(
                AvatarKind::Fox,
                AvatarBackground::White,
                AvatarAccessory::Halo,
                AvatarColor::NeonMint,
                AvatarExpression::Cool,
                AvatarShape::Squircle,
            ),
        ),
        (
            "style-monster-horns-crimson-grumpy-hexagon",
            AvatarStyleOptions::new(
                AvatarKind::Monster,
                AvatarBackground::Dark,
                AvatarAccessory::Horns,
                AvatarColor::Crimson,
                AvatarExpression::Grumpy,
                AvatarShape::Hexagon,
            ),
        ),
        (
            "style-knight-scarf-deepsea-winking-octagon",
            AvatarStyleOptions::new(
                AvatarKind::Knight,
                AvatarBackground::Light,
                AvatarAccessory::Scarf,
                AvatarColor::DeepSeaBlue,
                AvatarExpression::Winking,
                AvatarShape::Octagon,
            ),
        ),
    ]
}

#[cfg(not(any(feature = "blake3", feature = "xxh3")))]
fn regression_fingerprint_for(label: &str) -> Option<&'static str> {
    include_str!("../tests/golden_fingerprints.txt")
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .find_map(|line| {
            let (name, value) = line.split_once('=')?;
            (name.trim() == label).then_some(value.trim())
        })
}

fn image_fingerprint(image: &RgbaImage) -> String {
    let digest = <TestSha512 as sha2::Digest>::digest(image.as_raw());
    digest[..12]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
