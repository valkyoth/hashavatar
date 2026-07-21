//! Public integration tests for the alpha.2 canonical Cat renderer.

use hashavatar::{
    CatError, CatRequest, IdentityComponent, MAX_DIMENSION, MAX_IDENTITY_BYTES,
    MAX_NAMESPACE_COMPONENT_BYTES, MIN_DIMENSION, RgbaSurfaceMut, SvgOptions,
};

#[test]
fn one_request_produces_pixels_and_svg_from_one_scene() -> Result<(), CatError> {
    let prepared = CatRequest::new(192, 160, 42, b"alpha-one-integration")?.prepare()?;
    let report = prepared.scene_report();
    let image = prepared.render_rgba()?;
    let svg = prepared.render_svg()?;

    assert_eq!(image.dimensions(), (192, 160));
    assert_eq!(image.pixels().len(), report.rgba_bytes());
    assert_eq!(report.command_count(), 23);
    assert_eq!(report.path_count(), 2);
    assert_eq!(report.maximum_clip_depth(), 1);
    assert_eq!(report.maximum_opacity_depth(), 1);
    assert!(report.estimated_pixel_tests() >= 192 * 160);
    assert!(distinct_colors(image.pixels()) >= 6);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    Ok(())
}

#[test]
fn canonical_outputs_are_repeatable() -> Result<(), CatError> {
    let first = CatRequest::new(128, 128, 9, b"repeatable")?.prepare()?;
    let second = CatRequest::new(128, 128, 9, b"repeatable")?.prepare()?;

    assert_eq!(first.trait_vector(), second.trait_vector());
    assert_eq!(
        first.render_rgba()?.pixels(),
        second.render_rgba()?.pixels()
    );
    assert_eq!(first.render_svg()?, second.render_svg()?);
    Ok(())
}

#[test]
fn invalid_public_bounds_return_errors() {
    assert!(matches!(
        CatRequest::new(MIN_DIMENSION - 1, 128, 0, b"id"),
        Err(CatError::UnsupportedDimensions { .. })
    ));
    assert!(matches!(
        CatRequest::new(128, MAX_DIMENSION + 1, 0, b"id"),
        Err(CatError::UnsupportedDimensions { .. })
    ));

    let oversized_identity = vec![0_u8; MAX_IDENTITY_BYTES + 1];
    assert!(matches!(
        CatRequest::new(128, 128, 0, &oversized_identity),
        Err(CatError::IdentityComponentTooLong {
            component: IdentityComponent::Input,
            ..
        })
    ));

    let oversized_tenant = vec![0_u8; MAX_NAMESPACE_COMPONENT_BYTES + 1];
    assert!(matches!(
        CatRequest::with_namespace(128, 128, 0, &oversized_tenant, b"style", b"id"),
        Err(CatError::IdentityComponentTooLong {
            component: IdentityComponent::Tenant,
            ..
        })
    ));
}

#[test]
fn maximum_request_reports_bounded_work_without_rendering() -> Result<(), CatError> {
    let prepared = CatRequest::new(MAX_DIMENSION, MAX_DIMENSION, 0, b"max-budget")?.prepare()?;
    let report = prepared.scene_report();
    assert_eq!(report.rgba_bytes(), 2_048_usize * 2_048 * 4);
    assert!(report.estimated_pixel_tests() <= 512 * 2_048_u64 * 2_048);
    Ok(())
}

#[test]
fn namespace_and_seed_change_named_traits() -> Result<(), CatError> {
    let first =
        CatRequest::with_namespace(128, 128, 0, b"tenant-a", b"style-a", b"id")?.prepare()?;
    let second =
        CatRequest::with_namespace(128, 128, 1, b"tenant-b", b"style-a", b"id")?.prepare()?;
    assert_ne!(first.trait_vector(), second.trait_vector());
    Ok(())
}

#[test]
fn canonical_cat_has_a_stable_pixel_fingerprint() -> Result<(), CatError> {
    let prepared = CatRequest::new(96, 96, 17, b"alpha-one-golden")?.prepare()?;
    let traits = prepared.trait_vector();
    assert_eq!(
        (
            traits.head_width(),
            traits.head_height(),
            traits.head_drop(),
            traits.ear_width(),
            traits.ear_height(),
            traits.eye_spacing(),
            traits.eye_size(),
            traits.background_hue(),
            traits.accent_hue(),
            traits.fur_hue(),
            traits.eye_hue(),
            traits.muzzle_hue(),
        ),
        (
            61_339, 51_427, 40_871, 53_342, 55_821, 37_946, 60_747, 18_303, 53_576, 41_066, 44_018,
            32_922,
        )
    );
    let image = prepared.render_rgba()?;
    assert_eq!(fnv1a64(image.pixels()), 13_260_193_517_231_355_930);
    assert_eq!(
        format!("{:?}", image.pixel_digest()?),
        "PixelDigest(e9e9a8cc0d10e23abb5d752866239366c8aaf3804da861aa213d13086e09361b19008d907192c730263b27ddab9d4848b8a2aa2a4ccc80eb71badba196e61ef8)"
    );
    Ok(())
}

#[test]
fn caller_surface_matches_owned_output_and_preserves_padding() -> Result<(), CatError> {
    let prepared = CatRequest::new(64, 64, 5, b"surface")?.prepare()?;
    let owned = prepared.render_rgba()?;
    let stride = 64 * 4 + 7;
    let mut storage = vec![0xa5_u8; stride * 64];
    let digest;
    {
        let mut surface = RgbaSurfaceMut::new(&mut storage, 64, 64, stride)?;
        prepared.render_into(&mut surface)?;
        digest = surface.pixel_digest()?;
    }
    for (row, expected) in owned.pixels().chunks_exact(64 * 4).enumerate() {
        let start = row * stride;
        assert_eq!(storage.get(start..start + 64 * 4), Some(expected));
        assert_eq!(
            storage.get(start + 64 * 4..start + stride),
            Some(&[0xa5; 7][..])
        );
    }
    assert_eq!(digest, owned.pixel_digest()?);
    Ok(())
}

#[test]
fn mismatched_surface_fails_before_modification() -> Result<(), CatError> {
    let prepared = CatRequest::new(64, 64, 0, b"mismatch")?.prepare()?;
    let mut storage = vec![0x5a_u8; 65 * 64 * 4];
    {
        let mut surface = RgbaSurfaceMut::new(&mut storage, 65, 64, 65 * 4)?;
        assert_eq!(
            prepared.render_into(&mut surface),
            Err(CatError::InvalidSurface)
        );
    }
    assert!(storage.iter().all(|byte| *byte == 0x5a));
    Ok(())
}

#[test]
fn svg_fragment_uses_the_caller_prefix() -> Result<(), CatError> {
    let prepared = CatRequest::new(64, 64, 0, b"fragment")?.prepare()?;
    let first = prepared.render_svg_with(SvgOptions::fragment("avatar-a")?)?;
    let second = prepared.render_svg_with(SvgOptions::fragment("avatar-b")?)?;
    assert!(first.contains("avatar-a-fill-0"));
    assert!(!first.contains("avatar-b-"));
    assert_ne!(first, second);
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn distinct_colors(bytes: &[u8]) -> usize {
    let mut colors = [[0_u8; 4]; 16];
    let mut count = 0_usize;
    for pixel in bytes.chunks_exact(4) {
        let Ok(color) = <[u8; 4]>::try_from(pixel) else {
            continue;
        };
        if !colors.get(..count).unwrap_or_default().contains(&color)
            && let Some(slot) = colors.get_mut(count)
        {
            *slot = color;
            count = count.saturating_add(1);
        }
    }
    count
}
