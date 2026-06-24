pub fn render_cat_avatar(spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    let seed = spec.seed.to_le_bytes();
    let identity = AvatarIdentity::new_unchecked(AvatarIdentityOptions::default(), &seed);
    Ok(render_cat_avatar_with_identity(
        spec,
        &identity,
        AvatarBackground::Themed,
    ))
}

/// Render a cat face avatar from a stable identity digest.
pub fn render_cat_avatar_for_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    Ok(render_cat_avatar_with_identity(
        spec,
        identity,
        AvatarBackground::Themed,
    ))
}

pub fn render_cat_avatar_for_identity_with_background(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> Result<RgbaImage, AvatarSpecError> {
    spec.validate()?;
    Ok(render_cat_avatar_with_identity(spec, identity, background))
}

fn render_cat_avatar_with_identity(
    spec: AvatarSpec,
    identity: &AvatarIdentity,
    background: AvatarBackground,
) -> RgbaImage {
    let mut rng = seeded_renderer_rng(spec, identity);
    let genome = CatGenome::from_identity(identity, &mut rng);
    let palette = CatPalette::from_genome(&genome);
    let mut image = ImageBuffer::from_pixel(
        spec.width,
        spec.height,
        background_fill(background, palette.background).into(),
    );

    let width = spec.width as i32;
    let height = spec.height as i32;
    let center_x = width / 2;
    let center_y = ((height as f32) * (0.53 + genome.head_drop * 0.08)) as i32;

    let head_rx = ((width as f32) * (0.26 + genome.head_width * 0.07)) as i32;
    let head_ry = ((height as f32) * (0.22 + genome.head_height * 0.08)) as i32;
    let ear_height = ((height as f32) * (0.15 + genome.ear_height * 0.08)) as i32;
    let ear_width = ((width as f32) * (0.12 + genome.ear_width * 0.08)) as i32;

    draw_background_accent(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.accent,
        genome.accent_band_height,
        background,
        identity,
    );
    draw_ear(
        &mut image,
        EarSpec::left(
            center_x,
            center_y,
            head_rx,
            head_ry,
            ear_width,
            ear_height,
            genome.ear_tilt,
        ),
        palette.head,
        palette.ear_inner,
        palette.outline,
    );
    draw_ear(
        &mut image,
        EarSpec::right(
            center_x,
            center_y,
            head_rx,
            head_ry,
            ear_width,
            ear_height,
            genome.ear_tilt,
        ),
        palette.head,
        palette.ear_inner,
        palette.outline,
    );

    draw_filled_ellipse_mut(
        &mut image,
        (center_x, center_y),
        head_rx,
        head_ry,
        palette.head.into(),
    );
    draw_hollow_circle_mut(
        &mut image,
        (center_x, center_y),
        head_rx.min(head_ry),
        palette.outline.into(),
    );

    let muzzle_center = (center_x, center_y + head_ry / 4);
    draw_filled_ellipse_mut(
        &mut image,
        muzzle_center,
        (head_rx as f32 * (0.40 + genome.muzzle_width * 0.18)) as i32,
        (head_ry as f32 * (0.24 + genome.muzzle_height * 0.14)) as i32,
        palette.muzzle.into(),
    );

    draw_eyes(
        &mut image, center_x, center_y, head_rx, head_ry, palette, genome,
    );
    draw_nose_and_mouth(
        &mut image, center_x, center_y, head_rx, head_ry, palette, genome,
    );
    draw_whiskers(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.outline,
        genome,
    );
    draw_cat_markings(
        &mut image,
        center_x,
        center_y,
        head_rx,
        head_ry,
        palette.marking,
        genome,
    );

    image
}

