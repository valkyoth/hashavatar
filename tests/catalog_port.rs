//! Exhaustive alpha.3 catalog and capability-manifest integration coverage.

use std::collections::BTreeSet;

use hashavatar::{
    AVATAR_FAMILY_CAPABILITIES, AvatarBackground, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle, MAX_DIMENSION, MIN_DIMENSION, RgbaSurfaceMut,
};
use sanitization_crypto_interop::sha2::SanitizedSha512;

fn prepare(
    width: u32,
    height: u32,
    kind: AvatarKind,
    background: AvatarBackground,
    shape: AvatarShape,
) -> Result<hashavatar::PreparedAvatar, hashavatar::AvatarError> {
    let style = AvatarStyle::new(kind, background, shape);
    AvatarRequest::new(width, height, 7, b"catalog-port-fixture", style)
        .and_then(AvatarRequest::prepare)
}

#[test]
fn every_family_renders_distinct_raster_and_valid_svg() -> Result<(), hashavatar::AvatarError> {
    let mut digests = BTreeSet::new();
    for kind in AvatarKind::ALL {
        let prepared = prepare(96, 96, kind, AvatarBackground::Themed, AvatarShape::Square)?;
        let image = prepared.render_rgba()?;
        let digest = image.pixel_digest()?;
        assert!(
            digests.insert(*digest.as_bytes()),
            "duplicate family: {}",
            kind.as_str()
        );
        let svg = prepared.render_svg()?;
        assert!(
            roxmltree::Document::parse(&svg).is_ok(),
            "invalid SVG: {}",
            kind.as_str()
        );
        let report = prepared.scene_report();
        assert!(report.command_count() <= 64);
        assert!(report.path_count() <= 8);
    }
    assert_eq!(digests.len(), AvatarKind::ALL.len());
    Ok(())
}

#[test]
fn every_background_and_frame_combination_executes_one_scene() -> Result<(), hashavatar::AvatarError>
{
    for kind in AvatarKind::ALL {
        for background in AvatarBackground::ALL {
            for shape in AvatarShape::ALL {
                let prepared = prepare(64, 64, kind, background, shape)?;
                let image = prepared.render_rgba()?;
                let svg = prepared.render_svg()?;
                assert_eq!(image.dimensions(), (64, 64));
                assert!(roxmltree::Document::parse(&svg).is_ok());
            }
        }
    }
    Ok(())
}

#[test]
fn all_families_prepare_at_minimum_default_and_maximum_dimensions()
-> Result<(), hashavatar::AvatarError> {
    for kind in AvatarKind::ALL {
        for (width, height) in [
            (MIN_DIMENSION, MIN_DIMENSION),
            (256, 192),
            (MAX_DIMENSION, MAX_DIMENSION),
        ] {
            let prepared = prepare(
                width,
                height,
                kind,
                AvatarBackground::Themed,
                AvatarShape::Octagon,
            )?;
            assert_eq!((prepared.width(), prepared.height()), (width, height));
            assert_eq!(
                prepared.scene_report().rgba_bytes(),
                usize::try_from(width).unwrap_or_default()
                    * usize::try_from(height).unwrap_or_default()
                    * 4
            );
        }
    }
    Ok(())
}

#[test]
fn shaped_and_transparent_outputs_clear_outside_pixels() -> Result<(), hashavatar::AvatarError> {
    for shape in [
        AvatarShape::Circle,
        AvatarShape::Squircle,
        AvatarShape::Hexagon,
        AvatarShape::Octagon,
    ] {
        let prepared = prepare(
            64,
            64,
            AvatarKind::Rocket,
            AvatarBackground::Transparent,
            shape,
        )?;
        let image = prepared.render_rgba()?;
        for offset in [0, 63 * 4, 63 * 64 * 4, (64 * 64 - 1) * 4] {
            assert_eq!(
                image.pixels().get(offset..offset + 4),
                Some([0, 0, 0, 0].as_slice())
            );
        }
    }
    Ok(())
}

#[test]
fn transparent_render_clears_prior_caller_pixels() -> Result<(), hashavatar::AvatarError> {
    let prepared = prepare(
        64,
        64,
        AvatarKind::Ghost,
        AvatarBackground::Transparent,
        AvatarShape::Circle,
    )?;
    let mut storage = vec![0xa5_u8; 64 * 64 * 4];
    let mut surface = RgbaSurfaceMut::new(&mut storage, 64, 64, 64 * 4)?;
    prepared.render_into(&mut surface)?;
    assert_eq!(surface.as_bytes().get(..4), Some([0, 0, 0, 0].as_slice()));
    assert_eq!(
        surface.as_bytes().get((64 * 64 - 1) * 4..),
        Some([0, 0, 0, 0].as_slice())
    );
    Ok(())
}

#[test]
fn non_square_svg_uses_a_real_clip_path() -> Result<(), hashavatar::AvatarError> {
    for shape in [
        AvatarShape::Circle,
        AvatarShape::Squircle,
        AvatarShape::Hexagon,
        AvatarShape::Octagon,
    ] {
        let svg =
            prepare(64, 64, AvatarKind::Planet, AvatarBackground::Ocean, shape)?.render_svg()?;
        assert!(svg.contains("<clipPath"));
        assert!(svg.contains("clip-path=\"url(#hashavatar-clip-"));
        assert!(roxmltree::Document::parse(&svg).is_ok());
    }
    Ok(())
}

#[test]
fn capability_manifest_is_complete_and_ordered() {
    assert_eq!(AVATAR_FAMILY_CAPABILITIES.len(), AvatarKind::ALL.len());
    for (index, (expected, entry)) in AvatarKind::ALL
        .iter()
        .zip(AVATAR_FAMILY_CAPABILITIES)
        .enumerate()
    {
        assert_eq!(*expected, entry.kind());
        assert_eq!(
            expected.catalog_id(),
            u16::try_from(index).unwrap_or_default()
        );
        assert!(entry.capabilities().supports_backgrounds());
        assert!(entry.capabilities().supports_shapes());
    }
    for (index, background) in AvatarBackground::ALL.iter().enumerate() {
        assert_eq!(
            background.catalog_id(),
            u16::try_from(index).unwrap_or_default()
        );
    }
    for (index, shape) in AvatarShape::ALL.iter().enumerate() {
        assert_eq!(shape.catalog_id(), u16::try_from(index).unwrap_or_default());
    }
    assert!(!AvatarKind::Planet.capabilities().has_face_anchors());
    assert!(AvatarKind::Cat.capabilities().has_face_anchors());
}

#[test]
fn complete_catalog_has_a_stable_aggregate_pixel_fingerprint() -> Result<(), hashavatar::AvatarError>
{
    let mut aggregate = SanitizedSha512::new();
    for kind in AvatarKind::ALL {
        let prepared = prepare(64, 64, kind, AvatarBackground::Themed, AvatarShape::Square)?;
        let digest = prepared.render_rgba()?.pixel_digest()?;
        aggregate.update(digest.as_bytes());
    }
    assert_eq!(
        aggregate.finalize(),
        [
            238, 123, 213, 179, 34, 38, 186, 2, 201, 167, 182, 249, 125, 247, 61, 89, 151, 249,
            185, 117, 214, 111, 120, 193, 77, 172, 16, 34, 80, 162, 248, 76, 84, 248, 202, 154,
            238, 72, 120, 214, 201, 190, 40, 105, 120, 49, 11, 167, 181, 9, 83, 126, 248, 117, 108,
            213, 161, 220, 193, 218, 99, 194, 93, 128,
        ]
    );
    Ok(())
}
