pub fn render_monster_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = (height as f32 * 0.58) as i32;
    let mut image =
        ImageBuffer::from_pixel(spec.width, spec.height, Color::rgb(255, 255, 255).into());

    let skin = hsl_to_color(
        identity.unit_f32(0) * 360.0,
        0.48 + identity.unit_f32(1) * 0.24,
        0.46 + identity.unit_f32(2) * 0.20,
    );
    let shade = hsl_to_color(
        identity.unit_f32(3) * 360.0,
        0.38 + identity.unit_f32(4) * 0.18,
        0.24 + identity.unit_f32(5) * 0.10,
    );
    let accent = hsl_to_color(
        20.0 + identity.unit_f32(6) * 320.0,
        0.34 + identity.unit_f32(7) * 0.26,
        0.86,
    );
    let mouth = Color::rgb(48, 18, 24);
    let eye_white = Color::rgb(252, 248, 236);
    let pupil = Color::rgb(24, 20, 28);

    image
        .pixels_mut()
        .for_each(|pixel| *pixel = background_fill(background, accent).into());

    let head_rx = (width as f32 * (0.23 + identity.unit_f32(8) * 0.10)) as i32;
    let head_ry = (height as f32 * (0.22 + identity.unit_f32(9) * 0.11)) as i32;
    let horn_height = (height as f32 * (0.08 + identity.unit_f32(10) * 0.10)) as i32;
    let horn_width = (width as f32 * (0.06 + identity.unit_f32(11) * 0.05)) as i32;
    let eye_count = 1 + (identity.byte(12) % 3) as usize;
    let mouth_style = identity.byte(13) % 3;
    let body_style = identity.byte(14) % 3;
    let spot_count = 3 + (identity.byte(15) % 5) as i32;
    let tentacle_count = 2 + (identity.byte(16) % 4) as i32;

    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        shade,
        0.30 + identity.unit_f32(17) * 0.25,
        background,
        identity,
    );

    match body_style {
        0 => draw_filled_ellipse_mut(
            &mut image,
            (center_x, center_y),
            head_rx,
            head_ry,
            skin.into(),
        ),
        1 => {
            draw_filled_ellipse_mut(
                &mut image,
                (center_x, center_y + head_ry / 8),
                head_rx,
                head_ry - head_ry / 8,
                skin.into(),
            );
            draw_polygon_mut(
                &mut image,
                &[
                    Point::new(center_x - head_rx, center_y),
                    Point::new(center_x, center_y - head_ry),
                    Point::new(center_x + head_rx, center_y),
                    Point::new(center_x + head_rx / 2, center_y + head_ry),
                    Point::new(center_x - head_rx / 2, center_y + head_ry),
                ],
                skin.into(),
            );
        }
        _ => {
            draw_filled_rect_mut(
                &mut image,
                Rect::at(center_x - head_rx, center_y - head_ry)
                    .of_size((head_rx * 2) as u32, (head_ry * 2) as u32),
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x - head_rx, center_y - head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x + head_rx, center_y - head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x - head_rx, center_y + head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
            draw_filled_circle_mut(
                &mut image,
                (center_x + head_rx, center_y + head_ry / 2),
                head_ry / 2,
                skin.into(),
            );
        }
    }

    if identity.byte(18).is_multiple_of(2) {
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x - head_rx / 2, center_y - head_ry),
                Point::new(
                    center_x - head_rx / 3 - horn_width,
                    center_y - head_ry - horn_height,
                ),
                Point::new(center_x - head_rx / 8, center_y - head_ry / 2),
            ],
            shade.into(),
        );
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(center_x + head_rx / 2, center_y - head_ry),
                Point::new(
                    center_x + head_rx / 3 + horn_width,
                    center_y - head_ry - horn_height,
                ),
                Point::new(center_x + head_rx / 8, center_y - head_ry / 2),
            ],
            shade.into(),
        );
    } else {
        for spike in 0..3 {
            let spike_x = center_x - head_rx / 2 + spike * head_rx / 2;
            draw_polygon_mut(
                &mut image,
                &[
                    Point::new(spike_x - horn_width / 2, center_y - head_ry / 2),
                    Point::new(spike_x, center_y - head_ry - horn_height / 2),
                    Point::new(spike_x + horn_width / 2, center_y - head_ry / 2),
                ],
                shade.into(),
            );
        }
    }

    for index in 0..spot_count {
        let x = center_x - head_rx / 2 + (index * head_rx) / spot_count;
        let y = center_y - head_ry / 3 + ((index * 37 + identity.byte(19) as i32) % head_ry.max(1));
        let radius =
            (head_rx as f32 * (0.05 + ((index + 1) as f32 / spot_count as f32) * 0.06)) as i32;
        draw_filled_circle_mut(
            &mut image,
            (x, y),
            radius.max(3),
            Color::rgba(shade.0[0], shade.0[1], shade.0[2], 168).into(),
        );
    }

    let eye_y = center_y - head_ry / 5;
    let eye_rx = (head_rx as f32 * (0.10 + identity.unit_f32(20) * 0.08)) as i32;
    let eye_ry = (head_ry as f32 * (0.10 + identity.unit_f32(21) * 0.10)) as i32;
    let eye_spacing = if eye_count == 1 {
        0
    } else {
        (head_rx as f32 * 0.46 / (eye_count - 1) as f32) as i32
    };
    let eye_start = center_x - eye_spacing * ((eye_count.saturating_sub(1)) as i32) / 2;
    for index in 0..eye_count {
        let x = eye_start + eye_spacing * index as i32;
        draw_filled_ellipse_mut(&mut image, (x, eye_y), eye_rx, eye_ry, eye_white.into());
        if identity.byte(22).is_multiple_of(2) {
            draw_filled_ellipse_mut(
                &mut image,
                (x, eye_y),
                (eye_rx / 3).max(2),
                (eye_ry - 1).max(2),
                pupil.into(),
            );
        } else {
            draw_filled_circle_mut(&mut image, (x, eye_y), (eye_ry / 2).max(2), pupil.into());
        }
        draw_filled_circle_mut(
            &mut image,
            (x - eye_rx / 3, eye_y - eye_ry / 3),
            (eye_rx / 5).max(1),
            Color::rgba(255, 255, 255, 220).into(),
        );
    }

    let mouth_y = center_y + head_ry / 3;
    match mouth_style {
        0 => {
            draw_filled_ellipse_mut(
                &mut image,
                (center_x, mouth_y),
                head_rx / 3,
                head_ry / 8,
                mouth.into(),
            );
            for fang_x in [center_x - head_rx / 8, center_x + head_rx / 8] {
                draw_polygon_mut(
                    &mut image,
                    &[
                        Point::new(fang_x - head_rx / 24, mouth_y - 2),
                        Point::new(fang_x + head_rx / 24, mouth_y - 2),
                        Point::new(fang_x, mouth_y + head_ry / 5),
                    ],
                    eye_white.into(),
                );
            }
        }
        1 => {
            draw_smile_arc(
                &mut image,
                center_x - head_rx / 10,
                mouth_y,
                head_rx / 4,
                mouth,
                0.50,
            );
            draw_smile_arc(
                &mut image,
                center_x + head_rx / 10,
                mouth_y,
                head_rx / 4,
                mouth,
                0.50,
            );
            draw_line_segment_mut(
                &mut image,
                ((center_x - head_rx / 4) as f32, mouth_y as f32),
                ((center_x + head_rx / 4) as f32, mouth_y as f32),
                mouth.into(),
            );
        }
        _ => {
            draw_filled_rect_mut(
                &mut image,
                Rect::at(center_x - head_rx / 3, mouth_y - head_ry / 10)
                    .of_size((head_rx * 2 / 3) as u32, (head_ry / 5).max(1) as u32),
                mouth.into(),
            );
            for tooth in 0..4 {
                let tooth_x = center_x - head_rx / 4 + tooth * head_rx / 6;
                draw_polygon_mut(
                    &mut image,
                    &[
                        Point::new(tooth_x - head_rx / 30, mouth_y - head_ry / 10),
                        Point::new(tooth_x + head_rx / 30, mouth_y - head_ry / 10),
                        Point::new(tooth_x, mouth_y + head_ry / 14),
                    ],
                    eye_white.into(),
                );
            }
        }
    }

    if identity.byte(23).is_multiple_of(2) {
        for index in 0..tentacle_count {
            let start_x = center_x - head_rx / 2 + (index * head_rx) / tentacle_count;
            let start_y = center_y + head_ry - 4;
            let end_x = start_x + ((index % 2) * 2 - 1) * head_rx / 6;
            let end_y = start_y + head_ry / 2;
            draw_antialiased_line_segment_mut(
                &mut image,
                (start_x, start_y),
                (end_x, end_y),
                shade.into(),
                interpolate,
            );
            draw_filled_circle_mut(
                &mut image,
                (end_x, end_y),
                (head_rx / 18).max(2),
                shade.into(),
            );
        }
    }

    Ok(image)
}

