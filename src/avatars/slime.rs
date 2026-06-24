use super::*;

pub fn render_slime_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.20 + identity.unit_f32(6) * 0.10)) as i32;
    let head_ry = (height as f32 * (0.16 + identity.unit_f32(7) * 0.08)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.56 + identity.unit_f32(8) * 0.08)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(110.0 + identity.unit_f32(3) * 80.0, 0.18, 0.93);
    let slime = hsl_to_color(
        70.0 + identity.unit_f32(4) * 130.0,
        0.44 + identity.unit_f32(9) * 0.22,
        0.46 + identity.unit_f32(10) * 0.18,
    );
    let dark = hsl_to_color(
        95.0 + identity.unit_f32(5) * 80.0,
        0.34 + identity.unit_f32(11) * 0.18,
        0.25 + identity.unit_f32(12) * 0.14,
    );
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        dark,
        0.32,
        background,
        identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        slime.into(),
    );
    let drip_count = 2 + (identity.byte(13) % 4) as i32;
    for index in 0..drip_count {
        let spacing = (layout.head_rx * 2 / drip_count.max(1)).max(1);
        let drip_w =
            (layout.head_rx as f32 * (0.18 + identity.unit_f32(14 + index as usize) * 0.12)) as i32;
        let drip_x = layout.center_x - layout.head_rx
            + index * spacing
            + (identity.byte(18 + index as usize) as i32 % spacing.max(1) / 3);
        let drip_h =
            (layout.head_ry as f32 * (0.35 + identity.unit_f32(22 + index as usize) * 0.55)) as i32;
        draw_filled_rect_mut(
            &mut image,
            Rect::at(drip_x, layout.center_y).of_size(drip_w.max(2) as u32, drip_h.max(2) as u32),
            slime.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (drip_x + drip_w / 2, layout.center_y + drip_h),
            layout.head_rx / 7,
            slime.into(),
        );
    }
    let bubble_count = 3 + (identity.byte(27) % 5) as i32;
    for bubble in 0..bubble_count {
        let bx = layout.center_x - layout.head_rx
            + (identity.byte(28 + bubble as usize) as i32 % (layout.head_rx * 2).max(1));
        let by = layout.center_y - layout.head_ry / 2
            + (identity.byte(35 + bubble as usize) as i32 % layout.head_ry.max(1));
        draw_filled_circle_mut(
            &mut image,
            (bx, by),
            ((layout.head_rx as f32) * (0.05 + identity.unit_f32(42 + bubble as usize) * 0.07))
                as i32,
            Color::rgba(255, 255, 255, 90).into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        1 + (identity.byte(49) % 3) as usize,
        if identity.byte(50).is_multiple_of(2) {
            CreatureEyeStyle::Round
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(248, 255, 236),
        Color::rgb(32, 48, 24),
    );
    let mouth_style = match identity.byte(51) % 3 {
        0 => CreatureMouthStyle::Flat,
        1 => CreatureMouthStyle::Smile,
        _ => CreatureMouthStyle::Fang,
    };
    draw_creature_mouth(&mut image, layout, mouth_style, dark);
    Ok(image)
}
