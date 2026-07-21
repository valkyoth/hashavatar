use std::io::{self, Write};

use hashavatar::prelude::*;

fn identity() -> AvatarIdentity {
    AvatarIdentity::new_with_namespace(
        AvatarNamespace::new("prepared-api-tests", "v1").expect("valid namespace"),
        b"person@example.test",
    )
    .expect("valid identity")
}

fn explicit_style() -> AvatarStyleOptions {
    AvatarStyleOptions::new(
        AvatarKind::Cat,
        AvatarBackground::Ocean,
        AvatarAccessory::Glasses,
        AvatarColor::NeonMint,
        AvatarExpression::Happy,
        AvatarShape::Squircle,
    )
}

fn prepared() -> PreparedAvatar {
    AvatarRequest::builder(identity())
        .size(96, 80)
        .seed(17)
        .style(explicit_style())
        .prepare()
        .expect("valid request")
}

#[test]
fn strict_requests_reject_unsupported_layers() {
    let error = AvatarRequest::builder(identity())
        .kind(AvatarKind::Paws)
        .accessory(AvatarAccessory::Eyepatch)
        .prepare()
        .expect_err("strict requests must reject ignored layers");

    assert!(matches!(error, AvatarRequestError::Style(_)));
}

#[test]
fn legacy_requests_report_and_apply_family_fallbacks() {
    let requested = AvatarStyleOptions::new(
        AvatarKind::Paws,
        AvatarBackground::Starry,
        AvatarAccessory::Crown,
        AvatarColor::Gold,
        AvatarExpression::Winking,
        AvatarShape::Circle,
    );
    let prepared = AvatarRequest::builder(identity())
        .size(80, 72)
        .style(requested)
        .legacy_v1_compatibility()
        .prepare()
        .expect("legacy fallback request");
    let resolved = prepared.resolved_style();

    assert_eq!(resolved.requested(), requested);
    assert_eq!(resolved.effective(), requested.canonicalized_for_family());
    assert!(resolved.applied_legacy_fallbacks());
    assert!(resolved.ignored_accessory());
    assert!(resolved.ignored_expression());
    assert!(
        !prepared
            .layout_report()
            .family_capabilities()
            .supports_accessories()
    );
}

#[test]
fn legacy_builder_prepare_preserves_pixels_svg_and_keys() {
    let old_image = AvatarBuilder::for_id(b"legacy-builder-parity")
        .namespace("prepared-api-tests", "v1")
        .size(96, 80)
        .seed(23)
        .style(explicit_style())
        .render()
        .expect("legacy image");
    let old_svg = AvatarBuilder::for_id(b"legacy-builder-parity")
        .namespace("prepared-api-tests", "v1")
        .size(96, 80)
        .seed(23)
        .style(explicit_style())
        .render_svg()
        .expect("legacy SVG");
    let old_key = AvatarBuilder::for_id(b"legacy-builder-parity")
        .namespace("prepared-api-tests", "v1")
        .size(96, 80)
        .seed(23)
        .style(explicit_style())
        .avatar_asset_key()
        .expect("legacy key");
    let prepared = AvatarBuilder::for_id(b"legacy-builder-parity")
        .namespace("prepared-api-tests", "v1")
        .size(96, 80)
        .seed(23)
        .style(explicit_style())
        .prepare()
        .expect("prepared legacy builder");

    assert_eq!(prepared.render().expect("prepared image"), old_image);
    assert_eq!(prepared.render_svg(), old_svg);
    assert_eq!(prepared.avatar_asset_key(), old_key);
}

#[test]
fn automatic_requests_expose_resolution_without_retaining_raw_input() {
    let identity_key = identity().identity_cache_key();
    let request =
        AvatarRequest::automatic(identity(), AvatarSpec::new(72, 72, 9).expect("valid spec"));
    let request_debug = format!("{request:?}");
    let prepared = request.prepare().expect("automatic request");
    let prepared_debug = format!("{prepared:?}");

    assert!(prepared.resolved_style().is_automatically_derived());
    assert_eq!(prepared.identity_cache_key(), identity_key);
    assert!(request_debug.contains("[REDACTED]"));
    assert!(prepared_debug.contains("[REDACTED]"));
    assert!(!request_debug.contains("person@example.test"));
    assert!(!prepared_debug.contains(&identity_key.to_hex()));
}

#[test]
fn resource_budget_is_explicit_and_conservative() {
    let prepared = prepared();
    let budget = prepared.resource_budget();

    assert_eq!(budget.spec(), prepared.spec());
    assert_eq!(budget.minimum_rgba8_stride(), 96 * 4);
    assert_eq!(budget.minimum_rgba8_surface_bytes(), 96 * 80 * 4);
    assert_eq!(budget.render_into_temporary_bytes(), 96 * 80 * 4);
    assert_eq!(
        budget.minimum_render_into_known_rgba_bytes(),
        96 * 80 * 4 * 2
    );
    assert_eq!(budget.encode_vec_known_base_bytes(), Some(96 * 80 * 4 * 2));
    assert_eq!(budget.encode_writer_known_base_bytes(), 96 * 80 * 4);

    let stride = 96 * 4 + 17;
    let mut pixels = vec![0_u8; stride * 80 + 29];
    let surface = RasterSurfaceMut::new_rgba8(&mut pixels, 96, 80, stride).expect("padded surface");
    assert_eq!(surface.required_len(), stride * 80);
    assert_eq!(surface.provided_len(), stride * 80 + 29);
    assert_eq!(
        budget
            .render_into_known_rgba_bytes_for(&surface)
            .expect("matching surface"),
        stride * 80 + 96 * 80 * 4
    );
}

