use super::*;

/// Render a knight helmet avatar from a stable identity.
pub fn render_knight_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.55) as i32;
    let bg = hsl_to_color(215.0 + identity.unit_f32(0) * 30.0, 0.12, 0.92);
    let steel = hsl_to_color(205.0 + identity.unit_f32(1) * 45.0, 0.12, 0.66);
    let dark = hsl_to_color(215.0 + identity.unit_f32(2) * 45.0, 0.14, 0.22);
    let plume = hsl_to_color(identity.unit_f32(3) * 360.0, 0.58, 0.54);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let helm_rx = (width as f32 * (0.20 + identity.unit_f32(4) * 0.05)) as i32;
    let helm_ry = (height as f32 * (0.24 + identity.unit_f32(5) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, helm_rx, helm_ry, plume, 0.18, background, identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y - helm_ry / 5),
        helm_rx,
        helm_ry,
        steel.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - helm_rx, center_y - helm_ry / 5)
            .of_size((helm_rx * 2) as u32, (helm_ry * 6 / 5) as u32),
        steel.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - helm_rx * 3 / 4, center_y - helm_ry / 5)
            .of_size((helm_rx * 3 / 2) as u32, (helm_ry / 5).max(2) as u32),
        dark.into(),
    );
    let slit_count = 2 + (identity.byte(6) % 3) as i32;
    for slit in 0..slit_count {
        let x = center_x - helm_rx / 2 + slit * helm_rx / slit_count.max(1);
        draw_filled_rect_mut(
            &mut image,
            Rect::at(x, center_y - helm_ry / 5)
                .of_size((helm_rx / 10).max(2) as u32, (helm_ry / 5).max(2) as u32),
            Color::rgba(255, 255, 255, 90).into(),
        );
    }
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, (center_y - helm_ry) as f32),
        (center_x as f32, (center_y + helm_ry) as f32),
        Color::rgba(255, 255, 255, 130).into(),
    );
    if !identity.byte(7).is_multiple_of(3) {
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x, center_y - helm_ry),
                Point::new(center_x - helm_rx / 5, center_y - helm_ry - helm_ry / 2),
                Point::new(center_x + helm_rx / 4, center_y - helm_ry - helm_ry / 3),
            ],
            plume.into(),
        );
    }
    Ok(image)
}
