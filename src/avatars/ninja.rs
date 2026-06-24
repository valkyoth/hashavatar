use super::*;

pub fn render_ninja_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let head_r = (width.min(height) as f32 * 0.28) as i32;
    let cloth = hsl_to_color(220.0 + identity.unit_f32(1) * 50.0, 0.18, 0.14);
    let skin = hsl_to_color(28.0 + identity.unit_f32(2) * 18.0, 0.42, 0.72);
    let band = hsl_to_color(identity.unit_f32(3) * 360.0, 0.56, 0.50);
    let bg = hsl_to_color(225.0 + identity.unit_f32(4) * 30.0, 0.13, 0.92);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, head_r, head_r, band, 0.38, background, identity,
    );

    draw_filled_circle_mut(&mut image, (center_x, center_y), head_r, cloth.into());
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - head_r * 3 / 5, center_y - head_r / 4)
            .of_size((head_r * 6 / 5) as u32, (head_r / 2) as u32),
        skin.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - head_r, center_y - head_r * 2 / 3)
            .of_size((head_r * 2) as u32, (head_r / 6).max(2) as u32),
        band.into(),
    );
    for x in [center_x - head_r / 3, center_x + head_r / 3] {
        draw_filled_ellipse_mut(
            &mut image,
            (x, center_y - head_r / 12),
            head_r / 9,
            head_r / 14,
            Color::rgb(20, 24, 32).into(),
        );
    }
    let tie = [
        Point::new(center_x + head_r * 4 / 5, center_y - head_r * 2 / 3),
        Point::new(center_x + head_r * 7 / 5, center_y - head_r),
        Point::new(center_x + head_r, center_y - head_r / 3),
    ];
    draw_polygon_mut(&mut image, &tie, band.into());

    Ok(image)
}
