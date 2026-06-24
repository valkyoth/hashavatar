fn render_cat_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let rx = w * (0.26 + identity.unit_f32(20) * 0.07);
    let ry = h * (0.22 + identity.unit_f32(21) * 0.08);
    let head = hsl_to_color(20.0 + identity.unit_f32(0) * 40.0, 0.48, 0.64);
    let muzzle = hsl_to_color(28.0 + identity.unit_f32(1) * 18.0, 0.18, 0.90);
    let eye = hsl_to_color(90.0 + identity.unit_f32(2) * 40.0, 0.7, 0.55);
    let outline = Color::rgb(64, 45, 32);
    let left_ear = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.8,
        cy - ry * 0.4,
        cx - rx * 0.4,
        cy - ry * 1.3,
        cx - rx * 0.1,
        cy - ry * 0.1
    );
    let right_ear = format!(
        "{},{} {},{} {},{}",
        cx + rx * 0.8,
        cy - ry * 0.4,
        cx + rx * 0.4,
        cy - ry * 1.3,
        cx + rx * 0.1,
        cy - ry * 0.1
    );
    let nose = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.06,
        cy + ry * 0.1,
        cx + rx * 0.06,
        cy + ry * 0.1,
        cx,
        cy + ry * 0.2
    );
    format!(
        r##"<polygon points="{left_ear}" fill="{head}"/><polygon points="{right_ear}" fill="{head}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{head}"/><ellipse cx="{cx}" cy="{muzzle_y}" rx="{muzzle_rx}" ry="{muzzle_ry}" fill="{muzzle}"/><ellipse cx="{left_eye_x}" cy="{eye_y}" rx="{eye_rx}" ry="{eye_ry}" fill="{eye}"/><ellipse cx="{right_eye_x}" cy="{eye_y}" rx="{eye_rx}" ry="{eye_ry}" fill="{eye}"/><polygon points="{nose}" fill="#d6818d"/><path d="M {left_mx} {mouth_y} q {curve_x} {curve_y} {curve_end} 0 M {right_mx} {mouth_y} q {curve_x} {curve_y} {curve_end} 0" stroke="{outline}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        left_ear = left_ear,
        right_ear = right_ear,
        head = color_hex(head),
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        muzzle_y = cy + ry * 0.28,
        muzzle_rx = rx * 0.45,
        muzzle_ry = ry * 0.28,
        muzzle = color_hex(muzzle),
        left_eye_x = cx - rx * 0.34,
        right_eye_x = cx + rx * 0.34,
        eye_y = cy - ry * 0.1,
        eye_rx = rx * 0.13,
        eye_ry = ry * 0.16,
        eye = color_hex(eye),
        nose = nose,
        left_mx = cx - rx * 0.08,
        right_mx = cx + rx * 0.08,
        mouth_y = cy + ry * 0.22,
        curve_x = rx * 0.1,
        curve_y = ry * 0.12,
        curve_end = rx * 0.16,
        outline = color_hex(outline),
    )
}

