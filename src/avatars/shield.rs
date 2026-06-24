pub fn render_shield_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.55) as i32;
    let rx = (width as f32 * 0.25) as i32;
    let ry = (height as f32 * 0.32) as i32;
    let metal = hsl_to_color(210.0 + identity.unit_f32(1) * 45.0, 0.28, 0.58);
    let accent = hsl_to_color(identity.unit_f32(2) * 360.0, 0.50, 0.50);
    let light = hsl_to_color(210.0 + identity.unit_f32(3) * 35.0, 0.18, 0.82);
    let bg = hsl_to_color(215.0 + identity.unit_f32(4) * 35.0, 0.16, 0.92);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, rx, ry, accent, 0.36, background, identity,
    );

    let shield = [
        Point::new(center_x - rx, center_y - ry),
        Point::new(center_x + rx, center_y - ry),
        Point::new(center_x + rx * 4 / 5, center_y + ry / 4),
        Point::new(center_x, center_y + ry),
        Point::new(center_x - rx * 4 / 5, center_y + ry / 4),
    ];
    draw_polygon_mut(&mut image, &shield, metal.into());
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - rx, center_y - ry),
            Point::new(center_x, center_y - ry),
            Point::new(center_x, center_y + ry),
            Point::new(center_x - rx * 4 / 5, center_y + ry / 4),
        ],
        light.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - rx / 8, center_y - ry * 3 / 4)
            .of_size((rx / 4).max(2) as u32, (ry * 6 / 5) as u32),
        accent.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - rx * 2 / 3, center_y - ry / 5)
            .of_size((rx * 4 / 3) as u32, (ry / 4).max(2) as u32),
        accent.into(),
    );

    Ok(image)
}

