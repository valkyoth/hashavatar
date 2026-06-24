use super::*;

pub fn render_wizard_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let head_rx = (width as f32 * (0.16 + identity.unit_f32(15) * 0.06)) as i32;
    let head_ry = (height as f32 * (0.16 + identity.unit_f32(16) * 0.06)) as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * (0.57 + identity.unit_f32(17) * 0.07)) as i32,
        head_rx,
        head_ry,
    };
    let bg = hsl_to_color(
        220.0 + identity.unit_f32(10) * 85.0,
        0.20 + identity.unit_f32(18) * 0.12,
        0.90 + identity.unit_f32(19) * 0.04,
    );
    let hat = hsl_to_color(
        210.0 + identity.unit_f32(11) * 110.0,
        0.34 + identity.unit_f32(20) * 0.22,
        0.28 + identity.unit_f32(21) * 0.16,
    );
    let hat_band = hsl_to_color(
        24.0 + identity.unit_f32(12) * 160.0,
        0.62 + identity.unit_f32(22) * 0.24,
        0.48 + identity.unit_f32(23) * 0.18,
    );
    let skin = hsl_to_color(
        18.0 + identity.unit_f32(13) * 28.0,
        0.22 + identity.unit_f32(24) * 0.20,
        0.74 + identity.unit_f32(25) * 0.12,
    );
    let beard = hsl_to_color(
        35.0 + identity.unit_f32(14) * 45.0,
        0.06 + identity.unit_f32(26) * 0.12,
        0.80 + identity.unit_f32(27) * 0.16,
    );
    let hat_width = (layout.head_rx as f32 * (1.0 + identity.unit_f32(28) * 0.55)) as i32;
    let hat_height = (layout.head_ry as f32 * (1.7 + identity.unit_f32(29) * 0.75)) as i32;
    let tip_shift = (layout.head_rx as f32 * (identity.unit_f32(30) - 0.5) * 0.8) as i32;
    let brim_width = (layout.head_rx as f32 * (2.45 + identity.unit_f32(31) * 0.75)) as i32;
    let brim_height = (layout.head_ry as f32 * (0.22 + identity.unit_f32(32) * 0.16)) as i32;
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, bg).into(),
    );
    draw_background_accent(
        &mut image,
        layout.center_x,
        layout.center_y,
        layout.head_rx,
        layout.head_ry,
        hat_band,
        0.20,
        background,
        identity,
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(
                layout.center_x - hat_width,
                layout.center_y - layout.head_ry / 2,
            ),
            Point::new(
                layout.center_x + hat_width,
                layout.center_y - layout.head_ry / 2,
            ),
            Point::new(
                layout.center_x + tip_shift,
                layout.center_y - layout.head_ry / 2 - hat_height,
            ),
        ],
        hat.into(),
    );
    draw_filled_rect_mut(
        &mut image,
        Rect::at(
            layout.center_x - brim_width / 2,
            layout.center_y - layout.head_ry / 2,
        )
        .of_size(brim_width.max(2) as u32, brim_height.max(2) as u32),
        hat_band.into(),
    );
    let star_count = 1 + (identity.byte(33) % 4) as i32;
    for star in 0..star_count {
        let sx = layout.center_x - hat_width / 2
            + (identity.byte(34 + star as usize) as i32 % hat_width.max(1));
        let sy = layout.center_y - layout.head_ry / 2 - hat_height / 2
            + (identity.byte(39 + star as usize) as i32 % (hat_height / 2).max(1));
        draw_filled_circle_mut(
            &mut image,
            (sx, sy),
            (layout.head_rx / 12).max(2),
            Color::rgba(hat_band.0[0], hat_band.0[1], hat_band.0[2], 210).into(),
        );
    }
    draw_filled_circle_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        skin.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(
                layout.center_x
                    - layout.head_rx / 2
                    - (identity.byte(44) as i32 % layout.head_rx.max(1)) / 6,
                layout.center_y + layout.head_ry / 3,
            ),
            Point::new(
                layout.center_x
                    + layout.head_rx / 2
                    + (identity.byte(45) as i32 % layout.head_rx.max(1)) / 6,
                layout.center_y + layout.head_ry / 3,
            ),
            Point::new(
                layout.center_x + (identity.unit_f32(46) * layout.head_rx as f32 * 0.4) as i32
                    - layout.head_rx / 5,
                layout.center_y
                    + layout.head_ry
                    + (layout.head_ry as f32 * (0.35 + identity.unit_f32(47) * 0.55)) as i32,
            ),
        ],
        beard.into(),
    );
    draw_creature_eyes(
        &mut image,
        layout,
        if identity.byte(48).is_multiple_of(7) {
            1
        } else {
            2
        },
        if identity.byte(49).is_multiple_of(2) {
            CreatureEyeStyle::Round
        } else {
            CreatureEyeStyle::Tall
        },
        Color::rgb(255, 255, 255),
        Color::rgb(36, 30, 52),
    );
    draw_creature_mouth(
        &mut image,
        layout,
        CreatureMouthStyle::Smile,
        Color::rgb(86, 64, 58),
    );
    draw_filled_circle_mut(
        &mut image,
        (
            layout.center_x + tip_shift + layout.head_rx / 2,
            layout.center_y - layout.head_ry / 2 - hat_height,
        ),
        (layout.head_rx / 6).max(3),
        hat_band.into(),
    );
    Ok(image)
}
