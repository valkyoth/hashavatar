#![no_main]

use hashavatar::CatRequest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some(width_bytes) = data.get(0..2) else {
        return;
    };
    let Some(height_bytes) = data.get(2..4) else {
        return;
    };
    let Some(seed_bytes) = data.get(4..12) else {
        return;
    };
    let Ok(width_array) = <[u8; 2]>::try_from(width_bytes) else {
        return;
    };
    let Ok(height_array) = <[u8; 2]>::try_from(height_bytes) else {
        return;
    };
    let Ok(seed_array) = <[u8; 8]>::try_from(seed_bytes) else {
        return;
    };

    let width = u32::from(u16::from_le_bytes(width_array));
    let height = u32::from(u16::from_le_bytes(height_array));
    let style_seed = u64::from_le_bytes(seed_array);
    let tenant_end = data.len().min(28);
    let style_end = data.len().min(44);
    let identity_end = data.len().min(1_068);
    let tenant = data.get(12..tenant_end).unwrap_or_default();
    let style = data.get(tenant_end..style_end).unwrap_or_default();
    let identity = data.get(style_end..identity_end).unwrap_or_default();

    let Ok(request) =
        CatRequest::with_namespace(width, height, style_seed, tenant, style, identity)
    else {
        return;
    };
    let Ok(prepared) = request.prepare() else {
        return;
    };
    let report = prepared.scene_report();
    if let Ok(image) = prepared.render_rgba() {
        assert_eq!(image.pixels().len(), report.rgba_bytes());
    }
    if let Ok(svg) = prepared.render_svg() {
        assert!(roxmltree::Document::parse(&svg).is_ok());
    }
});
