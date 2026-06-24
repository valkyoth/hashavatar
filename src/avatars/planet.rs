use super::*;

/// Render a ringed planet avatar from a stable identity.
pub fn render_planet_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * (0.53 + identity.unit_f32(8) * 0.06)) as i32;
    let bg = hsl_to_color(215.0 + identity.unit_f32(0) * 90.0, 0.24, 0.91);
    let planet = hsl_to_color(identity.unit_f32(1) * 360.0, 0.46, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.42);
    let ring = hsl_to_color(32.0 + identity.unit_f32(3) * 120.0, 0.44, 0.72);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );

    if background == AvatarBackground::Themed {
        let star_count = 3 + (identity.byte(4) % 4) as i32;
        for star in 0..star_count {
            let x = width / 8 + (identity.byte(5 + star as usize) as i32 % (width * 3 / 4).max(1));
            let y =
                height / 8 + (identity.byte(10 + star as usize) as i32 % (height * 3 / 4).max(1));
            draw_filled_circle_mut(
                &mut image,
                (x, y),
                (width as f32 * (0.010 + identity.unit_f32(15 + star as usize) * 0.010)) as i32,
                Color::rgba(255, 255, 255, 170).into(),
            );
        }
    } else {
        draw_decorative_background(&mut image, background, ring, identity);
    }

    let radius = (width.min(height) as f32 * (0.18 + identity.unit_f32(20) * 0.08)) as i32;
    let ring_rx = (radius as f32 * (1.55 + identity.unit_f32(21) * 0.28)) as i32;
    let ring_ry = (radius as f32 * (0.38 + identity.unit_f32(22) * 0.12)) as i32;
    let bg_fill = background_fill(background, bg);
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        ring_rx,
        ring_ry,
        ring.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        (ring_rx as f32 * 0.84) as i32,
        (ring_ry as f32 * 0.58) as i32,
        bg_fill.into(),
    );
    draw_filled_circle_mut(&mut image, (center_x, center_y), radius, planet.into());
    draw_filled_ellipse_mut(
        &mut image,
        (center_x - radius / 4, center_y - radius / 5),
        radius / 2,
        radius / 5,
        Color::rgba(shade.0[0], shade.0[1], shade.0[2], 120).into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (center_x + radius / 4, center_y + radius / 5),
        radius / 2,
        radius / 6,
        Color::rgba(255, 255, 255, 80).into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(center_x - ring_rx, center_y - ring_ry / 5)
            .of_size((ring_rx * 2) as u32, (ring_ry / 3).max(1) as u32),
        Color::rgba(ring.0[0], ring.0[1], ring.0[2], 190).into(),
    );
    Ok(image)
}
