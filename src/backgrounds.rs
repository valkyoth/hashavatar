use super::*;

pub(crate) fn draw_decorative_background(
    image: &mut RgbaImage,
    background: AvatarBackground,
    accent: Color,
    identity: &AvatarIdentity,
) {
    match background {
        AvatarBackground::PolkaDot => draw_polka_dot_background(image, accent),
        AvatarBackground::Striped => draw_striped_background(image, accent),
        AvatarBackground::Checkerboard => draw_checkerboard_background(image),
        AvatarBackground::Grid => draw_grid_background(image),
        AvatarBackground::Sunrise => draw_vertical_gradient_background(
            image,
            Color::rgb(255, 247, 212),
            Color::rgb(255, 184, 107),
        ),
        AvatarBackground::Ocean => draw_vertical_gradient_background(
            image,
            Color::rgb(220, 248, 252),
            Color::rgb(75, 145, 190),
        ),
        AvatarBackground::Starry => draw_starry_background(image, identity),
        AvatarBackground::Themed
        | AvatarBackground::White
        | AvatarBackground::Black
        | AvatarBackground::Dark
        | AvatarBackground::Light
        | AvatarBackground::Transparent => {}
    }
}

pub(crate) fn draw_polka_dot_background(image: &mut RgbaImage, accent: Color) {
    let base = Color::rgb(248, 250, 247);
    let dot = rgba_over(base, Color::rgba(accent.0[0], accent.0[1], accent.0[2], 62));
    fill_image(image, base);

    let min_side = image.width().min(image.height()).max(1);
    let step = (min_side / 8).clamp(8, 44) as i32;
    let radius = (step / 5).max(1);
    for y in (step / 2..image.height() as i32).step_by(step as usize) {
        for x in (step / 2..image.width() as i32).step_by(step as usize) {
            draw_filled_circle_mut(image, (x, y), radius, dot.into());
        }
    }
}

pub(crate) fn draw_striped_background(image: &mut RgbaImage, accent: Color) {
    let base = Color::rgb(248, 250, 247);
    let stripe = rgba_over(base, Color::rgba(accent.0[0], accent.0[1], accent.0[2], 42));
    let min_side = image.width().min(image.height()).max(1);
    let width = (min_side / 10).clamp(6, 36);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let band = ((x + y) / width).is_multiple_of(2);
            image.put_pixel(x, y, if band { stripe.into() } else { base.into() });
        }
    }
}

pub(crate) fn draw_checkerboard_background(image: &mut RgbaImage) {
    let light = Color::rgb(248, 250, 247);
    let dark = Color::rgb(232, 236, 231);
    let min_side = image.width().min(image.height()).max(1);
    let tile = (min_side / 8).clamp(8, 48);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let even = ((x / tile) + (y / tile)).is_multiple_of(2);
            image.put_pixel(x, y, if even { light.into() } else { dark.into() });
        }
    }
}

pub(crate) fn draw_grid_background(image: &mut RgbaImage) {
    let base = Color::rgb(248, 250, 247);
    let line = Color::rgb(221, 226, 221);
    let min_side = image.width().min(image.height()).max(1);
    let step = (min_side / 8).clamp(8, 48);

    for y in 0..image.height() {
        for x in 0..image.width() {
            let grid_line = x.is_multiple_of(step) || y.is_multiple_of(step);
            image.put_pixel(x, y, if grid_line { line.into() } else { base.into() });
        }
    }
}

pub(crate) fn draw_vertical_gradient_background(image: &mut RgbaImage, top: Color, bottom: Color) {
    let max_y = image.height().saturating_sub(1).max(1);
    for y in 0..image.height() {
        let color = lerp_color_u32(top, bottom, y, max_y);
        for x in 0..image.width() {
            image.put_pixel(x, y, color.into());
        }
    }
}

pub(crate) fn draw_starry_background(image: &mut RgbaImage, identity: &AvatarIdentity) {
    let base = Color::rgb(17, 24, 39);
    fill_image(image, base);

    let min_side = image.width().min(image.height()).max(1);
    let star_count = (min_side / 7).clamp(10, 180);
    let mut state = 0x9e37_79b9_u32
        ^ image.width().wrapping_mul(0x85eb_ca6b)
        ^ image.height().wrapping_mul(0xc2b2_ae35)
        ^ u32::from_le_bytes([
            identity.byte(40),
            identity.byte(41),
            identity.byte(42),
            identity.byte(43),
        ])
        ^ (u32::from_le_bytes([
            identity.byte(44),
            identity.byte(45),
            identity.byte(46),
            identity.byte(47),
        ])
        .rotate_left(13));
    for index in 0..star_count {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let x = state % image.width().max(1);
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let y = state % image.height().max(1);
        let radius = if index.is_multiple_of(7) { 2 } else { 1 };
        draw_filled_circle_mut(
            image,
            (x as i32, y as i32),
            radius,
            Rgba([255, 255, 255, 170]),
        );
    }
}

pub(crate) fn fill_image(image: &mut RgbaImage, color: Color) {
    for pixel in image.pixels_mut() {
        *pixel = color.into();
    }
}

pub(crate) fn rgba_over(bottom: Color, top: Color) -> Color {
    let alpha = u32::from(top.0[3]);
    let inverse = 255 - alpha;
    Color::rgb(
        ((u32::from(top.0[0]) * alpha + u32::from(bottom.0[0]) * inverse + 127) / 255) as u8,
        ((u32::from(top.0[1]) * alpha + u32::from(bottom.0[1]) * inverse + 127) / 255) as u8,
        ((u32::from(top.0[2]) * alpha + u32::from(bottom.0[2]) * inverse + 127) / 255) as u8,
    )
}

pub(crate) fn lerp_color_u32(start: Color, end: Color, position: u32, max_position: u32) -> Color {
    let max = max_position.max(1);
    Color::rgb(
        lerp_channel_u32(start.0[0], end.0[0], position, max),
        lerp_channel_u32(start.0[1], end.0[1], position, max),
        lerp_channel_u32(start.0[2], end.0[2], position, max),
    )
}

pub(crate) fn lerp_channel_u32(start: u8, end: u8, position: u32, max_position: u32) -> u8 {
    let start = u64::from(start);
    let end = u64::from(end);
    let max = u64::from(max_position.max(1));
    let position = u64::from(position).min(max);
    let value = (start * (max - position) + end * position + max / 2) / max;
    value.min(u64::from(u8::MAX)) as u8
}

pub(crate) fn color_hex(color: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0[0], color.0[1], color.0[2])
}
