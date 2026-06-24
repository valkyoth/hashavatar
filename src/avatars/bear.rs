use super::*;

pub fn render_bear_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let head_rx = (width as f32 * (0.27 + identity.unit_f32(4) * 0.05)) as i32;
    let head_ry = (height as f32 * (0.24 + identity.unit_f32(5) * 0.05)) as i32;
    let fur = hsl_to_color(24.0 + identity.unit_f32(1) * 24.0, 0.38, 0.48);
    let muzzle = hsl_to_color(32.0 + identity.unit_f32(2) * 12.0, 0.22, 0.84);
    let inner = hsl_to_color(18.0 + identity.unit_f32(3) * 18.0, 0.34, 0.72);
    let bg = hsl_to_color(34.0 + identity.unit_f32(6) * 24.0, 0.18, 0.93);
    let dark = Color::rgb(45, 34, 28);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, inner, 0.42, background, identity,
    );

    let ear_r = (head_rx as f32 * 0.28) as i32;
    for x in [center_x - head_rx * 3 / 4, center_x + head_rx * 3 / 4] {
        draw_filled_circle_mut(&mut image, (x, center_y - head_ry), ear_r, fur.into());
        draw_filled_circle_mut(&mut image, (x, center_y - head_ry), ear_r / 2, inner.into());
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        fur.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 4),
        head_rx * 2 / 5,
        head_ry / 3,
        muzzle.into(),
    );
    let eye_y = center_y - head_ry / 5;
    for x in [center_x - head_rx / 3, center_x + head_rx / 3] {
        draw_filled_circle_mut(&mut image, (x, eye_y), (head_rx / 10).max(3), dark.into());
    }
    let nose_y = center_y + head_ry / 6;
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, nose_y),
        head_rx / 8,
        head_ry / 10,
        dark.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x,
        nose_y + head_ry / 12,
        head_rx / 5,
        dark,
        0.35,
    );

    Ok(image)
}
