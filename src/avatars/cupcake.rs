/// Render a cupcake avatar from a stable identity.
pub fn render_cupcake_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let bg = hsl_to_color(320.0 + identity.unit_f32(0) * 45.0, 0.22, 0.94);
    let wrapper = hsl_to_color(28.0 + identity.unit_f32(1) * 35.0, 0.46, 0.62);
    let frosting = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.78);
    let shadow = hsl_to_color(330.0 + identity.unit_f32(3) * 45.0, 0.28, 0.58);
    let cherry = hsl_to_color(345.0 + identity.unit_f32(4) * 22.0, 0.66, 0.50);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let cup_w = (width as f32 * (0.26 + identity.unit_f32(5) * 0.07)) as i32;
    let cup_h = (height as f32 * (0.22 + identity.unit_f32(6) * 0.05)) as i32;
    let frosting_rx = (cup_w as f32 * (0.58 + identity.unit_f32(7) * 0.10)) as i32;
    let frosting_ry = (height as f32 * (0.13 + identity.unit_f32(8) * 0.04)) as i32;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        cup_w / 2,
        cup_h,
        shadow,
        0.24,
        background,
        identity,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - cup_w / 2, center_y),
            Point::new(center_x + cup_w / 2, center_y),
            Point::new(center_x + cup_w / 3, center_y + cup_h),
            Point::new(center_x - cup_w / 3, center_y + cup_h),
        ],
        wrapper.into(),
    );
    for stripe in [-2, 0, 2] {
        let x = center_x + stripe * cup_w / 10;
        draw_line_segment_mut(
            &mut image,
            (x as f32, center_y as f32),
            ((x - stripe * cup_w / 40) as f32, (center_y + cup_h) as f32),
            Color::rgba(255, 244, 214, 115).into(),
        );
    }
    let base_y = center_y - frosting_ry / 2;
    for layer in 0..3 {
        let y = base_y - layer * frosting_ry / 2;
        let rx = (frosting_rx as f32 * (1.0 - layer as f32 * 0.22)) as i32;
        let ry = (frosting_ry as f32 * (0.82 - layer as f32 * 0.10)) as i32;
        draw_filled_ellipse_mut(&mut image, (center_x, y), rx, ry, frosting.into());
    }
    let sprinkle_count = 3 + (identity.byte(9) % 5) as i32;
    for sprinkle in 0..sprinkle_count {
        let sx = center_x - frosting_rx / 2
            + (identity.byte(10 + sprinkle as usize) as i32 % frosting_rx.max(1));
        let sy = base_y - frosting_ry
            + (identity.byte(16 + sprinkle as usize) as i32 % (frosting_ry * 2).max(1));
        let color = hsl_to_color(
            identity.unit_f32(23 + sprinkle as usize) * 360.0,
            0.62,
            0.55,
        );
        draw_filled_rect_mut(
            &mut image,
            Rect::at(sx, sy).of_size((width / 40).max(2) as u32, (height / 80).max(2) as u32),
            color.into(),
        );
    }
    if !identity.byte(30).is_multiple_of(3) {
        draw_filled_circle_mut(
            &mut image,
            (center_x, base_y - frosting_ry),
            (width as f32 * 0.035) as i32,
            cherry.into(),
        );
    }
    Ok(image)
}

