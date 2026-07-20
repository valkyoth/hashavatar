use super::*;

pub(crate) fn apply_style_layers(
    image: &mut RgbaImage,
    spec: AvatarSpec,
    style: AvatarStyleOptions,
    identity: &AvatarIdentity,
) {
    if !style.has_extra_layers() {
        return;
    }

    let accent = style_accent_color(style.color, identity);
    if style.color != AvatarColor::Default {
        draw_style_color_layer(image, spec, accent);
    }
    draw_expression_layer(image, spec, style.kind, style.expression, accent);
    draw_accessory_layer(image, spec, style.kind, style.accessory, accent);
    apply_shape_layer(image, spec, style.shape, accent);
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AvatarLayerAnchors {
    left_eye: (f32, f32),
    right_eye: (f32, f32),
    mouth: (f32, f32),
    top: f32,
    neck: f32,
    face_width: f32,
    eye_radius: f32,
}

impl AvatarLayerAnchors {
    fn point(self, spec: AvatarSpec, point: (f32, f32)) -> (i32, i32) {
        (
            (point.0 * spec.width as f32).round() as i32,
            (point.1 * spec.height as f32).round() as i32,
        )
    }

    fn y(self, spec: AvatarSpec, value: f32) -> i32 {
        (value * spec.height as f32).round() as i32
    }

    fn span(self, spec: AvatarSpec, value: f32) -> i32 {
        (value * spec.width.min(spec.height) as f32).round() as i32
    }
}

pub(crate) fn avatar_layer_anchors(kind: AvatarKind) -> Option<AvatarLayerAnchors> {
    let anchors = match kind {
        AvatarKind::Cat => AvatarLayerAnchors {
            left_eye: (0.40, 0.50),
            right_eye: (0.60, 0.50),
            mouth: (0.50, 0.62),
            top: 0.28,
            neck: 0.72,
            face_width: 0.58,
            eye_radius: 0.055,
        },
        AvatarKind::Dog => AvatarLayerAnchors {
            left_eye: (0.40, 0.46),
            right_eye: (0.60, 0.46),
            mouth: (0.50, 0.61),
            top: 0.24,
            neck: 0.72,
            face_width: 0.62,
            eye_radius: 0.055,
        },
        AvatarKind::Robot => AvatarLayerAnchors {
            left_eye: (0.38, 0.45),
            right_eye: (0.62, 0.45),
            mouth: (0.50, 0.60),
            top: 0.25,
            neck: 0.72,
            face_width: 0.54,
            eye_radius: 0.060,
        },
        AvatarKind::Fox => AvatarLayerAnchors {
            left_eye: (0.40, 0.48),
            right_eye: (0.60, 0.48),
            mouth: (0.50, 0.62),
            top: 0.25,
            neck: 0.72,
            face_width: 0.60,
            eye_radius: 0.050,
        },
        AvatarKind::Alien => AvatarLayerAnchors {
            left_eye: (0.39, 0.48),
            right_eye: (0.61, 0.48),
            mouth: (0.50, 0.63),
            top: 0.25,
            neck: 0.72,
            face_width: 0.58,
            eye_radius: 0.070,
        },
        AvatarKind::Monster => AvatarLayerAnchors {
            left_eye: (0.45, 0.52),
            right_eye: (0.55, 0.52),
            mouth: (0.50, 0.66),
            top: 0.27,
            neck: 0.74,
            face_width: 0.62,
            eye_radius: 0.045,
        },
        AvatarKind::Ghost => AvatarLayerAnchors {
            left_eye: (0.44, 0.50),
            right_eye: (0.56, 0.50),
            mouth: (0.50, 0.61),
            top: 0.27,
            neck: 0.72,
            face_width: 0.54,
            eye_radius: 0.045,
        },
        AvatarKind::Slime => AvatarLayerAnchors {
            left_eye: (0.40, 0.48),
            right_eye: (0.60, 0.48),
            mouth: (0.50, 0.61),
            top: 0.32,
            neck: 0.73,
            face_width: 0.58,
            eye_radius: 0.055,
        },
        AvatarKind::Bird => AvatarLayerAnchors {
            left_eye: (0.445, 0.51),
            right_eye: (0.555, 0.51),
            mouth: (0.50, 0.61),
            top: 0.31,
            neck: 0.70,
            face_width: 0.50,
            eye_radius: 0.035,
        },
        AvatarKind::Wizard => AvatarLayerAnchors {
            left_eye: (0.455, 0.55),
            right_eye: (0.545, 0.55),
            mouth: (0.50, 0.67),
            top: 0.36,
            neck: 0.73,
            face_width: 0.44,
            eye_radius: 0.040,
        },
        AvatarKind::Skull => AvatarLayerAnchors {
            left_eye: (0.40, 0.45),
            right_eye: (0.60, 0.45),
            mouth: (0.50, 0.63),
            top: 0.24,
            neck: 0.72,
            face_width: 0.55,
            eye_radius: 0.065,
        },
        AvatarKind::Frog => AvatarLayerAnchors {
            left_eye: (0.37, 0.39),
            right_eye: (0.63, 0.39),
            mouth: (0.50, 0.62),
            top: 0.25,
            neck: 0.72,
            face_width: 0.64,
            eye_radius: 0.070,
        },
        AvatarKind::Panda => AvatarLayerAnchors {
            left_eye: (0.41, 0.53),
            right_eye: (0.59, 0.53),
            mouth: (0.50, 0.62),
            top: 0.28,
            neck: 0.72,
            face_width: 0.62,
            eye_radius: 0.040,
        },
        AvatarKind::Octopus => AvatarLayerAnchors {
            left_eye: (0.42, 0.52),
            right_eye: (0.58, 0.52),
            mouth: (0.50, 0.65),
            top: 0.28,
            neck: 0.70,
            face_width: 0.58,
            eye_radius: 0.045,
        },
        AvatarKind::Knight => AvatarLayerAnchors {
            left_eye: (0.40, 0.45),
            right_eye: (0.60, 0.45),
            mouth: (0.50, 0.60),
            top: 0.22,
            neck: 0.72,
            face_width: 0.58,
            eye_radius: 0.050,
        },
        AvatarKind::Bear => AvatarLayerAnchors {
            left_eye: (0.40, 0.48),
            right_eye: (0.60, 0.48),
            mouth: (0.50, 0.63),
            top: 0.25,
            neck: 0.73,
            face_width: 0.62,
            eye_radius: 0.055,
        },
        AvatarKind::Penguin => AvatarLayerAnchors {
            left_eye: (0.40, 0.45),
            right_eye: (0.60, 0.45),
            mouth: (0.50, 0.57),
            top: 0.20,
            neck: 0.74,
            face_width: 0.58,
            eye_radius: 0.045,
        },
        AvatarKind::Dragon => AvatarLayerAnchors {
            left_eye: (0.40, 0.48),
            right_eye: (0.60, 0.48),
            mouth: (0.50, 0.63),
            top: 0.23,
            neck: 0.73,
            face_width: 0.62,
            eye_radius: 0.052,
        },
        AvatarKind::Ninja => AvatarLayerAnchors {
            left_eye: (0.40, 0.48),
            right_eye: (0.60, 0.48),
            mouth: (0.50, 0.64),
            top: 0.24,
            neck: 0.73,
            face_width: 0.58,
            eye_radius: 0.045,
        },
        AvatarKind::Astronaut => AvatarLayerAnchors {
            left_eye: (0.40, 0.47),
            right_eye: (0.60, 0.47),
            mouth: (0.50, 0.62),
            top: 0.19,
            neck: 0.76,
            face_width: 0.60,
            eye_radius: 0.045,
        },
        AvatarKind::Paws
        | AvatarKind::Planet
        | AvatarKind::Rocket
        | AvatarKind::Mushroom
        | AvatarKind::Cactus
        | AvatarKind::Cupcake
        | AvatarKind::Pizza
        | AvatarKind::Icecream
        | AvatarKind::Diamond
        | AvatarKind::CoffeeCup
        | AvatarKind::Shield => return None,
    };
    Some(anchors)
}

pub(crate) fn glasses_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Dog | AvatarKind::Robot | AvatarKind::Ghost => 0.025,
        AvatarKind::Monster => 0.020,
        AvatarKind::Wizard | AvatarKind::Knight => 0.035,
        _ => 0.0,
    }
}

