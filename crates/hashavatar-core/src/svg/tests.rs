use core::fmt::Write;

use super::*;
use crate::CatRequest;

#[test]
fn document_and_fragment_are_well_formed() -> Result<(), CatError> {
    let prepared = CatRequest::new(128, 128, 0, b"svg-fixture")?.prepare()?;
    let document = prepared.render_svg_with(SvgOptions::document(
        "fixture",
        "Cat & owner",
        "A <deterministic> avatar",
    )?)?;
    assert!(roxmltree::Document::parse(&document).is_ok());
    assert!(document.contains("Cat &amp; owner"));
    assert!(document.contains("fill-opacity=\"0.1411764705882353\""));
    let fragment = prepared.render_svg_with(SvgOptions::fragment("fixture")?)?;
    assert!(roxmltree::Document::parse(&fragment).is_ok());
    assert!(fragment.starts_with("<g id=\"fixture-scene\">"));
    Ok(())
}

#[test]
fn invalid_prefix_is_rejected() {
    assert_eq!(
        SvgOptions::fragment("bad prefix"),
        Err(CatError::InvalidSvgOptions)
    );
}

#[test]
fn invalid_xml_control_character_is_rejected() {
    assert_eq!(
        SvgOptions::document("valid", "bad\u{1}", "description"),
        Err(CatError::InvalidSvgOptions)
    );
}

#[test]
fn failing_writer_reports_partial_output_contract() -> Result<(), CatError> {
    struct ShortWriter(usize);
    impl Write for ShortWriter {
        fn write_str(&mut self, value: &str) -> core::fmt::Result {
            if value.len() > self.0 {
                return Err(core::fmt::Error);
            }
            self.0 -= value.len();
            Ok(())
        }
    }
    let prepared = CatRequest::new(64, 64, 0, b"writer")?.prepare()?;
    assert_eq!(
        prepared.write_svg(&mut ShortWriter(32), SvgOptions::default()),
        Err(CatError::SvgWrite)
    );
    Ok(())
}
