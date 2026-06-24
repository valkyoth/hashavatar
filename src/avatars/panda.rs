/// Render a panda avatar from a stable identity.
pub fn render_panda_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.56 + identity.unit_f32(5) * 0.04)) as i32;
    let bg = hsl_to_color(200.0 + identity.unit_f32(0) * 45.0, 0.08, 0.94);
    let white = hsl_to_color(36.0 + identity.unit_f32(1) * 18.0, 0.10, 0.92);
    let black = hsl_to_color(210.0 + identity.unit_f32(2) * 28.0, 0.10, 0.18);
    let blush = hsl_to_color(345.0 + identity.unit_f32(3) * 25.0, 0.32, 0.78);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(6) * 0.05)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(7) * 0.04)) as i32;
    let ear_r = (head_rx as f32 * (0.28 + identity.unit_f32(8) * 0.08)) as i32;
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, black, 0.16, background, identity,
    );
    for side in [-1, 1] {
        draw_filled_circle_mut(
            &mut image,
            (
                center_x + side * head_rx * 3 / 4,
                center_y - head_ry * 3 / 4,
            ),
            ear_r,
            black.into(),
        );
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        white.into(),
    );
    for side in [-1, 1] {
        let patch_x = center_x + side * head_rx / 3;
        let patch_y = center_y - head_ry / 8;
        draw_filled_ellipse_mut(
            &mut image,
            (patch_x, patch_y),
            (head_rx as f32 * (0.20 + identity.unit_f32(9) * 0.05)) as i32,
            (head_ry as f32 * (0.26 + identity.unit_f32(10) * 0.05)) as i32,
            black.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (patch_x, patch_y),
            (head_rx as f32 * 0.055) as i32,
            Color::rgb(248, 248, 244).into(),
        );
        if identity.byte(11).is_multiple_of(2) {
            draw_filled_circle_mut(
                &mut image,
                (center_x + side * head_rx / 2, center_y + head_ry / 4),
                head_rx / 10,
                Color::rgba(blush.0[0], blush.0[1], blush.0[2], 120).into(),
            );
        }
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 5),
        head_rx / 9,
        head_ry / 12,
        black.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        center_y + head_ry / 4,
        head_rx / 6,
        black,
        0.45,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        center_y + head_ry / 4,
        head_rx / 6,
        black,
        0.45,
    );
    Ok(image)
}

