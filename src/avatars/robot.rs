pub fn render_robot_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let bg = hsl_to_color(
        210.0 + identity.unit_f32(0) * 70.0,
        0.18 + identity.unit_f32(1) * 0.18,
        0.92,
    );
    let accent = hsl_to_color(160.0 + identity.unit_f32(2) * 120.0, 0.48, 0.62);
    let metal = hsl_to_color(200.0 + identity.unit_f32(3) * 28.0, 0.16, 0.74);
    let trim = hsl_to_color(205.0 + identity.unit_f32(4) * 22.0, 0.18, 0.46);
    let light = hsl_to_color(50.0 + identity.unit_f32(5) * 120.0, 0.84, 0.66);
    let dark = Color::rgb(47, 60, 72);
    let bg_fill = background_fill(background, bg);
    image.pixels_mut().for_each(|pixel| *pixel = bg_fill.into());

    let head_w = (width as f32 * (0.44 + identity.unit_f32(6) * 0.12)) as i32;
    let head_h = (height as f32 * (0.34 + identity.unit_f32(7) * 0.10)) as i32;
    let head_x = center_x - head_w / 2;
    let head_y = center_y - head_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_w / 2,
        head_h / 2,
        accent,
        0.5,
        background,
        identity,
    );

    draw_filled_rect_mut(
        &mut image,
        Rect::at(head_x, head_y).of_size(head_w as u32, head_h as u32),
        metal.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, head_y as f32),
        ((head_x + head_w) as f32, head_y as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, (head_y + head_h) as f32),
        ((head_x + head_w) as f32, (head_y + head_h) as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (head_x as f32, head_y as f32),
        (head_x as f32, (head_y + head_h) as f32),
        trim.into(),
    );
    draw_line_segment_mut(
        &mut image,
        ((head_x + head_w) as f32, head_y as f32),
        ((head_x + head_w) as f32, (head_y + head_h) as f32),
        trim.into(),
    );

    let antenna_h = (height as f32 * 0.10) as i32;
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, (head_y - antenna_h / 2) as f32),
        (center_x as f32, head_y as f32),
        dark.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (center_x, head_y - antenna_h / 2),
        (head_w as f32 * 0.05) as i32,
        accent.into(),
    );

    let eye_y = center_y - head_h / 6;
    let eye_offset = head_w / 4;
    let eye_rx = (head_w as f32 * 0.12) as i32;
    let eye_ry = (head_h as f32 * 0.10) as i32;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_ellipse_mut(&mut image, (x, eye_y), eye_rx, eye_ry, light.into());
        if identity.byte(8).is_multiple_of(2) {
            draw_filled_circle_mut(
                &mut image,
                (x, eye_y),
                (eye_rx as f32 * 0.35) as i32,
                dark.into(),
            );
        }
    }

    let mouth_y = center_y + head_h / 5;
    let mouth_w = (head_w as f32 * 0.42) as i32;
    let mouth_h = (head_h as f32 * 0.12) as i32;
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - mouth_w / 2, mouth_y - mouth_h / 2)
            .of_size(mouth_w as u32, mouth_h as u32),
        dark.into(),
    );
    let teeth = 4 + (identity.byte(9) % 4) as i32;
    for idx in 1..teeth {
        let x = center_x - mouth_w / 2 + idx * mouth_w / teeth;
        draw_line_segment_mut(
            &mut image,
            (x as f32, (mouth_y - mouth_h / 2) as f32),
            (x as f32, (mouth_y + mouth_h / 2) as f32),
            metal.into(),
        );
    }

    let bolt_y = center_y;
    draw_filled_circle_mut(
        &mut image,
        (head_x + head_w / 8, bolt_y),
        (head_w as f32 * 0.035) as i32,
        trim.into(),
    );
    draw_filled_circle_mut(
        &mut image,
        (head_x + head_w - head_w / 8, bolt_y),
        (head_w as f32 * 0.035) as i32,
        trim.into(),
    );
    Ok(image)
}

