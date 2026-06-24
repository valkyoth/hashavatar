use super::*;

pub fn render_dragon_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.57) as i32;
    let head_rx = (width as f32 * 0.27) as i32;
    let head_ry = (height as f32 * 0.23) as i32;
    let scale = hsl_to_color(105.0 + identity.unit_f32(1) * 70.0, 0.46, 0.46);
    let belly = hsl_to_color(70.0 + identity.unit_f32(2) * 35.0, 0.42, 0.72);
    let horn = hsl_to_color(40.0 + identity.unit_f32(3) * 20.0, 0.34, 0.84);
    let flame = hsl_to_color(14.0 + identity.unit_f32(4) * 25.0, 0.78, 0.56);
    let bg = hsl_to_color(120.0 + identity.unit_f32(5) * 45.0, 0.18, 0.92);
    let dark = Color::rgb(24, 48, 34);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, flame, 0.40, background, identity,
    );

    for side in [-1, 1] {
        let horn_points = [
            Point::new(center_x + side * head_rx / 2, center_y - head_ry),
            Point::new(center_x + side * head_rx / 4, center_y - head_ry * 8 / 5),
            Point::new(center_x + side * head_rx / 8, center_y - head_ry),
        ];
        draw_polygon_mut(&mut image, &horn_points, horn.into());
    }
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        scale.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 4),
        head_rx / 2,
        head_ry / 3,
        belly.into(),
    );
    for x in [center_x - head_rx / 3, center_x + head_rx / 3] {
        draw_filled_circle_mut(
            &mut image,
            (x, center_y - head_ry / 5),
            (head_rx / 10).max(3),
            Color::rgb(255, 255, 255).into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (x, center_y - head_ry / 5),
            (head_rx / 20).max(2),
            dark.into(),
        );
    }
    for x in [center_x - head_rx / 7, center_x + head_rx / 7] {
        draw_filled_circle_mut(
            &mut image,
            (x, center_y + head_ry / 3),
            (head_rx / 24).max(2),
            dark.into(),
        );
    }
    for offset in [-1, 0, 1] {
        let x = center_x + offset * head_rx / 5;
        let spike_y = center_y - head_ry;
        let spike = [
            Point::new(x - head_rx / 14, spike_y),
            Point::new(x, spike_y - head_ry / 4),
            Point::new(x + head_rx / 14, spike_y),
        ];
        draw_polygon_mut(&mut image, &spike, flame.into());
    }

    Ok(image)
}
