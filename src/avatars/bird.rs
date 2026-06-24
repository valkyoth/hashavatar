pub fn render_bird_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let width = spec.width as i32;
    let height = spec.height as i32;
    let layout = FaceLayout {
        center_x: width / 2,
        center_y: (height as f32 * 0.56) as i32,
        head_rx: (width as f32 * 0.22) as i32,
        head_ry: (height as f32 * 0.22) as i32,
    };
    let bg = hsl_to_color(190.0 + identity.unit_f32(6) * 60.0, 0.18, 0.93);
    let plumage = hsl_to_color(identity.unit_f32(7) * 360.0, 0.42, 0.62);
    let wing = hsl_to_color(20.0 + identity.unit_f32(8) * 160.0, 0.32, 0.46);
    let beak = hsl_to_color(32.0 + identity.unit_f32(9) * 26.0, 0.82, 0.58);
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
        wing,
        0.24,
        background,
        identity,
    );
    draw_filled_circle_mut(
        &mut image,
        (layout.center_x, layout.center_y),
        layout.head_rx,
        plumage.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (
            layout.center_x - layout.head_rx / 2,
            layout.center_y + layout.head_ry / 6,
        ),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_filled_ellipse_mut(
        &mut image,
        (
            layout.center_x + layout.head_rx / 2,
            layout.center_y + layout.head_ry / 6,
        ),
        layout.head_rx / 3,
        layout.head_ry / 2,
        wing.into(),
    );
    draw_polygon_mut(
        &mut image,
        &[
            Point::new(layout.center_x, layout.center_y),
            Point::new(
                layout.center_x + layout.head_rx / 2,
                layout.center_y + layout.head_ry / 6,
            ),
            Point::new(layout.center_x, layout.center_y + layout.head_ry / 3),
        ],
        beak.into(),
    );
    for feather in 0..3 {
        let fx = layout.center_x - layout.head_rx / 5 + feather * layout.head_rx / 5;
        draw_polygon_mut(
            &mut image,
            &[
                Point::new(fx, layout.center_y - layout.head_ry),
                Point::new(
                    fx + layout.head_rx / 10,
                    layout.center_y - layout.head_ry - layout.head_ry / 2,
                ),
                Point::new(
                    fx + layout.head_rx / 5,
                    layout.center_y - layout.head_ry / 2,
                ),
            ],
            wing.into(),
        );
    }
    draw_creature_eyes(
        &mut image,
        layout,
        2,
        CreatureEyeStyle::Round,
        Color::rgb(255, 255, 255),
        Color::rgb(28, 24, 34),
    );
    Ok(image)
}

