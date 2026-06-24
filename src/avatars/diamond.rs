use super::*;

pub fn render_diamond_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.56) as i32;
    let rx = (width as f32 * 0.25) as i32;
    let ry = (height as f32 * 0.30) as i32;
    let gem = hsl_to_color(180.0 + identity.unit_f32(1) * 95.0, 0.55, 0.60);
    let highlight = hsl_to_color(190.0 + identity.unit_f32(2) * 70.0, 0.40, 0.82);
    let shade = hsl_to_color(200.0 + identity.unit_f32(3) * 70.0, 0.42, 0.42);
    let bg = hsl_to_color(200.0 + identity.unit_f32(4) * 50.0, 0.18, 0.94);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image, center_x, center_y, rx, ry, highlight, 0.32, background, identity,
    );

    let outer = [
        Point::new(center_x - rx, center_y - ry / 3),
        Point::new(center_x - rx / 2, center_y - ry),
        Point::new(center_x + rx / 2, center_y - ry),
        Point::new(center_x + rx, center_y - ry / 3),
        Point::new(center_x, center_y + ry),
    ];
    draw_polygon_mut(&mut image, &outer, gem.into());
    draw_polygon_mut(
        &mut image,
        &[
            outer[0],
            outer[1],
            Point::new(center_x, center_y + ry),
            Point::new(center_x - rx / 5, center_y - ry / 3),
        ],
        highlight.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            outer[2],
            outer[3],
            Point::new(center_x, center_y + ry),
            Point::new(center_x + rx / 5, center_y - ry / 3),
        ],
        shade.into(),
    );
    for x in [center_x - rx / 2, center_x, center_x + rx / 2] {
        draw_line_segment_mut(
            &mut image,
            (x as f32, (center_y - ry) as f32),
            (center_x as f32, (center_y + ry) as f32),
            Color::rgba(255, 255, 255, 110).into(),
        );
    }

    Ok(image)
}