pub(crate) fn hat_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Cat | AvatarKind::Frog => -0.035,
        _ => 0.0,
    }
}

pub(crate) fn crown_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Cat => -0.035,
        AvatarKind::Alien | AvatarKind::Frog => -0.080,
        _ => 0.0,
    }
}

pub(crate) fn bowtie_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Cat
        | AvatarKind::Fox
        | AvatarKind::Slime
        | AvatarKind::Wizard
        | AvatarKind::Octopus => 0.060,
        _ => 0.0,
    }
}

pub(crate) fn eyepatch_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Knight => 0.030,
        _ => 0.0,
    }
}

pub(crate) fn horns_y_offset(kind: AvatarKind) -> f32 {
    match kind {
        AvatarKind::Dog | AvatarKind::Robot => 0.070,
        _ => 0.0,
    }
}

pub(crate) fn style_accent_color(color: AvatarColor, identity: &AvatarIdentity) -> Color {
    match color {
        AvatarColor::Default => hsl_to_color(180.0 + identity.unit_f32(25) * 160.0, 0.58, 0.48),
        AvatarColor::NeonMint => Color::rgb(27, 235, 179),
        AvatarColor::PastelPink => Color::rgb(246, 160, 196),
        AvatarColor::Crimson => Color::rgb(190, 18, 60),
        AvatarColor::Gold => Color::rgb(234, 179, 8),
        AvatarColor::DeepSeaBlue => Color::rgb(14, 116, 144),
    }
}

pub(crate) fn rgba_with_alpha(color: Color, alpha: u8) -> Rgba<u8> {
    Rgba([color.0[0], color.0[1], color.0[2], alpha])
}

pub(crate) fn blend_pixel(image: &mut RgbaImage, x: i32, y: i32, source: Rgba<u8>) {
    if !in_bounds(image, x, y) {
        return;
    }

    let destination = *image.get_pixel(x as u32, y as u32);
    let alpha = u32::from(source.0[3]);
    let inverse = 255 - alpha;
    let blended = Rgba([
        ((u32::from(source.0[0]) * alpha + u32::from(destination.0[0]) * inverse + 127) / 255)
            as u8,
        ((u32::from(source.0[1]) * alpha + u32::from(destination.0[1]) * inverse + 127) / 255)
            as u8,
        ((u32::from(source.0[2]) * alpha + u32::from(destination.0[2]) * inverse + 127) / 255)
            as u8,
        source.0[3].saturating_add(((u32::from(destination.0[3]) * inverse + 127) / 255) as u8),
    ]);
    image.put_pixel(x as u32, y as u32, blended);
}

pub(crate) fn draw_blended_rect_mut(image: &mut RgbaImage, rect: Rect, color: Rgba<u8>) {
    if image.width() == 0 || image.height() == 0 {
        return;
    }

    let bounds = Rect::at(0, 0).of_size(image.width(), image.height());
    if let Some(intersection) = bounds.intersect(rect) {
        for dy in 0..intersection.height() {
            for dx in 0..intersection.width() {
                let x = intersection.left() + dx as i32;
                let y = intersection.top() + dy as i32;
                blend_pixel(image, x, y, color);
            }
        }
    }
}

pub(crate) fn draw_style_color_layer(image: &mut RgbaImage, spec: AvatarSpec, accent: Color) {
    let width = spec.width as i32;
    let height = spec.height as i32;
    let bar_height = ((height as f32 * 0.08) as u32).max(3);
    draw_blended_rect_mut(
        image,
        Rect::at(0, height - bar_height as i32).of_size(spec.width, bar_height),
        rgba_with_alpha(accent, 210),
    );

    let stripe = ((spec.width.min(spec.height) as f32 * 0.03) as u32).max(2);
    draw_blended_rect_mut(
        image,
        Rect::at(0, 0).of_size(stripe, spec.height),
        rgba_with_alpha(accent, 145),
    );
    draw_blended_rect_mut(
        image,
        Rect::at(width - stripe as i32, 0).of_size(stripe, spec.height),
        rgba_with_alpha(accent, 145),
    );
}

