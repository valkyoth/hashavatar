pub fn render_astronaut_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.54) as i32;
    let helmet_r = (width.min(height) as f32 * 0.28) as i32;
    let suit = hsl_to_color(205.0 + identity.unit_f32(1) * 35.0, 0.16, 0.90);
    let visor = hsl_to_color(195.0 + identity.unit_f32(2) * 55.0, 0.52, 0.56);
    let trim = hsl_to_color(identity.unit_f32(3) * 360.0, 0.45, 0.55);
    let bg = hsl_to_color(220.0 + identity.unit_f32(4) * 60.0, 0.18, 0.91);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, helmet_r, helmet_r, trim, 0.40, background, identity,
    );

    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - helmet_r / 2, center_y + helmet_r / 2)
            .of_size(helmet_r as u32, (helmet_r * 3 / 5) as u32),
        suit.into(),
    );
    draw_filled_circle_mut(&mut image, (center_x, center_y), helmet_r, suit.into());
    draw_hollow_circle_mut(
        &mut image,
        (center_x, center_y),
        helmet_r,
        Color::rgb(96, 110, 128).into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        helmet_r * 2 / 3,
        helmet_r / 2,
        visor.into(),
    );
    draw_blended_rect_mut(
        &mut image,
        Rect::at(center_x - helmet_r / 3, center_y - helmet_r / 4)
            .of_size((helmet_r / 2).max(2) as u32, (helmet_r / 8).max(2) as u32),
        Rgba([255, 255, 255, 90]),
    );
    draw_filled_circle_mut(
        &mut image,
        (center_x + helmet_r / 2, center_y + helmet_r * 2 / 3),
        (helmet_r / 10).max(3),
        trim.into(),
    );

    Ok(image)
}

