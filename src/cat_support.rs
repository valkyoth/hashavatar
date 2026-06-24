
#[derive(Clone, Copy, Debug)]
struct CatPalette {
    background: Color,
    accent: Color,
    head: Color,
    ear_inner: Color,
    muzzle: Color,
    eye: Color,
    pupil: Color,
    nose: Color,
    outline: Color,
    marking: Color,
}

impl CatPalette {
    fn from_genome(genome: &CatGenome) -> Self {
        let hue = genome.base_hue;
        let head = hsl_to_color(
            hue,
            0.42 + genome.head_saturation * 0.25,
            0.55 + genome.head_lightness * 0.16,
        );
        let background = hsl_to_color(
            (hue + 180.0 + genome.background_shift * 40.0) % 360.0,
            0.25 + genome.background_sat * 0.20,
            0.90,
        );
        let accent = hsl_to_color(
            (hue + 18.0 + genome.accent_shift * 60.0) % 360.0,
            0.34 + genome.accent_sat * 0.20,
            0.80,
        );

        Self {
            background,
            accent,
            head,
            ear_inner: hsl_to_color(
                hue - 6.0,
                0.50 + genome.ear_inner_sat * 0.20,
                0.72 + genome.ear_inner_light * 0.12,
            ),
            muzzle: hsl_to_color(
                hue + 8.0,
                0.18 + genome.muzzle_sat * 0.16,
                0.84 + genome.muzzle_light * 0.10,
            ),
            eye: hsl_to_color(
                genome.eye_hue,
                0.65 + genome.eye_sat * 0.20,
                0.50 + genome.eye_light * 0.12,
            ),
            pupil: Color::rgb(28, 24, 18),
            nose: hsl_to_color(
                344.0 + genome.nose_hue * 18.0,
                0.58 + genome.nose_sat * 0.18,
                0.66 + genome.nose_light * 0.10,
            ),
            outline: Color::rgb(64, 45, 32),
            marking: hsl_to_color(
                hue + genome.marking_hue_shift * 24.0,
                0.25 + genome.marking_sat * 0.20,
                0.42 + genome.marking_light * 0.16,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CatGenome {
    base_hue: f32,
    eye_hue: f32,
    head_saturation: f32,
    head_lightness: f32,
    background_shift: f32,
    background_sat: f32,
    accent_shift: f32,
    accent_sat: f32,
    ear_inner_sat: f32,
    ear_inner_light: f32,
    muzzle_sat: f32,
    muzzle_light: f32,
    eye_sat: f32,
    eye_light: f32,
    nose_hue: f32,
    nose_sat: f32,
    nose_light: f32,
    marking_hue_shift: f32,
    marking_sat: f32,
    marking_light: f32,
    head_width: f32,
    head_height: f32,
    head_drop: f32,
    ear_width: f32,
    ear_height: f32,
    ear_tilt: f32,
    muzzle_width: f32,
    muzzle_height: f32,
    eye_spacing: f32,
    eye_width: f32,
    eye_height: f32,
    pupil_width: f32,
    whisker_len: f32,
    whisker_tilt: f32,
    smile_width: f32,
    smile_depth: f32,
    accent_band_height: f32,
    forehead_mark: f32,
    cheek_spots: f32,
    stripe_count: u8,
}

impl CatGenome {
    fn from_identity(identity: &AvatarIdentity, rng: &mut StdRng) -> Self {
        let mut noise =
            |idx: usize| (identity.unit_f32(idx) + rng.random_range(0.0..0.03)).min(1.0);
        Self {
            base_hue: 12.0 + identity.unit_f32(0) * 300.0,
            eye_hue: 45.0 + identity.unit_f32(1) * 120.0,
            head_saturation: noise(2),
            head_lightness: noise(3),
            background_shift: noise(4),
            background_sat: noise(5),
            accent_shift: noise(6),
            accent_sat: noise(7),
            ear_inner_sat: noise(8),
            ear_inner_light: noise(9),
            muzzle_sat: noise(10),
            muzzle_light: noise(11),
            eye_sat: noise(12),
            eye_light: noise(13),
            nose_hue: noise(14),
            nose_sat: noise(15),
            nose_light: noise(16),
            marking_hue_shift: identity.unit_f32(17) * 2.0 - 1.0,
            marking_sat: noise(18),
            marking_light: noise(19),
            head_width: noise(20),
            head_height: noise(21),
            head_drop: noise(22),
            ear_width: noise(23),
            ear_height: noise(24),
            ear_tilt: identity.unit_f32(25) * 2.0 - 1.0,
            muzzle_width: noise(26),
            muzzle_height: noise(27),
            eye_spacing: noise(28),
            eye_width: noise(29),
            eye_height: noise(30),
            pupil_width: noise(31),
            whisker_len: noise(32),
            whisker_tilt: identity.unit_f32(33) * 2.0 - 1.0,
            smile_width: noise(34),
            smile_depth: noise(35),
            accent_band_height: noise(36),
            forehead_mark: noise(37),
            cheek_spots: noise(38),
            stripe_count: 2 + (identity.byte(39) % 4),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct EarSpec {
    outer: [Point<i32>; 3],
    inner: [Point<i32>; 3],
}

impl EarSpec {
    fn left(
        center_x: i32,
        center_y: i32,
        head_rx: i32,
        head_ry: i32,
        ear_width: i32,
        ear_height: i32,
        ear_tilt: f32,
    ) -> Self {
        let base_x = center_x - head_rx / 2;
        let base_y = center_y - head_ry + 12;
        let tip_shift = (ear_width as f32 * 0.35 * ear_tilt) as i32;
        Self {
            outer: [
                Point::new(base_x - ear_width / 2, base_y + ear_height / 2),
                Point::new(base_x + ear_width / 3 + tip_shift, base_y - ear_height),
                Point::new(base_x + ear_width, base_y + ear_height / 3),
            ],
            inner: [
                Point::new(base_x - ear_width / 6, base_y + ear_height / 4),
                Point::new(
                    base_x + ear_width / 4 + tip_shift / 2,
                    base_y - (ear_height as f32 * 0.55) as i32,
                ),
                Point::new(
                    base_x + (ear_width as f32 * 0.6) as i32,
                    base_y + ear_height / 8,
                ),
            ],
        }
    }

    fn right(
        center_x: i32,
        center_y: i32,
        head_rx: i32,
        head_ry: i32,
        ear_width: i32,
        ear_height: i32,
        ear_tilt: f32,
    ) -> Self {
        let base_x = center_x + head_rx / 2;
        let base_y = center_y - head_ry + 12;
        let tip_shift = (ear_width as f32 * 0.35 * ear_tilt) as i32;
        Self {
            outer: [
                Point::new(base_x - ear_width, base_y + ear_height / 3),
                Point::new(base_x - ear_width / 3 - tip_shift, base_y - ear_height),
                Point::new(base_x + ear_width / 2, base_y + ear_height / 2),
            ],
            inner: [
                Point::new(
                    base_x - (ear_width as f32 * 0.6) as i32,
                    base_y + ear_height / 8,
                ),
                Point::new(
                    base_x - ear_width / 4 - tip_shift / 2,
                    base_y - (ear_height as f32 * 0.55) as i32,
                ),
                Point::new(base_x + ear_width / 6, base_y + ear_height / 4),
            ],
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_background_accent(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    accent: Color,
    accent_band_height: f32,
    background: AvatarBackground,
    identity: &AvatarIdentity,
) {
    if background != AvatarBackground::Themed {
        draw_decorative_background(image, background, accent, identity);
        return;
    }
    let width = image.width() as i32;
    let stripe_top = center_y - head_ry - 18;
    let stripe_height = ((head_ry as f32) * (0.25 + accent_band_height * 0.45)) as i32;

    draw_filled_rect_mut(
        image,
        Rect::at(0, stripe_top.max(0)).of_size(width as u32, stripe_height.max(1) as u32),
        accent.into(),
    );
    draw_filled_circle_mut(
        image,
        (center_x + head_rx / 2, center_y - head_ry / 2),
        head_ry / 3,
        Color::rgba(accent.0[0], accent.0[1], accent.0[2], 180).into(),
    );
}

fn draw_ear(
    image: &mut RgbaImage,
    spec: EarSpec,
    outer_color: Color,
    inner_color: Color,
    outline: Color,
) {
    draw_polygon_mut(image, &spec.outer, outer_color.into());
    draw_polygon_mut(image, &spec.inner, inner_color.into());

    for edge in spec.outer.windows(2) {
        draw_antialiased_line_segment_mut(
            image,
            (edge[0].x, edge[0].y),
            (edge[1].x, edge[1].y),
            outline.into(),
            interpolate,
        );
    }
    draw_antialiased_line_segment_mut(
        image,
        (spec.outer[2].x, spec.outer[2].y),
        (spec.outer[0].x, spec.outer[0].y),
        outline.into(),
        interpolate,
    );
}

fn draw_eyes(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    palette: CatPalette,
    genome: CatGenome,
) {
    let eye_offset_x = (head_rx as f32 * (0.31 + genome.eye_spacing * 0.18)) as i32;
    let eye_y = center_y - head_ry / 6;
    let eye_rx = (head_rx as f32 * (0.12 + genome.eye_width * 0.10)) as i32;
    let eye_ry = (head_ry as f32 * (0.11 + genome.eye_height * 0.10)) as i32;
    let pupil_ry = (eye_ry as f32 * 0.90) as i32;
    let pupil_rx = ((eye_rx as f32) * (0.12 + genome.pupil_width * 0.18)) as i32;

    for eye_x in [center_x - eye_offset_x, center_x + eye_offset_x] {
        draw_filled_ellipse_mut(image, (eye_x, eye_y), eye_rx, eye_ry, palette.eye.into());
        draw_filled_ellipse_mut(
            image,
            (eye_x, eye_y),
            pupil_rx,
            pupil_ry,
            palette.pupil.into(),
        );
        draw_filled_circle_mut(
            image,
            (eye_x - eye_rx / 3, eye_y - eye_ry / 3),
            (eye_rx as f32 * 0.15) as i32,
            Color::rgba(255, 255, 255, 220).into(),
        );
    }
}

fn draw_nose_and_mouth(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    palette: CatPalette,
    genome: CatGenome,
) {
    let nose_y = center_y + head_ry / 7;
    let nose_half_width = (head_rx as f32 * (0.08 + genome.muzzle_width * 0.05)) as i32;
    let nose_height = (head_ry as f32 * (0.08 + genome.muzzle_height * 0.05)) as i32;
    let nose = [
        Point::new(center_x - nose_half_width, nose_y),
        Point::new(center_x + nose_half_width, nose_y),
        Point::new(center_x, nose_y + nose_height),
    ];
    draw_polygon_mut(image, &nose, palette.nose.into());

    let mouth_top = nose_y + nose_height;
    draw_line_segment_mut(
        image,
        (center_x as f32, mouth_top as f32),
        (center_x as f32, (mouth_top + head_ry / 8) as f32),
        palette.outline.into(),
    );

    let smile_radius = (head_rx as f32 * (0.08 + genome.smile_width * 0.10)) as i32;
    draw_smile_arc(
        image,
        center_x - smile_radius,
        mouth_top + smile_radius / 2,
        smile_radius,
        palette.outline,
        genome.smile_depth,
    );
    draw_smile_arc(
        image,
        center_x + smile_radius,
        mouth_top + smile_radius / 2,
        smile_radius,
        palette.outline,
        genome.smile_depth,
    );
}

fn draw_smile_arc(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: Color,
    smile_depth: f32,
) {
    for step in 20..=160 {
        let theta = (step as f32).to_radians();
        let x = center_x as f32 + theta.cos() * radius as f32 * 0.55;
        let y = center_y as f32 + theta.sin() * radius as f32 * (0.24 + smile_depth * 0.28);
        draw_filled_circle_mut(image, (x.round() as i32, y.round() as i32), 1, color.into());
    }
}

fn draw_whiskers(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    color: Color,
    genome: CatGenome,
) {
    let muzzle_y = center_y + head_ry / 5;
    let left_start = center_x - head_rx / 6;
    let right_start = center_x + head_rx / 6;
    let whisker_len = (head_rx as f32 * (0.58 + genome.whisker_len * 0.42)) as i32;
    let whisker_slope = (genome.whisker_tilt * 12.0) as i32;

    for offset in [-12, 0, 12] {
        draw_antialiased_line_segment_mut(
            image,
            (left_start, muzzle_y + offset),
            (
                left_start - whisker_len,
                muzzle_y + offset - 8 + whisker_slope,
            ),
            color.into(),
            interpolate,
        );
        draw_antialiased_line_segment_mut(
            image,
            (right_start, muzzle_y + offset),
            (
                right_start + whisker_len,
                muzzle_y + offset - 8 - whisker_slope,
            ),
            color.into(),
            interpolate,
        );
    }
}

fn draw_cat_markings(
    image: &mut RgbaImage,
    center_x: i32,
    center_y: i32,
    head_rx: i32,
    head_ry: i32,
    color: Color,
    genome: CatGenome,
) {
    let stripe_count = genome.stripe_count as i32;
    let forehead_y = center_y - head_ry / 2;
    let stripe_spacing = (head_rx / 5).max(6);
    let stripe_length = ((head_ry as f32) * (0.14 + genome.forehead_mark * 0.12)) as i32;

    for stripe in 0..stripe_count {
        let offset = stripe - stripe_count / 2;
        let x = center_x + offset * stripe_spacing / 2;
        draw_line_segment_mut(
            image,
            (x as f32, forehead_y as f32),
            ((x + offset * 2) as f32, (forehead_y + stripe_length) as f32),
            color.into(),
        );
    }

    if genome.cheek_spots > 0.35 {
        let cheek_y = center_y + head_ry / 5;
        let cheek_x = (head_rx as f32 * 0.55) as i32;
        let cheek_radius = ((head_rx as f32) * (0.05 + genome.cheek_spots * 0.04)) as i32;
        draw_filled_circle_mut(
            image,
            (center_x - cheek_x, cheek_y),
            cheek_radius,
            Color::rgba(color.0[0], color.0[1], color.0[2], 120).into(),
        );
        draw_filled_circle_mut(
            image,
            (center_x + cheek_x, cheek_y),
            cheek_radius,
            Color::rgba(color.0[0], color.0[1], color.0[2], 120).into(),
        );
    }
}

fn hsl_to_color(hue: f32, saturation: f32, lightness: f32) -> Color {
    let rgb_u8: Srgb<u8> = Srgb::from_color(Hsl::new(hue, saturation, lightness)).into_format();
    Color::rgb(rgb_u8.red, rgb_u8.green, rgb_u8.blue)
}

fn background_fill(background: AvatarBackground, themed: Color) -> Color {
    match background {
        AvatarBackground::Themed => themed,
        AvatarBackground::White => Color::rgb(255, 255, 255),
        AvatarBackground::Black => Color::rgb(0, 0, 0),
        AvatarBackground::Dark => Color::rgb(17, 24, 39),
        AvatarBackground::Light => Color::rgb(248, 250, 247),
        AvatarBackground::Transparent => Color::rgba(255, 255, 255, 0),
        AvatarBackground::PolkaDot
        | AvatarBackground::Striped
        | AvatarBackground::Checkerboard
        | AvatarBackground::Grid => Color::rgb(248, 250, 247),
        AvatarBackground::Sunrise => Color::rgb(255, 244, 214),
        AvatarBackground::Ocean => Color::rgb(221, 246, 252),
        AvatarBackground::Starry => Color::rgb(17, 24, 39),
    }
}
