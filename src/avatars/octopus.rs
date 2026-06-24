use super::*;

/// Render an octopus avatar from a stable identity.
pub fn render_octopus_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.54 + identity.unit_f32(5) * 0.05)) as i32;
    let bg = hsl_to_color(185.0 + identity.unit_f32(0) * 70.0, 0.22, 0.92);
    let body = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.34, 0.38);
    let eye = Color::rgb(28, 26, 38);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.21 + identity.unit_f32(3) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.20 + identity.unit_f32(4) * 0.06)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, shade, 0.22, background, identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        body.into(),
    );
    let tentacles = 4 + (identity.byte(6) % 3) as i32;
    for index in 0..tentacles {
        let denominator = (tentacles - 1).max(1);
        let x = center_x - head_rx + index * (head_rx * 2 / denominator);
        let length =
            (head_ry as f32 * (0.42 + identity.unit_f32(7 + index as usize) * 0.35)) as i32;
        draw_filled_rect_mut(
            &mut image,
            Rect::at(x - head_rx / 12, center_y + head_ry / 2)
                .of_size((head_rx / 6).max(2) as u32, length.max(2) as u32),
            body.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (x, center_y + head_ry / 2 + length),
            (head_rx / 10).max(3),
            body.into(),
        );
    }
    for side in [-1, 1] {
        let ex = center_x + side * head_rx / 3;
        let ey = center_y - head_ry / 6;
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            head_rx / 9,
            Color::rgb(255, 255, 248).into(),
        );
        draw_filled_circle_mut(&mut image, (ex, ey), head_rx / 20, eye.into());
    }
    let mouth = match identity.byte(14) % 3 {
        0 => CreatureMouthStyle::Smile,
        1 => CreatureMouthStyle::Flat,
        _ => CreatureMouthStyle::Fang,
    };
    draw_creature_mouth(
        &mut image,
        FaceLayout {
            center_x,
            center_y,
            head_rx,
            head_ry,
        },
        mouth,
        shade,
    );
    Ok(image)
}
