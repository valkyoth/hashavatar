#![no_main]

use hashavatar::{
    AccessoryStack, AvatarAccessory, AvatarBackground, AvatarExpression, AvatarKind, AvatarRequest,
    AvatarShape, AvatarStyle, StyleResolutionPolicy,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let kind = AvatarKind::from_byte(data.first().copied().unwrap_or_default());
    let mut forward = AccessoryStack::new();
    let mut reverse = AccessoryStack::new();
    let requested: Vec<_> = data
        .get(1..5)
        .unwrap_or_default()
        .iter()
        .map(|value| AvatarAccessory::from_sample(u16::from(*value)))
        .collect();
    for accessory in requested.iter().copied() {
        if forward.try_push(accessory).is_err() {
            return;
        }
    }
    for accessory in requested.iter().rev().copied() {
        if reverse.try_push(accessory).is_err() {
            return;
        }
    }
    let expression = AvatarExpression::from_sample(u16::from(
        data.get(5).copied().unwrap_or_default(),
    ));
    let prepare = |accessories| {
        let style = AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Square)
            .with_accessories(accessories)
            .with_expression(expression)
            .with_resolution_policy(StyleResolutionPolicy::AutomaticFallback);
        AvatarRequest::new(64, 64, 0, b"layout-fuzz", style).and_then(AvatarRequest::prepare)
    };
    let Ok(forward) = prepare(forward) else {
        return;
    };
    let Ok(reverse) = prepare(reverse) else {
        return;
    };
    assert_eq!(forward.resolved_style(), reverse.resolved_style());
    assert_eq!(forward.layout_report(), reverse.layout_report());
});
