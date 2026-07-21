#![no_main]

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle, RgbaSurfaceMut,
};
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
    let style = AvatarStyle::new(
        AvatarKind::from_byte(data.get(3).copied().unwrap_or_default()),
        AvatarBackground::from_byte(data.get(4).copied().unwrap_or_default()),
        AvatarShape::from_byte(data.get(5).copied().unwrap_or_default()),
    );
    let identity = data.get(6..).unwrap_or_default();
    let Ok(request) = AvatarRequest::new(width, height, 0, identity, style) else {
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
