use super::*;

/// Render a mushroom avatar from a stable identity.
pub fn render_mushroom_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let bg = hsl_to_color(18.0 + identity.unit_f32(0) * 35.0, 0.20, 0.93);
    let cap = hsl_to_color(350.0 + identity.unit_f32(1) * 45.0, 0.58, 0.52);
    let stem = hsl_to_color(35.0 + identity.unit_f32(2) * 20.0, 0.24, 0.86);
    let gill = hsl_to_color(26.0 + identity.unit_f32(3) * 20.0, 0.20, 0.70);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let cap_rx = (width as f32 * (0.24 + identity.unit_f32(4) * 0.08)) as i32;
    let cap_ry = (height as f32 * (0.14 + identity.unit_f32(5) * 0.06)) as i32;
    let stem_rx = (width as f32 * (0.09 + identity.unit_f32(6) * 0.04)) as i32;
    let stem_ry = (height as f32 * (0.18 + identity.unit_f32(7) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, cap_rx, cap_ry, gill, 0.24, background, identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + stem_ry / 3),
        stem_rx,
        stem_ry,
        stem.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y - cap_ry / 2),
        cap_rx,
        cap_ry,
        cap.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - cap_rx, center_y - cap_ry / 2)
            .of_size((cap_rx * 2) as u32, cap_ry.max(1) as u32),
        cap.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + cap_ry / 3),
        cap_rx,
        cap_ry / 3,
        gill.into(),
    );
    let spot_count = 3 + (identity.byte(8) % 4) as i32;
    for spot in 0..spot_count {
        let sx = center_x - cap_rx / 2 + (identity.byte(9 + spot as usize) as i32 % cap_rx.max(1));
        let sy = center_y - cap_ry + (identity.byte(14 + spot as usize) as i32 % cap_ry.max(1));
        draw_filled_circle_mut(
            &mut image,
            (sx, sy),
            (cap_rx as f32 * (0.06 + identity.unit_f32(19 + spot as usize) * 0.04)) as i32,
            Color::rgba(255, 246, 230, 230).into(),
        );
    }
    Ok(image)
}