pub(crate) fn draw_accessory_layer(
    image: &mut RgbaImage,
    spec: AvatarSpec,
    kind: AvatarKind,
    accessory: AvatarAccessory,
    accent: Color,
) {
    if accessory == AvatarAccessory::None {
        return;
    }

    let Some(anchors) = avatar_layer_anchors(kind) else {
        return;
    };

    let min = spec.width.min(spec.height) as i32;
    let (left_eye_x, left_eye_y) = anchors.point(spec, anchors.left_eye);
    let (right_eye_x, right_eye_y) = anchors.point(spec, anchors.right_eye);
    let (mouth_x, mouth_y) = anchors.point(spec, anchors.mouth);
    let eye_y = (left_eye_y + right_eye_y) / 2;
    let top_y = anchors.y(spec, anchors.top);
    let neck_y = anchors.y(spec, anchors.neck);
    let face_half = anchors.span(spec, anchors.face_width) / 2;
    let eye_radius = anchors.span(spec, anchors.eye_radius).max(3);
    let dark = Rgba([31, 41, 55, 255]);
    let light = rgba_with_alpha(accent, 255);

    match accessory {
        AvatarAccessory::None => {}
        AvatarAccessory::Glasses => {
            let radius = (eye_radius * 2).max(5);
            let glasses_offset = (glasses_y_offset(kind) * spec.height as f32).round() as i32;
            let left_glasses_y = left_eye_y + glasses_offset;
            let right_glasses_y = right_eye_y + glasses_offset;
            let bridge_y = (left_glasses_y + right_glasses_y) / 2;
            draw_hollow_circle_mut(image, (left_eye_x, left_glasses_y), radius, dark);
            draw_hollow_circle_mut(image, (right_eye_x, right_glasses_y), radius, dark);
            draw_line_segment_mut(
                image,
                ((left_eye_x + radius) as f32, bridge_y as f32),
                ((right_eye_x - radius) as f32, bridge_y as f32),
                dark,
            );
        }
        AvatarAccessory::Hat => {
            let brim_w = (face_half * 14 / 10).max(min * 20 / 100);
            let brim_h = (min * 5 / 100).max(2);
            let top_w = (brim_w * 7 / 10).max(min * 16 / 100);
            let top_h = min * 18 / 100;
            let y_offset = (hat_y_offset(kind) * spec.height as f32).round() as i32;
            let y = (top_y - top_h / 2 + y_offset).max(0);
            draw_filled_rect_mut(
                image,
                Rect::at(mouth_x - top_w / 2, y).of_size(top_w as u32, top_h as u32),
                light,
            );
            draw_filled_rect_mut(
                image,
                Rect::at(mouth_x - brim_w / 2, y + top_h).of_size(brim_w as u32, brim_h as u32),
                dark,
            );
        }
        AvatarAccessory::Headphones => {
            let side_r = (eye_radius * 2).max(5);
            let left_x = mouth_x - face_half;
            let right_x = mouth_x + face_half;
            draw_top_hollow_ellipse_mut(
                image,
                (mouth_x, eye_y),
                face_half,
                (eye_y - top_y).max(min * 8 / 100),
                dark,
            );
            draw_filled_circle_mut(image, (left_x, eye_y), side_r, light);
            draw_filled_circle_mut(image, (right_x, eye_y), side_r, light);
            draw_hollow_circle_mut(image, (left_x, eye_y), side_r, dark);
            draw_hollow_circle_mut(image, (right_x, eye_y), side_r, dark);
        }
        AvatarAccessory::Crown => {
            let y_offset = (crown_y_offset(kind) * spec.height as f32).round() as i32;
            let base_y = (top_y + y_offset).max(0);
            let points = [
                Point::new(mouth_x - face_half * 7 / 10, base_y + min * 12 / 100),
                Point::new(mouth_x - face_half * 45 / 100, base_y),
                Point::new(mouth_x - face_half * 16 / 100, base_y + min * 10 / 100),
                Point::new(mouth_x, (base_y - min * 4 / 100).max(0)),
                Point::new(mouth_x + face_half * 16 / 100, base_y + min * 10 / 100),
                Point::new(mouth_x + face_half * 45 / 100, base_y),
                Point::new(mouth_x + face_half * 7 / 10, base_y + min * 12 / 100),
            ];
            draw_polygon_mut(image, &points, light);
            draw_line_segment_mut(
                image,
                (
                    (mouth_x - face_half * 7 / 10) as f32,
                    (base_y + min * 12 / 100) as f32,
                ),
                (
                    (mouth_x + face_half * 7 / 10) as f32,
                    (base_y + min * 12 / 100) as f32,
                ),
                dark,
            );
        }
        AvatarAccessory::Bowtie => {
            let y_offset = (bowtie_y_offset(kind) * spec.height as f32).round() as i32;
            let y = neck_y + y_offset;
            let size = min * 10 / 100;
            let left = [
                Point::new(mouth_x, y),
                Point::new(mouth_x - size * 2, y - size),
                Point::new(mouth_x - size * 2, y + size),
            ];
            let right = [
                Point::new(mouth_x, y),
                Point::new(mouth_x + size * 2, y - size),
                Point::new(mouth_x + size * 2, y + size),
            ];
            draw_polygon_mut(image, &left, light);
            draw_polygon_mut(image, &right, light);
            draw_filled_circle_mut(image, (mouth_x, y), (size / 2).max(2), dark);
        }
        AvatarAccessory::Eyepatch => {
            let patch_rx = (eye_radius * 2).max(5);
            let patch_ry = (eye_radius * 3 / 2).max(4);
            let y_offset = (eyepatch_y_offset(kind) * spec.height as f32).round() as i32;
            let left_patch_y = left_eye_y + y_offset;
            let strap_start_y = top_y + y_offset / 2;
            let strap_end_y = mouth_y - eye_radius + y_offset;
            draw_line_segment_mut(
                image,
                ((mouth_x - face_half) as f32, strap_start_y as f32),
                ((mouth_x + face_half * 7 / 10) as f32, strap_end_y as f32),
                dark,
            );
            draw_filled_ellipse_mut(image, (left_eye_x, left_patch_y), patch_rx, patch_ry, dark);
        }
        AvatarAccessory::Scarf => {
            let scarf_h = (min * 8 / 100).max(4);
            let y = neck_y;
            draw_filled_rect_mut(
                image,
                Rect::at(mouth_x - face_half * 8 / 10, y)
                    .of_size((face_half * 16 / 10) as u32, scarf_h as u32),
                light,
            );
            draw_filled_rect_mut(
                image,
                Rect::at(mouth_x + face_half / 5, y + scarf_h / 2)
                    .of_size((min * 9 / 100) as u32, (min * 20 / 100) as u32),
                light,
            );
        }
        AvatarAccessory::Halo => {
            draw_hollow_ellipse_mut(
                image,
                (mouth_x, (top_y - min * 7 / 100).max(0)),
                (face_half * 7 / 10).max(min * 10 / 100),
                min * 7 / 100,
                light,
            );
        }
        AvatarAccessory::Horns => {
            let y_offset = (horns_y_offset(kind) * spec.height as f32).round() as i32;
            let y = top_y + min * 5 / 100 + y_offset;
            let left = [
                Point::new(mouth_x - face_half * 6 / 10, y + min * 7 / 100),
                Point::new(mouth_x - face_half, (y - min * 12 / 100).max(0)),
                Point::new(mouth_x - face_half * 3 / 10, y + min * 2 / 100),
            ];
            let right = [
                Point::new(mouth_x + face_half * 6 / 10, y + min * 7 / 100),
                Point::new(mouth_x + face_half, (y - min * 12 / 100).max(0)),
                Point::new(mouth_x + face_half * 3 / 10, y + min * 2 / 100),
            ];
            draw_polygon_mut(image, &left, light);
            draw_polygon_mut(image, &right, light);
        }
    }
}

