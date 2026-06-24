use super::*;

pub(crate) fn render_bear_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let rx = w * (0.27 + identity.unit_f32(4) * 0.05);
    let ry = h * (0.24 + identity.unit_f32(5) * 0.05);
    let fur = hsl_to_color(24.0 + identity.unit_f32(1) * 24.0, 0.38, 0.48);
    let muzzle = hsl_to_color(32.0 + identity.unit_f32(2) * 12.0, 0.22, 0.84);
    let inner = hsl_to_color(18.0 + identity.unit_f32(3) * 18.0, 0.34, 0.72);
    format!(
        r##"<circle cx="{lel}" cy="{ey}" r="{er}" fill="{fur}"/><circle cx="{rel}" cy="{ey}" r="{er}" fill="{fur}"/><circle cx="{lel}" cy="{ey}" r="{ir}" fill="{inner}"/><circle cx="{rel}" cy="{ey}" r="{ir}" fill="{inner}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{fur}"/><ellipse cx="{cx}" cy="{my}" rx="{mrx}" ry="{mry}" fill="{muzzle}"/><circle cx="{lx}" cy="{eye_y}" r="{pr}" fill="#2d221c"/><circle cx="{rx2}" cy="{eye_y}" r="{pr}" fill="#2d221c"/><ellipse cx="{cx}" cy="{ny}" rx="{nrx}" ry="{nry}" fill="#2d221c"/><path d="M {mx1} {mouth_y} Q {cx} {curve_y} {mx2} {mouth_y}" stroke="#2d221c" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        lel = cx - rx * 0.75,
        rel = cx + rx * 0.75,
        ey = cy - ry,
        er = rx * 0.28,
        ir = rx * 0.14,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        fur = color_hex(fur),
        inner = color_hex(inner),
        my = cy + ry * 0.25,
        mrx = rx * 0.40,
        mry = ry * 0.34,
        muzzle = color_hex(muzzle),
        lx = cx - rx / 3.0,
        rx2 = cx + rx / 3.0,
        eye_y = cy - ry * 0.20,
        pr = rx * 0.10,
        ny = cy + ry * 0.16,
        nrx = rx * 0.13,
        nry = ry * 0.10,
        mx1 = cx - rx * 0.18,
        mx2 = cx + rx * 0.18,
        mouth_y = cy + ry * 0.30,
        curve_y = cy + ry * 0.42,
    )
}
