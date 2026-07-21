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
    let aggregate = aggregate.finalize();
    let expected_family_fingerprints = [
        16_213_333_167_405_360_225,
        5_753_094_311_568_932_757,
        7_779_670_557_938_237_177,
        12_565_801_981_545_221_942,
        13_397_859_520_559_166_265,
        16_933_512_960_078_939_178,
        2_605_228_308_326_819_384,
        11_216_341_629_529_196_612,
        1_334_843_491_332_943_135,
        5_786_818_311_251_665_190,
        3_167_892_409_447_852_483,
        4_568_691_605_609_754_493,
        5_970_636_059_914_220_005,
        11_874_149_948_522_176_569,
        7_918_069_477_284_194_231,
        9_275_310_335_073_050_193,
        16_833_294_924_013_900_125,
        7_400_073_407_904_865_353,
        8_853_086_671_596_485_388,
        1_682_025_975_739_740_998,
        5_399_080_176_009_215_906,
        16_540_821_786_562_566_486,
        17_639_134_060_814_003_555,
        6_549_211_148_677_610_237,
        9_586_995_340_015_360_875,
        13_235_850_235_817_774_647,
        10_184_591_101_052_580_705,
        7_423_227_272_348_555_885,
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
        aggregate,
        [
            219, 252, 51, 33, 187, 198, 91, 72, 169, 31, 216, 41, 187, 144, 68, 250, 40, 118, 80,
            234, 77, 99, 40, 42, 232, 69, 215, 147, 202, 244, 84, 212, 18, 35, 168, 123, 148, 12,
            104, 203, 223, 26, 142, 78, 83, 205, 142, 49, 154, 14, 34, 231, 199, 177, 26, 123, 131,
            165, 91, 57, 103, 204, 12, 95,
        ]
    );
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
