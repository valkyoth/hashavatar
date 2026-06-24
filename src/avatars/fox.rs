use super::*;

pub fn render_fox_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let bg = hsl_to_color(22.0 + identity.unit_f32(0) * 26.0, 0.26, 0.92);
    let orange = hsl_to_color(18.0 + identity.unit_f32(1) * 20.0, 0.76, 0.58);
    let deep_orange = hsl_to_color(16.0 + identity.unit_f32(2) * 12.0, 0.72, 0.42);
    let cream = hsl_to_color(40.0 + identity.unit_f32(3) * 10.0, 0.32, 0.93);
    let eye = Color::rgb(34, 28, 24);
    let nose = Color::rgb(55, 40, 34);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, bg).into());

    let head_rx = (width as f32 * (0.25 + identity.unit_f32(4) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(5) * 0.08)) as i32;
    let ear_h = (height as f32 * (0.16 + identity.unit_f32(6) * 0.09)) as i32;
    let ear_w = (width as f32 * (0.12 + identity.unit_f32(7) * 0.05)) as i32;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        deep_orange,
        0.35,
        background,
        identity,
    );
    draw_ear(
        &mut image,
        EarSpec::left(center_x, center_y, head_rx, head_ry, ear_w, ear_h, -0.2),
        orange,
        cream,
        deep_orange,
    );
    draw_ear(
        &mut image,
        EarSpec::right(center_x, center_y, head_rx, head_ry, ear_w, ear_h, 0.2),
        orange,
        cream,
        deep_orange,
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        orange.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 4),
        head_rx / 2,
        head_ry / 3,
        cream.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - head_rx / 2, center_y - head_ry / 8),
            Point::new(center_x, center_y + head_ry / 3),
            Point::new(center_x - head_rx / 8, center_y + head_ry / 2),
        ],
        cream.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x + head_rx / 2, center_y - head_ry / 8),
            Point::new(center_x, center_y + head_ry / 3),
            Point::new(center_x + head_rx / 8, center_y + head_ry / 2),
        ],
        cream.into(),
    );

    let eye_y = center_y - head_ry / 7;
    let eye_offset = head_rx / 3;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_ellipse_mut(
            &mut image,
            (x, eye_y),
            head_rx / 10,
            head_ry / 8,
            Color::rgb(255, 255, 255).into(),
        );
        draw_filled_ellipse_mut(
            &mut image,
            (x, eye_y),
            head_rx / 18,
            head_ry / 7,
            eye.into(),
        );
    }
    let nose_y = center_y + head_ry / 4;
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - head_rx / 10, nose_y),
            Point::new(center_x + head_rx / 10, nose_y),
            Point::new(center_x, nose_y + head_ry / 10),
        ],
        nose.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.45,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.45,
    );
    Ok(image)
}
