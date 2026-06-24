use super::*;

/// Render a rocket avatar from a stable identity.
pub fn render_rocket_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.52) as i32;
    let bg = hsl_to_color(205.0 + identity.unit_f32(0) * 70.0, 0.22, 0.92);
    let hull = hsl_to_color(200.0 + identity.unit_f32(1) * 50.0, 0.12, 0.88);
    let trim = hsl_to_color(identity.unit_f32(2) * 360.0, 0.58, 0.54);
    let window = hsl_to_color(185.0 + identity.unit_f32(3) * 70.0, 0.54, 0.72);
    let flame = hsl_to_color(20.0 + identity.unit_f32(4) * 30.0, 0.86, 0.58);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        width / 5,
        height / 5,
        trim,
        0.18,
        background,
        identity,
    );

    let body_w = (width as f32 * (0.18 + identity.unit_f32(5) * 0.05)) as i32;
    let body_h = (height as f32 * (0.42 + identity.unit_f32(6) * 0.08)) as i32;
    let top_y = center_y - body_h / 2;
    let bottom_y = center_y + body_h / 2;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 2, top_y + body_w / 2),
            Point::new(center_x + body_w / 2, top_y + body_w / 2),
            Point::new(center_x, top_y - body_w / 2),
        ],
        trim.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - body_w / 2, top_y + body_w / 2)
            .of_size(body_w as u32, (body_h - body_w / 2).max(1) as u32),
        hull.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, top_y + body_w / 2),
        body_w / 2,
        body_w / 5,
        hull.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 2, bottom_y - body_w / 2),
            Point::new(center_x - body_w, bottom_y + body_w / 3),
            Point::new(center_x - body_w / 2, bottom_y),
        ],
        trim.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x + body_w / 2, bottom_y - body_w / 2),
            Point::new(center_x + body_w, bottom_y + body_w / 3),
            Point::new(center_x + body_w / 2, bottom_y),
        ],
        trim.into(),
    );
    let window_count = 1 + (identity.byte(7) % 2) as i32;
    for window_index in 0..window_count {
        let y = top_y + body_h / 3 + window_index * body_w;
        draw_filled_circle_mut(
            &mut image,
            (center_x, y),
            (body_w as f32 * 0.22) as i32,
            trim.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x, y),
            (body_w as f32 * 0.15) as i32,
            window.into(),
        );
    }
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - body_w / 4, bottom_y),
            Point::new(center_x + body_w / 4, bottom_y),
            Point::new(
                center_x,
                bottom_y + (height as f32 * (0.10 + identity.unit_f32(8) * 0.08)) as i32,
            ),
        ],
        flame.into(),
    );
    Ok(image)
}
