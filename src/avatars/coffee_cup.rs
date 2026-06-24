pub fn render_coffee_cup_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let cup_w = (width as f32 * 0.38) as i32;
    let cup_h = (height as f32 * 0.32) as i32;
    let cup = hsl_to_color(20.0 + identity.unit_f32(1) * 35.0, 0.42, 0.60);
    let coffee = hsl_to_color(24.0 + identity.unit_f32(2) * 18.0, 0.42, 0.26);
    let cream = hsl_to_color(38.0 + identity.unit_f32(3) * 18.0, 0.26, 0.88);
    let bg = hsl_to_color(34.0 + identity.unit_f32(4) * 20.0, 0.18, 0.93);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        cup_w / 2,
        cup_h / 2,
        cream,
        0.30,
        background,
        identity,
    );

    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - cup_w / 2, center_y - cup_h / 2).of_size(cup_w as u32, cup_h as u32),
        cup.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y - cup_h / 2),
        cup_w / 2,
        cup_h / 7,
        coffee.into(),
    );
    draw_hollow_ellipse_mut(
        &mut image,
        (center_x + cup_w / 2, center_y - cup_h / 10),
        cup_w / 4,
        cup_h / 4,
        cup.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - cup_w * 3 / 5, center_y + cup_h / 2)
            .of_size((cup_w * 6 / 5) as u32, (cup_h / 8).max(2) as u32),
        Color::rgba(80, 55, 42, 180).into(),
    );
    for offset in [-1, 0, 1] {
        let x = center_x + offset * cup_w / 5;
        draw_line_segment_mut(
            &mut image,
            (x as f32, (center_y - cup_h) as f32),
            ((x + cup_w / 10) as f32, (center_y - cup_h * 4 / 3) as f32),
            Color::rgba(120, 98, 82, 120).into(),
        );
    }

    Ok(image)
}

