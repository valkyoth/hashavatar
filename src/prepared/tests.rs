use std::cell::Cell;

use super::*;

#[test]
fn dimension_mismatch_is_rejected_before_rendering() {
    let identity = AvatarIdentity::new(b"prepared-preflight").expect("bounded identity");
    let prepared = AvatarRequest::builder(identity)
        .size(64, 64)
        .prepare()
        .expect("valid request");
    let mut pixels = vec![0xa5; 65 * 64 * 4];
    let mut surface =
        RasterSurfaceMut::new_rgba8(&mut pixels, 65, 64, 65 * 4).expect("valid surface");
    let renderer_called = Cell::new(false);

    let result = prepared.render_into_with(&mut surface, || {
        renderer_called.set(true);
        Ok(RgbaImage::new(64, 64))
    });

    assert!(matches!(
        result,
        Err(RasterSurfaceError::DimensionMismatch { .. })
    ));
    assert!(!renderer_called.get());
    assert!(surface.pixels().iter().all(|byte| *byte == 0xa5));
}
