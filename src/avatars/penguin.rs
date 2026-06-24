pub fn render_penguin_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let body_rx = (width as f32 * 0.25) as i32;
    let body_ry = (height as f32 * 0.34) as i32;
    let black = hsl_to_color(210.0 + identity.unit_f32(1) * 30.0, 0.22, 0.18);
    let white = hsl_to_color(205.0 + identity.unit_f32(2) * 25.0, 0.16, 0.94);
    let orange = hsl_to_color(32.0 + identity.unit_f32(3) * 18.0, 0.72, 0.58);
    let bg = hsl_to_color(190.0 + identity.unit_f32(4) * 35.0, 0.22, 0.93);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, body_rx, body_ry, orange, 0.36, background, identity,
    );

    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        body_rx,
        body_ry,
        black.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + body_ry / 6),
        body_rx * 3 / 5,
        body_ry * 2 / 3,
        white.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x - body_rx, center_y + body_ry / 10),
        body_rx / 4,
        body_ry / 2,
        black.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + body_rx, center_y + body_ry / 10),
        body_rx / 4,
        body_ry / 2,
        black.into(),
    );
    let eye_y = center_y - body_ry / 3;
    for x in [center_x - body_rx / 3, center_x + body_rx / 3] {
        draw_filled_circle_mut(
            &mut image,
            (x, eye_y),
            (body_rx / 10).max(3),
            Color::rgb(10, 15, 20).into(),
        );
    }
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_rx / 7, center_y - body_ry / 6),
            Point::new(center_x + body_rx / 7, center_y - body_ry / 6),
            Point::new(center_x, center_y),
        ],
        orange.into(),
    );
    for x in [center_x - body_rx / 3, center_x + body_rx / 3] {
        draw_filled_ellipse_mut(
            &mut image,
            (x, center_y + body_ry),
            body_rx / 4,
            body_ry / 10,
            orange.into(),
        );
    }

    Ok(image)
}

