use super::*;

/// Render an ice cream cone avatar from a stable identity.
pub fn render_icecream_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.55) as i32;
    let bg = hsl_to_color(190.0 + identity.unit_f32(0) * 95.0, 0.18, 0.94);
    let scoop = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.76);
    let cone = hsl_to_color(32.0 + identity.unit_f32(2) * 22.0, 0.50, 0.64);
    let waffle = hsl_to_color(28.0 + identity.unit_f32(3) * 22.0, 0.42, 0.45);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    let scoop_r = (width as f32 * (0.18 + identity.unit_f32(4) * 0.06)) as i32;
    let cone_w = (width as f32 * (0.24 + identity.unit_f32(5) * 0.05)) as i32;
    let cone_h = (height as f32 * (0.32 + identity.unit_f32(6) * 0.06)) as i32;
    let scoop_y = center_y - scoop_r / 2;
    let cone_top_y = scoop_y + scoop_r / 2;
    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        scoop_r,
        cone_h / 2,
        waffle,
        0.18,
        background,
        identity,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(center_x - cone_w / 2, cone_top_y),
            Point::new(center_x + cone_w / 2, cone_top_y),
            Point::new(center_x, cone_top_y + cone_h),
        ],
        cone.into(),
    );
    for line in [-1, 1] {
        draw_line_segment_mut(
            &mut image,
            (
                (center_x + line * cone_w / 3) as f32,
                (cone_top_y + cone_h / 8) as f32,
            ),
            (center_x as f32, (cone_top_y + cone_h * 3 / 4) as f32),
            waffle.into(),
        );
    }
    draw_filled_circle_mut(&mut image, (center_x, scoop_y), scoop_r, scoop.into());
    if identity.byte(7).is_multiple_of(2) {
        draw_filled_circle_mut(
            &mut image,
            (center_x - scoop_r / 2, scoop_y + scoop_r / 3),
            scoop_r / 5,
            scoop.into(),
        );
        draw_filled_circle_mut(
            &mut image,
            (center_x + scoop_r / 3, scoop_y + scoop_r / 2),
            scoop_r / 6,
            scoop.into(),
        );
    }
    let chip_count = 2 + (identity.byte(8) % 4) as i32;
    for chip in 0..chip_count {
        let x = center_x - scoop_r / 2 + (identity.byte(9 + chip as usize) as i32 % scoop_r.max(1));
        let y = scoop_y - scoop_r / 3 + (identity.byte(14 + chip as usize) as i32 % scoop_r.max(1));
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            (width as f32 * 0.010).max(2.0) as i32,
            waffle.into(),
        );
    }
    Ok(image)
}
