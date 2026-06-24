/// Render a cactus avatar from a stable identity.
pub fn render_cactus_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let bg = hsl_to_color(80.0 + identity.unit_f32(0) * 55.0, 0.20, 0.92);
    let cactus = hsl_to_color(105.0 + identity.unit_f32(1) * 60.0, 0.42, 0.42);
    let shadow = hsl_to_color(105.0 + identity.unit_f32(2) * 60.0, 0.38, 0.30);
    let flower = hsl_to_color(320.0 + identity.unit_f32(3) * 55.0, 0.58, 0.64);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let body_w = (width as f32 * (0.13 + identity.unit_f32(4) * 0.04)) as i32;
    let body_h = (height as f32 * (0.36 + identity.unit_f32(5) * 0.10)) as i32;
    let top_y = center_y - body_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        body_w * 2,
        body_h / 2,
        shadow,
        0.20,
        background,
        identity,
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - body_w / 2, top_y).of_size(body_w as u32, body_h as u32),
        cactus.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y),
        body_w / 2,
        body_w / 2,
        cactus.into(),
    );
    for side in [-1, 1] {
        if side == 1 || identity.byte(6).is_multiple_of(2) {
            let arm_y =
                center_y - body_h / 5 + side * (identity.byte(7) as i32 % (body_h / 6).max(1));
            let arm_len = (width as f32
                * (0.11 + identity.unit_f32(8 + side.unsigned_abs() as usize) * 0.05))
                as i32;
            let arm_x = if side < 0 {
                center_x - body_w / 3 - arm_len
            } else {
                center_x + body_w / 3
            };
            let cap_x = if side < 0 {
                center_x - body_w / 3 - arm_len
            } else {
                center_x + body_w / 3 + arm_len
            };
            draw_filled_rect_mut(
                &mut image,
                Rect::at(arm_x, arm_y - body_w / 4)
                    .of_size(arm_len.max(2) as u32, (body_w / 2).max(2) as u32),
                cactus.into(),
            );
            draw_filled_ellipse_mut(
                &mut image,
                (cap_x, arm_y),
                body_w / 4,
                body_w / 4,
                cactus.into(),
            );
        }
    }
    for needle in 0..5 {
        let y = top_y + body_h / 5 + needle * body_h / 7;
        draw_line_segment_mut(
            &mut image,
            ((center_x - body_w / 8) as f32, y as f32),
            ((center_x - body_w / 4) as f32, (y - body_w / 8) as f32),
            Color::rgba(242, 255, 224, 180).into(),
        );
        draw_line_segment_mut(
            &mut image,
            ((center_x + body_w / 8) as f32, (y + body_w / 12) as f32),
            ((center_x + body_w / 4) as f32, y as f32),
            Color::rgba(242, 255, 224, 180).into(),
        );
    }
    if !identity.byte(12).is_multiple_of(3) {
        draw_filled_circle_mut(
            &mut image,
            (center_x, top_y - body_w / 2),
            body_w / 4,
            flower.into(),
        );
    }
    Ok(image)
}

