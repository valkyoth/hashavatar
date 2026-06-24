use super::*;

pub fn render_dog_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let head_rx = (width as f32 * (0.24 + identity.unit_f32(0) * 0.08)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(1) * 0.09)) as i32;
    let ear_drop = (head_ry as f32 * (0.65 + identity.unit_f32(2) * 0.35)) as i32;
    let muzzle_rx = (head_rx as f32 * (0.30 + identity.unit_f32(3) * 0.16)) as i32;
    let muzzle_ry = (head_ry as f32 * (0.22 + identity.unit_f32(4) * 0.10)) as i32;

    let fur = hsl_to_color(
        18.0 + identity.unit_f32(5) * 45.0,
        0.40,
        0.55 + identity.unit_f32(6) * 0.18,
    );
    let accent = hsl_to_color(190.0 + identity.unit_f32(7) * 70.0, 0.28, 0.88);
    let ear = hsl_to_color(
        22.0 + identity.unit_f32(8) * 30.0,
        0.38,
        0.38 + identity.unit_f32(9) * 0.12,
    );
    let muzzle = hsl_to_color(32.0 + identity.unit_f32(10) * 12.0, 0.18, 0.90);
    let nose = Color::rgb(45, 36, 34);
    let eye = Color::rgb(36, 26, 20);
    let tongue = hsl_to_color(350.0 + identity.unit_f32(11) * 10.0, 0.70, 0.70);
    let spot = hsl_to_color(
        24.0 + identity.unit_f32(12) * 20.0,
        0.36,
        0.34 + identity.unit_f32(13) * 0.10,
    );
    let bg_fill = background_fill(background, accent);
    image.pixels_mut().for_each(|pixel| *pixel = bg_fill.into());
    draw_background_accent(
        &mut image, center_x, center_y, head_rx, head_ry, accent, 0.45, background, identity,
    );

    draw_filled_ellipse_mut(
        &mut image,
        (center_x - head_rx / 2, center_y - head_ry / 5),
        head_rx / 3,
        ear_drop,
        ear.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + head_rx / 2, center_y - head_ry / 5),
        head_rx / 3,
        ear_drop,
        ear.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        fur.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y + head_ry / 3),
        muzzle_rx,
        muzzle_ry,
        muzzle.into(),
    );

    if identity.byte(14).is_multiple_of(2) {
        draw_filled_ellipse_mut(
            &mut image,
            (center_x - head_rx / 3, center_y - head_ry / 8),
            head_rx / 4,
            head_ry / 3,
            Color::rgba(spot.0[0], spot.0[1], spot.0[2], 150).into(),
        );
    }

    let eye_y = center_y - head_ry / 6;
    let eye_offset = (head_rx as f32 * (0.28 + identity.unit_f32(15) * 0.12)) as i32;
    for x in [center_x - eye_offset, center_x + eye_offset] {
        draw_filled_circle_mut(
            &mut image,
            (x, eye_y),
            (head_rx as f32 * 0.08) as i32,
            Color::rgb(255, 255, 255).into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (x, eye_y),
            (head_rx as f32 * 0.04) as i32,
            eye.into(),
        );
    }

    let nose_y = center_y + head_ry / 5;
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, nose_y),
        head_rx / 7,
        head_ry / 10,
        nose.into(),
    );
    draw_line_segment_mut(
        &mut image,
        (center_x as f32, nose_y as f32),
        (center_x as f32, (nose_y + head_ry / 7) as f32),
        nose.into(),
    );
    draw_smile_arc(
        &mut image,
        center_x - head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.55,
    );
    draw_smile_arc(
        &mut image,
        center_x + head_rx / 12,
        nose_y + head_ry / 10,
        head_rx / 7,
        nose,
        0.55,
    );

    if !identity.byte(16).is_multiple_of(3) {
        draw_filled_ellipse_mut(
            &mut image,
            (center_x, nose_y + head_ry / 4),
            head_rx / 10,
            head_ry / 7,
            tongue.into(),
        );
    }

    Ok(image)
}
