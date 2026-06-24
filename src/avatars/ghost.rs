pub fn render_ghost_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.19 + identity.unit_f32(3) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.21 + identity.unit_f32(4) * 0.08)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.52 + identity.unit_f32(5) * 0.06)) as i32,
        head_rx,
        head_ry,
    };
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(
            background,
            hsl_to_color(
                205.0 + identity.unit_f32(0) * 55.0,
                0.16 + identity.unit_f32(6) * 0.08,
                0.95,
            ),
        )
        .into(),
    );
    let body = hsl_to_color(
        190.0 + identity.unit_f32(1) * 55.0,
        0.10 + identity.unit_f32(7) * 0.10,
        0.94 + identity.unit_f32(8) * 0.04,
    );
    let shade = hsl_to_color(
        210.0 + identity.unit_f32(2) * 34.0,
        0.16 + identity.unit_f32(9) * 0.12,
        0.70 + identity.unit_f32(10) * 0.12,
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        shade,
        0.28,
        background,
        identity,
    );
    if background == AvatarBackground::Themed && identity.byte(11).is_multiple_of(2) {
        draw_filled_ellipse_mut(
            &mut image,
            (
                layout.center_x - layout.head_rx / 2,
                layout.center_y + layout.head_ry / 3,
            ),
            (layout.head_rx as f32 * 0.42) as i32,
            (layout.head_ry as f32 * 0.18) as i32,
            Color::rgba(shade.0[0], shade.0[1], shade.0[2], 80).into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        body.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(layout.center_x - layout.head_rx, layout.center_y).of_size(
            (layout.head_rx * 2) as u32,
            (layout.head_ry + layout.head_ry / 2) as u32,
        ),
        body.into(),
    );
    let scallops = 3 + (identity.byte(12) % 3) as i32;
    for index in 0..scallops {
        let denominator = (scallops - 1).max(1);
        let x = layout.center_x - layout.head_rx + index * (layout.head_rx * 2 / denominator);
        let radius = ((layout.head_rx as f32)
            * (0.18 + identity.unit_f32(13 + index as usize) * 0.12)) as i32;
        draw_filled_circle_mut(
            &mut image,
            (x, layout.center_y + layout.head_ry + layout.head_ry / 2),
            radius.max(3),
            body.into(),
        );
    }
    if identity.byte(16).is_multiple_of(2) {
        for side in [-1, 1] {
            draw_filled_ellipse_mut(
                &mut image,
                (
                    layout.center_x + side * (layout.head_rx + layout.head_rx / 5),
                    layout.center_y + layout.head_ry / 4,
                ),
                (layout.head_rx as f32 * (0.20 + identity.unit_f32(17) * 0.08)) as i32,
                (layout.head_ry as f32 * 0.16) as i32,
                Color::rgba(body.0[0], body.0[1], body.0[2], 210).into(),
            );
        }
    }
    draw_creature_eyes(
        &mut image,
        layout,
        if identity.byte(18).is_multiple_of(5) {
            3
        } else {
            2
        },
        if identity.byte(19).is_multiple_of(2) {
            CreatureEyeStyle::Tall
        } else {
            CreatureEyeStyle::Hollow
        },
        Color::rgb(42, 48, 68),
        Color::rgb(42, 48, 68),
    );
    let mouth_style = match identity.byte(20) % 3 {
        0 => CreatureMouthStyle::Smile,
        1 => CreatureMouthStyle::Fang,
        _ => CreatureMouthStyle::Flat,
    };
    draw_creature_mouth(&mut image, layout, mouth_style, shade);
    Ok(image)
}

