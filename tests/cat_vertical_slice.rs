//! Public integration tests for the alpha.1 Cat vertical slice.

use hashavatar::{
    CatError, CatRequest, IdentityComponent, MAX_DIMENSION, MAX_IDENTITY_BYTES,
    MAX_NAMESPACE_COMPONENT_BYTES, MIN_DIMENSION,
};

#[test]
fn one_request_produces_pixels_and_svg_from_one_scene() -> Result<(), CatError> {
    let prepared = CatRequest::new(192, 160, 42, b"alpha-one-integration")?.prepare()?;
    let report = prepared.scene_report();
    let image = prepared.render_rgba()?;
    let svg = prepared.render_svg()?;

    assert_eq!(image.dimensions(), (192, 160));
    assert_eq!(image.pixels().len(), report.rgba_bytes());
    assert_eq!(report.command_count(), 13);
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
    assert!(report.estimated_pixel_tests() <= 16 * 2_048_u64 * 2_048);
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
            34_835, 49_045, 65_322, 59_223, 8_706, 28_857, 11_047, 9_099, 31_264, 64_466, 39_997,
            59_018,
        )
    );
    let image = prepared.render_rgba()?;
    assert_eq!(fnv1a64(image.pixels()), 6_469_645_031_112_886_906);
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
