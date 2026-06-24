/// Render a pizza-slice avatar from a stable identity.
pub fn render_pizza_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.53) as i32;
    let bg = hsl_to_color(36.0 + identity.unit_f32(0) * 30.0, 0.24, 0.93);
    let crust = hsl_to_color(30.0 + identity.unit_f32(1) * 28.0, 0.54, 0.58);
    let cheese = hsl_to_color(45.0 + identity.unit_f32(2) * 16.0, 0.74, 0.70);
    let sauce = hsl_to_color(8.0 + identity.unit_f32(3) * 16.0, 0.62, 0.48);
    let topping = hsl_to_color(350.0 + identity.unit_f32(4) * 22.0, 0.54, 0.46);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let half_w = (width as f32 * (0.22 + identity.unit_f32(5) * 0.06)) as i32;
    let slice_h = (height as f32 * (0.44 + identity.unit_f32(6) * 0.07)) as i32;
    let top_y = center_y - slice_h / 2;
    let tip_y = center_y + slice_h / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        half_w,
        slice_h / 2,
        sauce,
        0.16,
        background,
        identity,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - half_w, top_y),
            Point::new(center_x + half_w, top_y),
            Point::new(center_x, tip_y),
        ],
        cheese.into(),
    );
    draw_line_segment_mut(
        &mut image,
        ((center_x - half_w) as f32, top_y as f32),
        ((center_x + half_w) as f32, top_y as f32),
        crust.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y),
        half_w,
        (height as f32 * 0.035) as i32,
        crust.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - half_w + half_w / 6, top_y + slice_h / 5),
            Point::new(center_x + half_w - half_w / 6, top_y + slice_h / 5),
            Point::new(center_x, tip_y - slice_h / 10),
        ],
        Color::rgba(sauce.0[0], sauce.0[1], sauce.0[2], 95).into(),
    );
    let topping_count = 3 + (identity.byte(7) % 4) as i32;
    for item in 0..topping_count {
        let y = top_y + slice_h / 5 + item * slice_h / (topping_count + 2);
        let span = half_w - (y - top_y) * half_w / slice_h;
        let x = center_x - span / 2 + (identity.byte(8 + item as usize) as i32 % span.max(1));
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            (width as f32 * (0.025 + identity.unit_f32(14 + item as usize) * 0.012)) as i32,
            topping.into(),
        );
    }
    Ok(image)
}

