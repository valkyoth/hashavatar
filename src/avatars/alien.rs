use super::*;

pub fn render_alien_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let skin = hsl_to_color(
        90.0 + identity.unit_f32(0) * 80.0,
        0.45 + identity.unit_f32(1) * 0.20,
        0.68,
    );
    let shade = hsl_to_color(110.0 + identity.unit_f32(2) * 50.0, 0.38, 0.44);
    let accent = hsl_to_color(280.0 + identity.unit_f32(3) * 40.0, 0.32, 0.92);
    let eye = Color::rgb(28, 18, 38);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, accent).into());
    let head_rx = (width as f32 * (0.20 + identity.unit_f32(4) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.28 + identity.unit_f32(5) * 0.10)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, shade, 0.28, background, identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        skin.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x - head_rx / 2, center_y - head_ry / 4),
        head_rx / 5,
        head_ry / 3,
        eye.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + head_rx / 2, center_y - head_ry / 4),
        head_rx / 5,
        head_ry / 3,
        eye.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (center_x, center_y + head_ry / 8),
        head_rx / 14,
        shade.into(),
    );
    if identity.byte(6).is_multiple_of(2) {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x - head_rx / 8) as f32,
                (center_y + head_ry / 3) as f32,
            ),
            (
                (center_x + head_rx / 8) as f32,
                (center_y + head_ry / 3) as f32,
            ),
            shade.into(),
        );
    }
    Ok(image)
}