pub(crate) fn draw_expression_layer(
    image: &mut RgbaImage,
    spec: AvatarSpec,
    kind: AvatarKind,
    expression: AvatarExpression,
    accent: Color,
) {
    if expression == AvatarExpression::Default {
        return;
    }

    let Some(anchors) = avatar_layer_anchors(kind) else {
        return;
    };

    let min = spec.width.min(spec.height) as i32;
    let (left_eye_x, left_eye_y) = anchors.point(spec, anchors.left_eye);
    let (right_eye_x, right_eye_y) = anchors.point(spec, anchors.right_eye);
    let (mouth_x, mouth_y) = anchors.point(spec, anchors.mouth);
    let eye_radius = anchors.span(spec, anchors.eye_radius).max(3);
    let dark = Rgba([17, 24, 39, 255]);
    let accent = rgba_with_alpha(accent, 255);

    match expression {
        AvatarExpression::Default => {}
        AvatarExpression::Happy => {
            draw_smile_curve(
                image,
                mouth_x,
                mouth_y,
                min * 14 / 100,
                min * 8 / 100,
                false,
                dark,
            );
        }
        AvatarExpression::Grumpy => {
            draw_smile_curve(
                image,
                mouth_x,
                mouth_y + min * 7 / 100,
                min * 14 / 100,
                min * 8 / 100,
                true,
                dark,
            );
        }
        AvatarExpression::Surprised => {
            draw_hollow_circle_mut(image, (mouth_x, mouth_y), (min * 7 / 100).max(3), dark);
        }
        AvatarExpression::Sleepy => {
            draw_line_segment_mut(
                image,
                ((left_eye_x - eye_radius) as f32, left_eye_y as f32),
                ((left_eye_x + eye_radius) as f32, left_eye_y as f32),
                dark,
            );
            draw_line_segment_mut(
                image,
                ((right_eye_x - eye_radius) as f32, right_eye_y as f32),
                ((right_eye_x + eye_radius) as f32, right_eye_y as f32),
                dark,
            );
        }
        AvatarExpression::Winking => {
            draw_line_segment_mut(
                image,
                ((left_eye_x - eye_radius) as f32, left_eye_y as f32),
                ((left_eye_x + eye_radius) as f32, left_eye_y as f32),
                dark,
            );
            draw_filled_circle_mut(
                image,
                (right_eye_x, right_eye_y),
                (eye_radius * 7 / 10).max(2),
                dark,
            );
        }
        AvatarExpression::Cool => {
            let rect_h = (eye_radius * 2).max(4) as u32;
            let left = left_eye_x.min(right_eye_x) - eye_radius * 2;
            let width = (right_eye_x.max(left_eye_x) - left + eye_radius * 2).max(min * 24 / 100);
            draw_filled_rect_mut(
                image,
                Rect::at(left, left_eye_y.min(right_eye_y) - eye_radius)
                    .of_size(width as u32, rect_h),
                dark,
            );
            draw_blended_rect_mut(
                image,
                Rect::at(
                    left + eye_radius,
                    left_eye_y.min(right_eye_y) - eye_radius / 2,
                )
                .of_size((width / 3).max(2) as u32, (eye_radius / 2).max(2) as u32),
                Rgba([255, 255, 255, 70]),
            );
        }
        AvatarExpression::Crying => {
            draw_smile_curve(
                image,
                mouth_x,
                mouth_y + min * 7 / 100,
                min * 12 / 100,
                min * 7 / 100,
                true,
                dark,
            );
            draw_filled_ellipse_mut(
                image,
                (right_eye_x + eye_radius, right_eye_y + eye_radius * 2),
                (eye_radius * 7 / 10).max(2),
                (eye_radius * 3 / 2).max(4),
                accent,
            );
        }
    }
}

