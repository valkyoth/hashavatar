pub fn render_skull_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.18 + identity.unit_f32(17) * 0.07)) as i32;
    let head_ry = (height as f32 * (0.18 + identity.unit_f32(18) * 0.07)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.51 + identity.unit_f32(19) * 0.07)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(
        195.0 + identity.unit_f32(15) * 55.0,
        0.06 + identity.unit_f32(20) * 0.08,
        0.92 + identity.unit_f32(21) * 0.04,
    );
    let bone = hsl_to_color(
        28.0 + identity.unit_f32(16) * 34.0,
        0.08 + identity.unit_f32(22) * 0.10,
        0.82 + identity.unit_f32(23) * 0.12,
    );
    let crack = hsl_to_color(
        20.0 + identity.unit_f32(24) * 40.0,
        0.06,
        0.22 + identity.unit_f32(25) * 0.12,
    );
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        crack,
        0.16,
        background,
        identity,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        layout.head_ry,
        bone.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(
            layout.center_x - layout.head_rx / 2,
            layout.center_y + layout.head_ry / 2,
        )
        .of_size(
            (layout.head_rx as f32 * (0.82 + identity.unit_f32(26) * 0.34)) as u32,
            (layout.head_ry as f32 * (0.34 + identity.unit_f32(27) * 0.28)) as u32,
        ),
        bone.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        if identity.byte(28).is_multiple_of(2) {
            CreatureEyeStyle::Hollow
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(44, 42, 44),
        Color::rgb(44, 42, 44),
    );
    let nose_half_width = (layout.head_rx as f32 * (0.08 + identity.unit_f32(29) * 0.08)) as i32;
    let nose_height = (layout.head_ry as f32 * (0.12 + identity.unit_f32(30) * 0.12)) as i32;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(
                layout.center_x - nose_half_width,
                layout.center_y + nose_height,
            ),
            Point::new(
                layout.center_x + nose_half_width,
                layout.center_y + nose_height,
            ),
        ],
        crack.into(),
    );
    draw_creature_mouth(
        &mut image,
        layout,
        if identity.byte(31).is_multiple_of(2) {
            CreatureMouthStyle::Flat
        } else {
            CreatureMouthStyle::Smile
        },
        crack,
    );
    let tooth_count = 3 + (identity.byte(32) % 4) as i32;
    for tooth in 0..tooth_count {
        let x = layout.center_x - layout.head_rx / 3
            + tooth * (layout.head_rx * 2 / tooth_count.max(1));
        draw_line_segment_mut(
            &mut image,
            (x as f32, (layout.center_y + layout.head_ry / 2) as f32),
            (x as f32, (layout.center_y + layout.head_ry) as f32),
            crack.into(),
        );
    }
    let crack_count = 1 + (identity.byte(33) % 3) as i32;
    for line in 0..crack_count {
        let start_x = layout.center_x - layout.head_rx / 4
            + (identity.byte(34 + line as usize) as i32 % (layout.head_rx / 2).max(1));
        let start_y = layout.center_y - layout.head_ry / 2
            + (identity.byte(38 + line as usize) as i32 % (layout.head_ry / 2).max(1));
        let end_x = start_x
            + ((identity.unit_f32(42 + line as usize) - 0.5) * layout.head_rx as f32 * 0.45) as i32;
        let end_y = start_y
            + (layout.head_ry as f32 * (0.18 + identity.unit_f32(46 + line as usize) * 0.32))
                as i32;
        draw_line_segment_mut(
            &mut image,
            (start_x as f32, start_y as f32),
            (end_x as f32, end_y as f32),
            crack.into(),
        );
    }
    draw_line_segment_mut(
        &mut image,
        (
            (layout.center_x + layout.head_rx / 4) as f32,
            (layout.center_y - layout.head_ry / 2) as f32,
        ),
        (
            (layout.center_x + layout.head_rx / 8) as f32,
            (layout.center_y - layout.head_ry / 8) as f32,
        ),
        crack.into(),
    );
    Ok(image)
}

