#![no_main]

use hashavatar::{CatRequest, RgbaSurfaceMut};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let width = 64 + u32::from(data.first().copied().unwrap_or(0)) % 193;
    let height = 64 + u32::from(data.get(1).copied().unwrap_or(0)) % 193;
    let padding = usize::from(data.get(2).copied().unwrap_or(0)) % 32;
    let Ok(visible) = usize::try_from(width).map(|value| value.saturating_mul(4)) else {
        return;
    };
    let stride = visible.saturating_add(padding);
    let Ok(rows) = usize::try_from(height) else {
        return;
    };
    let mut storage = vec![0xa5_u8; stride.saturating_mul(rows)];
    let identity = data.get(3..).unwrap_or_default();
    let Ok(request) = CatRequest::new(width, height, 0, identity) else {
        return;
    };
    let Ok(prepared) = request.prepare() else {
        return;
    };
    let Ok(mut surface) = RgbaSurfaceMut::new(&mut storage, width, height, stride) else {
        return;
    };
    assert!(prepared.render_into(&mut surface).is_ok());
    assert!(surface.pixel_digest().is_ok());
    for row in surface.as_bytes().chunks_exact(stride) {
        assert!(row
            .get(visible..)
            .unwrap_or_default()
            .iter()
            .all(|byte| *byte == 0xa5));
    }
});
