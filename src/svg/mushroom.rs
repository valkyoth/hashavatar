use super::*;

pub(crate) fn render_mushroom_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let cap_rx = w * (0.24 + identity.unit_f32(4) * 0.08);
    let cap_ry = h * (0.14 + identity.unit_f32(5) * 0.06);
    let stem_rx = w * (0.09 + identity.unit_f32(6) * 0.04);
    let stem_ry = h * (0.18 + identity.unit_f32(7) * 0.05);
    let cap = hsl_to_color(350.0 + identity.unit_f32(1) * 45.0, 0.58, 0.52);
    let stem = hsl_to_color(35.0 + identity.unit_f32(2) * 20.0, 0.24, 0.86);
    let gill = hsl_to_color(26.0 + identity.unit_f32(3) * 20.0, 0.20, 0.70);
    let spots = format!(
        r##"<circle cx="{s1x}" cy="{s1y}" r="{sr1}" fill="#fff6e6" opacity="0.92"/><circle cx="{s2x}" cy="{s2y}" r="{sr2}" fill="#fff6e6" opacity="0.92"/><circle cx="{s3x}" cy="{s3y}" r="{sr3}" fill="#fff6e6" opacity="0.92"/>"##,
        s1x = cx - cap_rx * 0.36,
        s2x = cx + cap_rx * 0.08,
        s3x = cx + cap_rx * 0.40,
        s1y = cy - cap_ry * 0.88,
        s2y = cy - cap_ry * 0.58,
        s3y = cy - cap_ry * 0.76,
        sr1 = cap_rx * (0.06 + identity.unit_f32(19) * 0.04),
        sr2 = cap_rx * (0.07 + identity.unit_f32(20) * 0.04),
        sr3 = cap_rx * (0.05 + identity.unit_f32(21) * 0.04),
    );
    format!(
        r##"<ellipse cx="{cx}" cy="{scy}" rx="{srx}" ry="{sry}" fill="{stem}"/><ellipse cx="{cx}" cy="{ccy}" rx="{crx}" ry="{cry}" fill="{cap}"/><rect x="{rx}" y="{ry}" width="{rw}" height="{rh}" fill="{cap}"/><ellipse cx="{cx}" cy="{gcy}" rx="{grx}" ry="{gry}" fill="{gill}"/>{spots}"##,
        cx = cx,
        scy = cy + stem_ry / 3.0,
        srx = stem_rx,
        sry = stem_ry,
        stem = color_hex(stem),
        ccy = cy - cap_ry / 2.0,
        crx = cap_rx,
        cry = cap_ry,
        cap = color_hex(cap),
        rx = cx - cap_rx,
        ry = cy - cap_ry / 2.0,
        rw = cap_rx * 2.0,
        rh = cap_ry,
        gcy = cy + cap_ry / 3.0,
        grx = cap_rx,
        gry = cap_ry / 3.0,
        gill = color_hex(gill),
        spots = spots,
    )
}
