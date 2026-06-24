use super::*;

pub(crate) fn render_planet_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.53 + identity.unit_f32(8) * 0.06);
    let r = w.min(h) * (0.18 + identity.unit_f32(20) * 0.08);
    let planet = hsl_to_color(identity.unit_f32(1) * 360.0, 0.46, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.42);
    let ring = hsl_to_color(32.0 + identity.unit_f32(3) * 120.0, 0.44, 0.72);
    let ring_rx = r * (1.55 + identity.unit_f32(21) * 0.28);
    let ring_ry = r * (0.38 + identity.unit_f32(22) * 0.12);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rrx}" ry="{rry}" fill="{ring}"/><circle cx="{cx}" cy="{cy}" r="{r}" fill="{planet}"/><ellipse cx="{sx}" cy="{sy}" rx="{srx}" ry="{sry}" fill="{shade}" opacity="0.45"/><ellipse cx="{hx}" cy="{hy}" rx="{hrx}" ry="{hry}" fill="#ffffff" opacity="0.32"/><rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" rx="{cr}" fill="{ring}" opacity="0.78"/>"##,
        cx = cx,
        cy = cy,
        r = r,
        rrx = ring_rx,
        rry = ring_ry,
        ring = color_hex(ring),
        planet = color_hex(planet),
        sx = cx - r * 0.25,
        sy = cy - r * 0.20,
        srx = r * 0.50,
        sry = r * 0.20,
        shade = color_hex(shade),
        hx = cx + r * 0.25,
        hy = cy + r * 0.20,
        hrx = r * 0.50,
        hry = r * 0.16,
        rx = cx - ring_rx,
        ry = cy - ring_ry * 0.16,
        rw = ring_rx * 2.0,
        rh = ring_ry * 0.32,
        cr = ring_ry * 0.16,
    )
}
