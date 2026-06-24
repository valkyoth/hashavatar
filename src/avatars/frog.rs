use super::*;

/// Render a frog avatar from a stable identity.
pub fn render_frog_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.57 + identity.unit_f32(6) * 0.04)) as i32;
    let bg = hsl_to_color(95.0 + identity.unit_f32(0) * 65.0, 0.23, 0.92);
    let green = hsl_to_color(92.0 + identity.unit_f32(1) * 72.0, 0.46, 0.54);
    let dark = hsl_to_color(98.0 + identity.unit_f32(2) * 60.0, 0.40, 0.28);
    let cheek = hsl_to_color(335.0 + identity.unit_f32(3) * 24.0, 0.42, 0.76);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(4) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.18 + identity.unit_f32(5) * 0.05)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, dark, 0.20, background, identity,
    );
    let eye_offset = (head_rx as f32 * 0.50) as i32;
    let eye_r = (head_rx as f32 * (0.18 + identity.unit_f32(7) * 0.04)) as i32;
    for side in [-1, 1] {
        draw_filled_circle_mut(
            &mut image,
            (center_x + side * eye_offset, center_y - head_ry),
            eye_r,
            green.into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        green.into(),
    );
    for side in [-1, 1] {
        let ex = center_x + side * eye_offset;
        let ey = center_y - head_ry;
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            (eye_r as f32 * 0.64) as i32,
            Color::rgb(255, 255, 245).into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (ex, ey),
            (eye_r as f32 * 0.30) as i32,
            dark.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x + side * head_rx / 2, center_y + head_ry / 4),
            head_rx / 9,
            Color::rgba(cheek.0[0], cheek.0[1], cheek.0[2], 150).into(),
        );
    }
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 9,
        center_y + head_ry / 4,
        head_rx / 4,
        dark,
        0.50,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 9,
        center_y + head_ry / 4,
        head_rx / 4,
        dark,
        0.50,
    );
    if identity.byte(8).is_multiple_of(2) {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x - head_rx / 10) as f32,
                (center_y + head_ry / 6) as f32,
            ),
            (
                (center_x - head_rx / 10) as f32,
                (center_y + head_ry / 4) as f32,
            ),
            dark.into(),
        );
        draw_line_segment_mut(
            &mut image,
            (
                (center_x + head_rx / 10) as f32,
                (center_y + head_ry / 6) as f32,
            ),
            (
                (center_x + head_rx / 10) as f32,
                (center_y + head_ry / 4) as f32,
            ),
            dark.into(),
        );
    }
    Ok(image)
}