#[derive(Clone, Copy)]
struct FaceLayout {
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
}

#[derive(Clone, Copy)]
enum CreatureEyeStyle {
    Round,
    Tall,
    Hollow,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum CreatureMouthStyle {
    Smile,
    Fang,
    Flat,
}

fn draw_creature_eyes(
    image: &mut RgbaImage,
    layout: FaceLayout,
    count: usize,
    style: CreatureEyeStyle,
    eye_white: Color,
    pupil: Color,
) {
    let spacing = if count <= 1 {
        0
    } else {
        (layout.head_rx as f32 * 0.48 / (count - 1) as f32) as i32
    };
    let start_x = layout.center_x - spacing * (count.saturating_sub(1) as i32) / 2;
    let eye_y = layout.center_y - layout.head_ry / 5;
    let eye_rx = (layout.head_rx as f32 * 0.12) as i32;
    let eye_ry = (layout.head_ry as f32 * 0.12) as i32;

    for index in 0..count {
        let x = start_x + spacing * index as i32;
        match style {
            CreatureEyeStyle::Round => {
                draw_filled_circle_mut(image, (x, eye_y), eye_rx.max(3), eye_white.into());
                draw_filled_circle_mut(image, (x, eye_y), (eye_rx / 2).max(2), pupil.into());
            }
            CreatureEyeStyle::Tall => {
                draw_filled_ellipse_mut(image, (x, eye_y), eye_rx, eye_ry + 4, eye_white.into());
                draw_filled_ellipse_mut(
                    image,
                    (x, eye_y),
                    (eye_rx / 3).max(2),
                    (eye_ry + 2).max(2),
                    pupil.into(),
                );
            }
            CreatureEyeStyle::Hollow => {
                draw_filled_ellipse_mut(image, (x, eye_y), eye_rx + 3, eye_ry + 5, pupil.into());
                draw_filled_ellipse_mut(
                    image,
                    (x, eye_y + 1),
                    (eye_rx / 2).max(2),
                    (eye_ry / 2).max(2),
                    Color::rgba(255, 255, 255, 20).into(),
                );
            }
        }
    }
}

fn draw_creature_mouth(
    image: &mut RgbaImage,
    layout: FaceLayout,
    style: CreatureMouthStyle,
    color: Color,
) {
    let mouth_y = layout.center_y + layout.head_ry / 3;
    match style {
        CreatureMouthStyle::Smile => {
            draw_smile_arc(
                image,
                layout.center_x - layout.head_rx / 10,
                mouth_y,
                layout.head_rx / 4,
                color,
                0.45,
            );
            draw_smile_arc(
                image,
                layout.center_x + layout.head_rx / 10,
                mouth_y,
                layout.head_rx / 4,
                color,
                0.45,
            );
        }
        CreatureMouthStyle::Fang => {
            draw_filled_ellipse_mut(
                image,
                (layout.center_x, mouth_y),
                layout.head_rx / 3,
                layout.head_ry / 8,
                color.into(),
            );
            for tooth_x in [
                layout.center_x - layout.head_rx / 7,
                layout.center_x + layout.head_rx / 7,
            ] {
                draw_polygon_mut(
                    image,
                    &[
                        Point::new(tooth_x - layout.head_rx / 26, mouth_y - 1),
                        Point::new(tooth_x + layout.head_rx / 26, mouth_y - 1),
                        Point::new(tooth_x, mouth_y + layout.head_ry / 5),
                    ],
                    Color::rgb(248, 246, 238).into(),
                );
            }
        }
        CreatureMouthStyle::Flat => {
            draw_filled_rect_mut(
                image,
                Rect::at(layout.center_x - layout.head_rx / 3, mouth_y - 3)
                    .of_size((layout.head_rx * 2 / 3) as u32, 6),
                color.into(),
            );
        }
    }
}
