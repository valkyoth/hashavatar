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
    let mut family_fingerprints = Vec::with_capacity(AvatarKind::ALL.len());
    for kind in AvatarKind::ALL {
        let prepared = prepare(64, 64, kind, AvatarBackground::Themed, AvatarShape::Square)?;
        let image = prepared.render_rgba()?;
        family_fingerprints.push(fnv1a64(image.pixels()));
        let digest = image.pixel_digest()?;
        aggregate.update(digest.as_bytes());
    }
    let expected_family_fingerprints = [
        14_959_214_206_026_651_746,
        9_202_591_113_189_904_873,
        7_779_670_557_938_237_177,
        8_552_577_103_144_023_970,
        15_689_753_943_708_702_665,
        14_625_612_522_906_726_477,
        1_712_830_299_425_462_904,
        18_156_709_895_062_844_882,
        11_055_067_729_218_570_545,
        15_791_166_020_840_224_950,
        3_167_892_409_447_852_483,
        4_568_691_605_609_754_493,
        5_970_636_059_914_220_005,
        11_874_149_948_522_176_569,
        7_918_069_477_284_194_231,
        9_275_310_335_073_050_193,
        17_892_788_220_383_271_177,
        7_400_073_407_904_865_353,
        8_853_086_671_596_485_388,
        1_682_025_975_739_740_998,
        5_399_080_176_009_215_906,
        340_207_536_650_252_233,
        17_639_134_060_814_003_555,
        6_113_982_546_575_385_341,
        9_586_995_340_015_360_875,
        3_655_959_163_641_752_247,
        4_672_810_096_955_459_917,
        40_325_208_071_989_337,
        16_056_988_084_459_822_721,
        15_077_432_928_493_723_768,
        16_278_937_666_865_071_925,
    ];
    for ((kind, actual), expected) in AvatarKind::ALL
        .iter()
        .zip(family_fingerprints)
        .zip(expected_family_fingerprints)
    {
        assert_eq!(actual, expected, "visual fingerprint: {}", kind.as_str());
    }
    assert_eq!(
        aggregate.finalize(),
        [
            132, 132, 130, 154, 148, 230, 0, 63, 47, 40, 28, 205, 50, 136, 18, 251, 192, 53, 115,
            102, 15, 253, 108, 163, 220, 16, 40, 112, 158, 64, 43, 233, 145, 215, 101, 125, 149,
            61, 204, 110, 103, 139, 59, 195, 123, 93, 245, 40, 165, 53, 146, 84, 5, 96, 219, 181,
            218, 251, 232, 66, 215, 238, 241, 48,
        ]
    );
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