pub(crate) fn draw_smile_curve(
    image: &mut RgbaImage,
    cx: i32,
    cy: i32,
    width: i32,
    height: i32,
    inverted: bool,
    color: Rgba<u8>,
) {
    let mut previous = None;
    for step in 0..=16 {
        let t = step as f32 / 16.0;
        let x = cx - width + (2.0 * width as f32 * t) as i32;
        let curve = (1.0 - (2.0 * t - 1.0).powi(2)) * height as f32;
        let y = if inverted {
            cy - curve as i32
        } else {
            cy + curve as i32
        };
        if let Some((px, py)) = previous {
            draw_antialiased_line_segment_mut(image, (px, py), (x, y), color, interpolate);
        }
        previous = Some((x, y));
    }
}

pub(crate) fn draw_hollow_ellipse_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
    color: Rgba<u8>,
) {
    let (cx, cy) = center;
    let steps = (width_radius.max(height_radius) * 8).max(24);
    let mut previous = None;
    for step in 0..=steps {
        let angle = step as f32 / steps as f32 * std::f32::consts::TAU;
        let x = cx + (angle.cos() * width_radius as f32).round() as i32;
        let y = cy + (angle.sin() * height_radius as f32).round() as i32;
        if let Some((px, py)) = previous {
            draw_antialiased_line_segment_mut(image, (px, py), (x, y), color, interpolate);
        }
        previous = Some((x, y));
    }
}

pub(crate) fn draw_top_hollow_ellipse_mut(
    image: &mut RgbaImage,
    center: (i32, i32),
    width_radius: i32,
    height_radius: i32,
    color: Rgba<u8>,
) {
    let (cx, cy) = center;
    let steps = (width_radius.max(height_radius) * 4).max(16);
    let mut previous = None;
    for step in 0..=steps {
        let angle = std::f32::consts::PI + step as f32 / steps as f32 * std::f32::consts::PI;
        let x = cx + (angle.cos() * width_radius as f32).round() as i32;
        let y = cy + (angle.sin() * height_radius as f32).round() as i32;
        if let Some((px, py)) = previous {
            draw_antialiased_line_segment_mut(image, (px, py), (x, y), color, interpolate);
        }
        previous = Some((x, y));
    }
}

pub(crate) fn apply_shape_layer(
    image: &mut RgbaImage,
    spec: AvatarSpec,
    shape: AvatarShape,
    accent: Color,
) {
    if shape == AvatarShape::Square {
        return;
    }

    for y in 0..spec.height {
        for x in 0..spec.width {
            if !point_inside_avatar_shape(x as i32, y as i32, spec, shape) {
                image.put_pixel(x, y, Rgba([255, 255, 255, 0]));
            }
        }
    }

    let frame = rgba_with_alpha(accent, 255);
    for y in 0..spec.height as i32 {
        for x in 0..spec.width as i32 {
            if point_inside_avatar_shape(x, y, spec, shape)
                && (!point_inside_avatar_shape(x - 1, y, spec, shape)
                    || !point_inside_avatar_shape(x + 1, y, spec, shape)
                    || !point_inside_avatar_shape(x, y - 1, spec, shape)
                    || !point_inside_avatar_shape(x, y + 1, spec, shape))
            {
                image.put_pixel(x as u32, y as u32, frame);
            }
        }
    }
}

pub(crate) fn point_inside_avatar_shape(
    x: i32,
    y: i32,
    spec: AvatarSpec,
    shape: AvatarShape,
) -> bool {
    if x < 0 || y < 0 || x >= spec.width as i32 || y >= spec.height as i32 {
        return false;
    }

    let w = i64::from(spec.width);
    let h = i64::from(spec.height);
    let x2 = i64::from(x) * 2 + 1;
    let y2 = i64::from(y) * 2 + 1;

    match shape {
        AvatarShape::Square => true,
        AvatarShape::Circle => {
            let dx = x2 - w;
            let dy = y2 - h;
            dx * dx * h * h + dy * dy * w * w <= w * w * h * h
        }
        AvatarShape::Squircle => {
            let r = (w.min(h) * 18 / 100).max(1);
            let inner_left = r;
            let inner_right = w - r;
            let inner_top = r;
            let inner_bottom = h - r;
            if (x2 >= inner_left * 2 && x2 <= inner_right * 2)
                || (y2 >= inner_top * 2 && y2 <= inner_bottom * 2)
            {
                true
            } else {
                let cx = if x2 < inner_left * 2 {
                    inner_left
                } else {
                    inner_right
                };
                let cy = if y2 < inner_top * 2 {
                    inner_top
                } else {
                    inner_bottom
                };
                let dx = x2 - cx * 2;
                let dy = y2 - cy * 2;
                dx * dx + dy * dy <= (r * 2) * (r * 2)
            }
        }
        AvatarShape::Hexagon => point_inside_percent_polygon(
            x,
            y,
            spec,
            &[(25, 0), (75, 0), (100, 50), (75, 100), (25, 100), (0, 50)],
        ),
        AvatarShape::Octagon => point_inside_percent_polygon(
            x,
            y,
            spec,
            &[
                (30, 0),
                (70, 0),
                (100, 30),
                (100, 70),
                (70, 100),
                (30, 100),
                (0, 70),
                (0, 30),
            ],
        ),
    }
}

pub(crate) fn point_inside_percent_polygon(
    x: i32,
    y: i32,
    spec: AvatarSpec,
    points: &[(i64, i64)],
) -> bool {
    let mut inside = false;
    let width = i64::from(spec.width);
    let height = i64::from(spec.height);
    let px = (i64::from(x) * 2 + 1) * 50;
    let py = (i64::from(y) * 2 + 1) * 50;
    let Some(&last) = points.last() else {
        return false;
    };
    let mut previous = last;
    for &current in points {
        let xi = current.0 * width;
        let yi = current.1 * height;
        let xj = previous.0 * width;
        let yj = previous.1 * height;
        let crosses = (yi > py) != (yj > py);
        if crosses {
            let lhs = (px - xi) * (yj - yi);
            let rhs = (xj - xi) * (py - yi);
            let is_left = if yj > yi { lhs < rhs } else { lhs > rhs };
            if is_left {
                inside = !inside;
            }
        }
        previous = current;
    }
    inside
}

