use super::*;

pub fn render_paws_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());
    let bg = hsl_to_color(24.0 + identity.unit_f32(0) * 36.0, 0.20, 0.94);
    let fur = hsl_to_color(
        identity.unit_f32(1) * 360.0,
        0.32 + identity.unit_f32(2) * 0.18,
        0.60,
    );
    let pad = hsl_to_color(
        330.0 + identity.unit_f32(3) * 20.0,
        0.36 + identity.unit_f32(4) * 0.18,
        0.72,
    );
    let accent = hsl_to_color(18.0 + identity.unit_f32(5) * 24.0, 0.34, 0.82);
    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, bg).into());

    if background == AvatarBackground::Themed {
        for stripe in 0..4 {
            let y = (height / 8) + stripe * (height / 5);
            draw_filled_rect_mut(
                &mut image,
                Rect::at(0, y).of_size(spec.width, (height / 18).max(1) as u32),
                Color::rgba(accent.0[0], accent.0[1], accent.0[2], 70).into(),
            );
        }
    } else {
        draw_decorative_background(&mut image, background, accent, identity);
    }

    let primary_x = width / 2;
    let primary_y = height / 2 + height / 12;
    let palm_rx = (width as f32 * (0.14 + identity.unit_f32(6) * 0.04)) as i32;
    let palm_ry = (height as f32 * (0.16 + identity.unit_f32(7) * 0.04)) as i32;
    draw_paw_print(
        &mut image,
        primary_x,
        primary_y,
        palm_rx,
        palm_ry,
        fur,
        pad,
        identity.byte(8),
    );

    if identity.byte(9).is_multiple_of(2) {
        draw_paw_print(
            &mut image,
            width / 3,
            height / 3,
            (palm_rx as f32 * 0.82) as i32,
            (palm_ry as f32 * 0.82) as i32,
            hsl_to_color(identity.unit_f32(10) * 360.0, 0.28, 0.66),
            pad,
            identity.byte(11),
        );
    }

    if !identity.byte(12).is_multiple_of(3) {
        draw_paw_print(
            &mut image,
            width * 2 / 3,
            height / 3 + height / 8,
            (palm_rx as f32 * 0.70) as i32,
            (palm_ry as f32 * 0.70) as i32,
            fur,
            hsl_to_color(340.0 + identity.unit_f32(13) * 12.0, 0.30, 0.80),
            identity.byte(14),
        );
    }

    Ok(image)
}

#[allow(clippy::too_many_arguments)]
fn draw_paw_print(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    palm_rx: i32,
    palm_ry: i32,
    fur: Color,
    pad: Color,
    shape_seed: u8,
) {
    let toe_offset_y = palm_ry;
    let toe_spacing = (palm_rx as f32 * (0.48 + (shape_seed as f32 / 255.0) * 0.12)) as i32;
    let toe_rx = (palm_rx as f32 * (0.26 + (shape_seed as f32 / 255.0) * 0.04)) as i32;
    let toe_ry = (palm_ry as f32 * (0.24 + ((shape_seed >> 2) as f32 / 255.0) * 0.06)) as i32;

    draw_filled_ellipse_mut(image, (center_x, center_y), palm_rx, palm_ry, fur.into());
    draw_filled_ellipse_mut(
        image,
        (center_x, center_y + palm_ry / 8),
        (palm_rx as f32 * 0.72) as i32,
        (palm_ry as f32 * 0.68) as i32,
        pad.into(),
    );

    for (index, offset) in [-3, -1, 1, 3].into_iter().enumerate() {
        let x = center_x + offset * toe_spacing / 4;
        let y = center_y - toe_offset_y + if index % 2 == 0 { 0 } else { toe_ry / 3 };
        draw_filled_ellipse_mut(image, (x, y), toe_rx, toe_ry, fur.into());
        draw_filled_ellipse_mut(
            image,
            (x, y + toe_ry / 5),
            (toe_rx as f32 * 0.68) as i32,
            (toe_ry as f32 * 0.68) as i32,
            pad.into(),
        );
    }
}