#[test]
fn builders_preserve_seed_across_invalid_intermediate_dimensions() {
    let expected = AvatarSpec::new(64, 64, 0xfeed_beef).expect("valid expected spec");
    let request = AvatarRequest::builder(identity())
        .seed(0xfeed_beef)
        .size(0, 0)
        .size(64, 64)
        .build()
        .expect("recovered request");
    assert_eq!(request.spec(), expected);

    let prepared = AvatarBuilder::for_id(b"seed-preservation")
        .seed(0xfeed_beef)
        .size(0, 0)
        .size(64, 64)
        .prepare()
        .expect("recovered legacy builder");
    assert_eq!(prepared.spec(), expected);
}

#[test]
fn render_into_supports_tight_and_padded_surfaces() {
    let prepared = prepared();
    let expected = prepared.render().expect("owned image");
    let row_bytes = 96 * 4;

    let mut tight_pixels = vec![0_u8; row_bytes * 80];
    let mut tight =
        RasterSurfaceMut::new_rgba8(&mut tight_pixels, 96, 80, row_bytes).expect("tight surface");
    prepared.render_into(&mut tight).expect("tight render");
    assert_eq!(tight.pixels(), expected.as_raw());

    let stride = row_bytes + 13;
    let mut padded_pixels = vec![0xa5_u8; stride * 80];
    let mut padded =
        RasterSurfaceMut::new_rgba8(&mut padded_pixels, 96, 80, stride).expect("padded surface");
    prepared.render_into(&mut padded).expect("padded render");
    for (expected_row, actual_row) in expected
        .as_raw()
        .chunks_exact(row_bytes)
        .zip(padded.pixels().chunks_exact(stride))
    {
        assert_eq!(&actual_row[..row_bytes], expected_row);
        assert!(actual_row[row_bytes..].iter().all(|byte| *byte == 0xa5));
    }
}

#[test]
fn raster_surfaces_fail_closed_on_invalid_layouts() {
    let mut bytes = vec![0_u8; 96 * 80 * 4];
    assert!(matches!(
        RasterSurfaceMut::new_rgba8(&mut bytes, 0, 80, 96 * 4),
        Err(RasterSurfaceError::ZeroDimension { .. })
    ));
    assert!(matches!(
        RasterSurfaceMut::new_rgba8(&mut bytes, 96, 80, 96 * 4 - 1),
        Err(RasterSurfaceError::StrideTooSmall { .. })
    ));
    let mut short = vec![0_u8; 96 * 80 * 4 - 1];
    assert!(matches!(
        RasterSurfaceMut::new_rgba8(&mut short, 96, 80, 96 * 4),
        Err(RasterSurfaceError::BufferTooSmall { .. })
    ));

    let prepared = prepared();
    let mut wrong_size = vec![0_u8; 95 * 80 * 4];
    let mut surface = RasterSurfaceMut::new_rgba8(&mut wrong_size, 95, 80, 95 * 4)
        .expect("valid but mismatched surface");
    assert!(matches!(
        prepared.render_into(&mut surface),
        Err(RasterSurfaceError::DimensionMismatch { .. })
    ));
}

#[derive(Default)]
struct ShortWriter {
    bytes: Vec<u8>,
    maximum_write: usize,
}

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let length = bytes.len().min(self.maximum_write.max(1));
        self.bytes.extend_from_slice(&bytes[..length]);
        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FailingWriter {
    bytes: Vec<u8>,
    remaining: usize,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("injected failure"));
        }
        let length = bytes.len().min(self.remaining);
        self.bytes.extend_from_slice(&bytes[..length]);
        self.remaining -= length;
        Ok(length)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn svg_writer_handles_short_writes_and_preserves_partial_output_on_error() {
    let prepared = prepared();
    let expected = prepared.render_svg();
    let mut short = ShortWriter {
        bytes: Vec::new(),
        maximum_write: 7,
    };
    prepared.write_svg(&mut short).expect("short writes");
    assert_eq!(short.bytes, expected.as_bytes());

    let mut failing = FailingWriter {
        bytes: Vec::new(),
        remaining: 31,
    };
    let error = prepared
        .write_svg(&mut failing)
        .expect_err("injected writer failure");
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(failing.bytes, expected.as_bytes()[..31]);
}

#[test]
fn encoded_writer_matches_vec_api_and_propagates_failures() {
    let prepared = prepared();
    let expected = prepared
        .encode(AvatarOutputFormat::WebP)
        .expect("WebP bytes");
    let mut writer = Vec::new();
    prepared
        .encode_to_writer(AvatarOutputFormat::WebP, &mut writer)
        .expect("WebP writer");
    assert_eq!(writer, expected);

    let mut failing = FailingWriter {
        bytes: Vec::new(),
        remaining: 19,
    };
    assert!(
        prepared
            .encode_to_writer(AvatarOutputFormat::WebP, &mut failing)
            .is_err()
    );
    assert!(!failing.bytes.is_empty());
}