pub(crate) fn render_style_svg_layers(
    spec: AvatarSpec,
    style: AvatarStyleOptions,
    identity: &AvatarIdentity,
) -> String {
    if !style.has_extra_layers() {
        return String::new();
    }

    let accent = style_accent_color(style.color, identity);
    let mut svg = String::new();
    if style.color != AvatarColor::Default {
        svg.push_str(&render_color_svg_layer(spec, accent));
    }
    svg.push_str(&render_expression_svg_layer(
        spec,
        style.kind,
        style.expression,
        accent,
    ));
    svg.push_str(&render_accessory_svg_layer(
        spec,
        style.kind,
        style.accessory,
        accent,
    ));
    svg
}

pub(crate) fn render_color_svg_layer(spec: AvatarSpec, accent: Color) -> String {
    let bar_height = spec.height as f32 * 0.08;
    let stripe = spec.width.min(spec.height) as f32 * 0.03;
    format!(
        r#"<g data-layer="color"><rect x="0" y="{bar_y}" width="{w}" height="{bar_h}" fill="{fill}" opacity="0.82"/><rect x="0" y="0" width="{stripe}" height="{h}" fill="{fill}" opacity="0.56"/><rect x="{right}" y="0" width="{stripe}" height="{h}" fill="{fill}" opacity="0.56"/></g>"#,
        bar_y = spec.height as f32 - bar_height,
        w = spec.width,
        h = spec.height,
        bar_h = bar_height,
        stripe = stripe,
        right = spec.width as f32 - stripe,
        fill = color_hex(accent),
    )
}

pub(crate) fn render_accessory_svg_layer(
    spec: AvatarSpec,
    kind: AvatarKind,
    accessory: AvatarAccessory,
    accent: Color,
) -> String {
    if accessory == AvatarAccessory::None {
        return String::new();
    }

    let Some(anchors) = avatar_layer_anchors(kind) else {
        return String::new();
    };

    let min = spec.width.min(spec.height) as f32;
    let (left_eye_x, left_eye_y) = (
        anchors.left_eye.0 * spec.width as f32,
        anchors.left_eye.1 * spec.height as f32,
    );
    let (right_eye_x, right_eye_y) = (
        anchors.right_eye.0 * spec.width as f32,
        anchors.right_eye.1 * spec.height as f32,
    );
    let (mouth_x, mouth_y) = (
        anchors.mouth.0 * spec.width as f32,
        anchors.mouth.1 * spec.height as f32,
    );
    let eye_y = (left_eye_y + right_eye_y) / 2.0;
    let top_y = anchors.top * spec.height as f32;
    let neck_y = anchors.neck * spec.height as f32;
    let face_half = anchors.face_width * min / 2.0;
    let eye_radius = (anchors.eye_radius * min).max(3.0);
    let glasses_offset = glasses_y_offset(kind) * spec.height as f32;
    let fill = color_hex(accent);
    let dark = "#1f2937";
    let layer = accessory.as_str();

    let body = match accessory {
        AvatarAccessory::None => String::new(),
        AvatarAccessory::Glasses => format!(
            r#"<circle cx="{lx}" cy="{y}" r="{r}" fill="none" stroke="{dark}" stroke-width="3"/><circle cx="{rx}" cy="{y}" r="{r}" fill="none" stroke="{dark}" stroke-width="3"/><line x1="{l2}" y1="{y}" x2="{r2}" y2="{y}" stroke="{dark}" stroke-width="3"/>"#,
            lx = left_eye_x,
            rx = right_eye_x,
            y = eye_y + glasses_offset,
            r = eye_radius * 2.0,
            l2 = left_eye_x + eye_radius * 2.0,
            r2 = right_eye_x - eye_radius * 2.0,
        ),
        AvatarAccessory::Hat => format!(
            r#"<rect x="{x}" y="{y}" width="{tw}" height="{th}" fill="{fill}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{dark}"/>"#,
            x = mouth_x - face_half * 0.35,
            y = (top_y - min * 0.09 + hat_y_offset(kind) * spec.height as f32).max(0.0),
            tw = face_half * 0.70,
            th = min * 0.18,
            bx = mouth_x - face_half * 0.70,
            by = (top_y - min * 0.09 + hat_y_offset(kind) * spec.height as f32).max(0.0)
                + min * 0.18,
            bw = face_half * 1.40,
            bh = min * 0.05,
        ),
        AvatarAccessory::Headphones => format!(
            r#"<path d="M {l} {y} Q {cx} {top} {rr} {y}" fill="none" stroke="{dark}" stroke-width="4" stroke-linecap="round"/><circle cx="{l}" cy="{ear_y}" r="{er}" fill="{fill}" stroke="{dark}" stroke-width="2"/><circle cx="{rr}" cy="{ear_y}" r="{er}" fill="{fill}" stroke="{dark}" stroke-width="2"/>"#,
            l = mouth_x - face_half,
            rr = mouth_x + face_half,
            cx = mouth_x,
            y = eye_y,
            top = top_y,
            ear_y = eye_y,
            er = eye_radius * 2.0,
        ),
        AvatarAccessory::Crown => {
            let crown_top_y = (top_y + crown_y_offset(kind) * spec.height as f32).max(0.0);
            let p = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{} {},{}",
                mouth_x - face_half * 0.70,
                crown_top_y + min * 0.12,
                mouth_x - face_half * 0.45,
                crown_top_y,
                mouth_x - face_half * 0.16,
                crown_top_y + min * 0.10,
                mouth_x,
                (crown_top_y - min * 0.04).max(0.0),
                mouth_x + face_half * 0.16,
                crown_top_y + min * 0.10,
                mouth_x + face_half * 0.45,
                crown_top_y,
                mouth_x + face_half * 0.70,
                crown_top_y + min * 0.12
            );
            format!(
                r#"<polygon points="{p}" fill="{fill}"/><line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="{dark}" stroke-width="3"/>"#,
                x1 = mouth_x - face_half * 0.70,
                x2 = mouth_x + face_half * 0.70,
                y = crown_top_y + min * 0.12,
            )
        }
        AvatarAccessory::Bowtie => {
            let bowtie_y = neck_y + bowtie_y_offset(kind) * spec.height as f32;
            let lp = format!(
                "{},{} {},{} {},{}",
                mouth_x,
                bowtie_y,
                mouth_x - min * 0.20,
                bowtie_y - min * 0.10,
                mouth_x - min * 0.20,
                bowtie_y + min * 0.10
            );
            let rp = format!(
                "{},{} {},{} {},{}",
                mouth_x,
                bowtie_y,
                mouth_x + min * 0.20,
                bowtie_y - min * 0.10,
                mouth_x + min * 0.20,
                bowtie_y + min * 0.10
            );
            format!(
                r#"<polygon points="{lp}" fill="{fill}"/><polygon points="{rp}" fill="{fill}"/><circle cx="{cx}" cy="{y}" r="{r}" fill="{dark}"/>"#,
                cx = mouth_x,
                y = bowtie_y,
                r = min * 0.05,
            )
        }
        AvatarAccessory::Eyepatch => format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{dark}" stroke-width="3"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{dark}"/>"#,
            x1 = mouth_x - face_half,
            y1 = top_y + eyepatch_y_offset(kind) * spec.height as f32 * 0.50,
            x2 = mouth_x + face_half * 0.70,
            y2 = mouth_y - eye_radius + eyepatch_y_offset(kind) * spec.height as f32,
            cx = left_eye_x,
            cy = left_eye_y + eyepatch_y_offset(kind) * spec.height as f32,
            rx = eye_radius * 2.0,
            ry = eye_radius * 1.5,
        ),
        AvatarAccessory::Scarf => format!(
            r#"<rect x="{x}" y="{y}" width="{sw}" height="{sh}" fill="{fill}"/><rect x="{tx}" y="{ty}" width="{tw}" height="{th}" fill="{fill}"/>"#,
            x = mouth_x - face_half * 0.80,
            y = neck_y,
            sw = face_half * 1.60,
            sh = min * 0.08,
            tx = mouth_x + face_half * 0.20,
            ty = neck_y + min * 0.04,
            tw = min * 0.09,
            th = min * 0.20,
        ),
        AvatarAccessory::Halo => format!(
            r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="none" stroke="{fill}" stroke-width="3"/>"#,
            cx = mouth_x,
            cy = (top_y - min * 0.07).max(0.0),
            rx = (face_half * 0.70).max(min * 0.10),
            ry = min * 0.07,
        ),
        AvatarAccessory::Horns => {
            let horn_top_y = top_y + horns_y_offset(kind) * spec.height as f32;
            let lp = format!(
                "{},{} {},{} {},{}",
                mouth_x - face_half * 0.60,
                horn_top_y + min * 0.12,
                mouth_x - face_half,
                (horn_top_y - min * 0.12).max(0.0),
                mouth_x - face_half * 0.30,
                horn_top_y + min * 0.07
            );
            let rp = format!(
                "{},{} {},{} {},{}",
                mouth_x + face_half * 0.60,
                horn_top_y + min * 0.12,
                mouth_x + face_half,
                (horn_top_y - min * 0.12).max(0.0),
                mouth_x + face_half * 0.30,
                horn_top_y + min * 0.07
            );
            format!(
                r#"<polygon points="{lp}" fill="{fill}"/><polygon points="{rp}" fill="{fill}"/>"#
            )
        }
    };

    format!(r#"<g data-layer="accessory-{layer}">{body}</g>"#)
}

