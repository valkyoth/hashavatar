#![no_main]

use hashavatar::{
    AvatarAccessory, AvatarBackground, AvatarColor, AvatarExpression, AvatarKind,
    AvatarOutputFormat, AvatarShape, AvatarSpec, AvatarStyleOptions, encode_avatar_style_for_id,
    render_avatar_svg_style_for_id,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let kind = AvatarKind::from_byte(data[0]);
    let background = AvatarBackground::from_byte(*data.get(1).unwrap_or(&0));
    let accessory = AvatarAccessory::from_byte(*data.get(3).unwrap_or(&0));
    let color = AvatarColor::from_byte(*data.get(4).unwrap_or(&0));
    let expression = AvatarExpression::from_byte(*data.get(5).unwrap_or(&0));
    let shape = AvatarShape::from_byte(*data.get(6).unwrap_or(&0));
    let size = 64 + u32::from(*data.get(2).unwrap_or(&0) % 8) * 64;
    let identity_len = data.len().min(128);
    let identity = &data[..identity_len];
    let Ok(spec) = AvatarSpec::new(size, size, 0) else {
        return;
    };
    let style = AvatarStyleOptions::new(kind, background, accessory, color, expression, shape);

    let _ = render_avatar_svg_style_for_id(spec, identity, style);
    let _ = encode_avatar_style_for_id(spec, identity, AvatarOutputFormat::Png, style);
});
