#![no_main]

use hashavatar::{
    AccessoryStack, AvatarAccessory, AvatarBackground, AvatarExpression, AvatarKind, AvatarPalette,
    AvatarRequest, AvatarShape, AvatarStyle, StyleResolutionPolicy,
};
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
    let mut accessories = AccessoryStack::new();
    for value in data.get(18..22).unwrap_or_default() {
        if accessories
            .try_push(AvatarAccessory::from_sample(u16::from(*value)))
            .is_err()
        {
            return;
        }
    }
    let avatar_style = AvatarStyle::new(
        AvatarKind::from_byte(data.get(12).copied().unwrap_or_default()),
        AvatarBackground::from_byte(data.get(13).copied().unwrap_or_default()),
        AvatarShape::from_byte(data.get(14).copied().unwrap_or_default()),
    )
    .with_palette(AvatarPalette::from_sample(u16::from(
        data.get(15).copied().unwrap_or_default(),
    )))
    .with_expression(AvatarExpression::from_sample(u16::from(
        data.get(16).copied().unwrap_or_default(),
    )))
    .with_accessories(accessories)
    .with_resolution_policy(if data.get(17).copied().unwrap_or_default() & 1 == 0 {
        StyleResolutionPolicy::Strict
    } else {
        StyleResolutionPolicy::AutomaticFallback
    });
    let tenant_end = data.len().min(31);
    let style_end = data.len().min(47);
    let identity_end = data.len().min(1_071);
    let tenant = data.get(22..tenant_end).unwrap_or_default();
    let style = data.get(tenant_end..style_end).unwrap_or_default();
    let identity = data.get(style_end..identity_end).unwrap_or_default();

    let Ok(request) = AvatarRequest::with_namespace(
        width,
        height,
        style_seed,
        tenant,
        style,
        identity,
        avatar_style,
    )
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