pub(crate) fn render_expression_svg_layer(
    spec: AvatarSpec,
    kind: AvatarKind,
    expression: AvatarExpression,
    accent: Color,
) -> String {
    if expression == AvatarExpression::Default {
        return String::new();
    }

    let Some(anchors) = avatar_layer_anchors(kind) else {
        return String::new();
    };

    let min = spec.width.min(spec.height) as f32;
    let left_eye_x = anchors.left_eye.0 * spec.width as f32;
    let left_eye_y = anchors.left_eye.1 * spec.height as f32;
    let right_eye_x = anchors.right_eye.0 * spec.width as f32;
    let right_eye_y = anchors.right_eye.1 * spec.height as f32;
    let mouth_x = anchors.mouth.0 * spec.width as f32;
    let mouth_y = anchors.mouth.1 * spec.height as f32;
    let eye_radius = (anchors.eye_radius * min).max(3.0);
    let dark = "#111827";
    let layer = expression.as_str();
    let body = match expression {
        AvatarExpression::Default => String::new(),
        AvatarExpression::Happy => format!(
            r#"<path d="M {x1} {y} Q {cx} {my} {x2} {y}" fill="none" stroke="{dark}" stroke-width="3" stroke-linecap="round"/>"#,
            x1 = mouth_x - min * 0.14,
            x2 = mouth_x + min * 0.14,
            y = mouth_y,
            cx = mouth_x,
            my = mouth_y + min * 0.09,
        ),
        AvatarExpression::Grumpy => format!(
            r#"<path d="M {x1} {y} Q {cx} {my} {x2} {y}" fill="none" stroke="{dark}" stroke-width="3" stroke-linecap="round"/>"#,
            x1 = mouth_x - min * 0.14,
            x2 = mouth_x + min * 0.14,
            y = mouth_y,
            cx = mouth_x,
            my = mouth_y - min * 0.09,
        ),
        AvatarExpression::Surprised => format!(
            r#"<circle cx="{cx}" cy="{y}" r="{r}" fill="none" stroke="{dark}" stroke-width="3"/>"#,
            cx = mouth_x,
            y = mouth_y,
            r = min * 0.07,
        ),
        AvatarExpression::Sleepy => format!(
            r#"<line x1="{x1}" y1="{ey}" x2="{x2}" y2="{ey}" stroke="{dark}" stroke-width="3" stroke-linecap="round"/><line x1="{x3}" y1="{ey}" x2="{x4}" y2="{ey}" stroke="{dark}" stroke-width="3" stroke-linecap="round"/>"#,
            x1 = left_eye_x - eye_radius,
            x2 = left_eye_x + eye_radius,
            x3 = right_eye_x - eye_radius,
            x4 = right_eye_x + eye_radius,
            ey = (left_eye_y + right_eye_y) / 2.0,
        ),
        AvatarExpression::Winking => format!(
            r#"<line x1="{x1}" y1="{ey}" x2="{x2}" y2="{ey}" stroke="{dark}" stroke-width="3" stroke-linecap="round"/><circle cx="{rx}" cy="{ey}" r="{r}" fill="{dark}"/>"#,
            x1 = left_eye_x - eye_radius,
            x2 = left_eye_x + eye_radius,
            rx = right_eye_x,
            ey = right_eye_y,
            r = eye_radius * 0.70,
        ),
        AvatarExpression::Cool => format!(
            r#"<rect x="{x}" y="{ey}" width="{ww}" height="{hh}" fill="{dark}"/>"#,
            x = left_eye_x.min(right_eye_x) - eye_radius * 2.0,
            ey = left_eye_y.min(right_eye_y) - eye_radius,
            ww = right_eye_x.max(left_eye_x) - left_eye_x.min(right_eye_x) + eye_radius * 4.0,
            hh = eye_radius * 2.0,
        ),
        AvatarExpression::Crying => format!(
            r#"<path d="M {x1} {y} Q {cx} {my} {x2} {y}" fill="none" stroke="{dark}" stroke-width="3" stroke-linecap="round"/><ellipse cx="{tx}" cy="{ty}" rx="{rx}" ry="{ry}" fill="{fill}"/>"#,
            x1 = mouth_x - min * 0.12,
            x2 = mouth_x + min * 0.12,
            y = mouth_y,
            cx = mouth_x,
            my = mouth_y - min * 0.08,
            tx = right_eye_x + eye_radius,
            ty = right_eye_y + eye_radius * 2.0,
            rx = eye_radius * 0.70,
            ry = eye_radius * 1.50,
            fill = color_hex(accent),
        ),
    };
    format!(r#"<g data-layer="expression-{layer}">{body}</g>"#)
}

pub(crate) fn render_shape_svg_layer(
    spec: AvatarSpec,
    shape: AvatarShape,
    accent: Color,
) -> String {
    if shape == AvatarShape::Square {
        return String::new();
    }

    let w = spec.width as f32;
    let h = spec.height as f32;
    let fill = color_hex(accent);
    let body = match shape {
        AvatarShape::Square => String::new(),
        AvatarShape::Circle => format!(
            r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="none" stroke="{fill}" stroke-width="3"/>"#,
            cx = w / 2.0,
            cy = h / 2.0,
            rx = w / 2.0 - 1.5,
            ry = h / 2.0 - 1.5,
        ),
        AvatarShape::Squircle => format!(
            r#"<rect x="1.5" y="1.5" width="{rw}" height="{rh}" rx="{r}" ry="{r}" fill="none" stroke="{fill}" stroke-width="3"/>"#,
            rw = w - 3.0,
            rh = h - 3.0,
            r = w.min(h) * 0.18,
        ),
        AvatarShape::Hexagon => {
            let points = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{}",
                w * 0.25,
                1.5,
                w * 0.75,
                1.5,
                w - 1.5,
                h * 0.5,
                w * 0.75,
                h - 1.5,
                w * 0.25,
                h - 1.5,
                1.5,
                h * 0.5
            );
            format!(r#"<polygon points="{points}" fill="none" stroke="{fill}" stroke-width="3"/>"#)
        }
        AvatarShape::Octagon => {
            let points = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{} {},{} {},{}",
                w * 0.30,
                1.5,
                w * 0.70,
                1.5,
                w - 1.5,
                h * 0.30,
                w - 1.5,
                h * 0.70,
                w * 0.70,
                h - 1.5,
                w * 0.30,
                h - 1.5,
                1.5,
                h * 0.70,
                1.5,
                h * 0.30
            );
            format!(r#"<polygon points="{points}" fill="none" stroke="{fill}" stroke-width="3"/>"#)
        }
    };
    format!(r#"<g data-layer="shape-{}">{body}</g>"#, shape.as_str())
}

pub(crate) fn render_shape_svg_clip(
    spec: AvatarSpec,
    shape: AvatarShape,
    definition_prefix: &str,
    content: &str,
) -> (String, String) {
    if shape == AvatarShape::Square {
        return (String::new(), content.to_owned());
    }

    let clip_id = format!("{definition_prefix}-frame-clip");
    let clip = render_shape_svg_clip_body(spec, shape);
    (
        format!(r#"<defs><clipPath id="{clip_id}">{clip}</clipPath></defs>"#),
        format!(r#"<g clip-path="url(#{clip_id})">{content}</g>"#),
    )
}

pub(crate) fn render_shape_svg_clip_body(spec: AvatarSpec, shape: AvatarShape) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    match shape {
        AvatarShape::Square => format!(r#"<rect x="0" y="0" width="{w}" height="{h}"/>"#),
        AvatarShape::Circle => format!(
            r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}"/>"#,
            cx = w / 2.0,
            cy = h / 2.0,
            rx = w / 2.0,
            ry = h / 2.0,
        ),
        AvatarShape::Squircle => format!(
            r#"<rect x="0" y="0" width="{w}" height="{h}" rx="{r}" ry="{r}"/>"#,
            r = w.min(h) * 0.18,
        ),
        AvatarShape::Hexagon => {
            let points = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{}",
                w * 0.25,
                0.0,
                w * 0.75,
                0.0,
                w,
                h * 0.5,
                w * 0.75,
                h,
                w * 0.25,
                h,
                0.0,
                h * 0.5
            );
            format!(r#"<polygon points="{points}"/>"#)
        }
        AvatarShape::Octagon => {
            let points = format!(
                "{},{} {},{} {},{} {},{} {},{} {},{} {},{} {},{}",
                w * 0.30,
                0.0,
                w * 0.70,
                0.0,
                w,
                h * 0.30,
                w,
                h * 0.70,
                w * 0.70,
                h,
                w * 0.30,
                h,
                0.0,
                h * 0.70,
                0.0,
                h * 0.30
            );
            format!(r#"<polygon points="{points}"/>"#)
        }
    }
}
