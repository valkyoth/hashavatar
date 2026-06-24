use super::*;

pub(crate) fn render_wizard_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.57 + identity.unit_f32(17) * 0.07);
    let r = w * (0.16 + identity.unit_f32(15) * 0.06);
    let hat = hsl_to_color(
        210.0 + identity.unit_f32(11) * 110.0,
        0.34 + identity.unit_f32(20) * 0.22,
        0.28 + identity.unit_f32(21) * 0.16,
    );
    let band = hsl_to_color(
        24.0 + identity.unit_f32(12) * 160.0,
        0.62 + identity.unit_f32(22) * 0.24,
        0.48 + identity.unit_f32(23) * 0.18,
    );
    let skin = hsl_to_color(
        18.0 + identity.unit_f32(13) * 28.0,
        0.22 + identity.unit_f32(24) * 0.20,
        0.74 + identity.unit_f32(25) * 0.12,
    );
    let beard = hsl_to_color(
        35.0 + identity.unit_f32(14) * 45.0,
        0.06 + identity.unit_f32(26) * 0.12,
        0.80 + identity.unit_f32(27) * 0.16,
    );
    let hat_width = r * (1.0 + identity.unit_f32(28) * 0.55);
    let hat_height = h * (0.28 + identity.unit_f32(29) * 0.12);
    let tip_shift = (identity.unit_f32(30) - 0.5) * r * 0.9;
    let stars = format!(
        r##"<circle cx="{s1x}" cy="{s1y}" r="{sr}" fill="{band}"/><circle cx="{s2x}" cy="{s2y}" r="{sr2}" fill="{band}"/>"##,
        s1x = cx - hat_width * 0.35,
        s1y = cy - h * 0.20,
        s2x = cx + tip_shift * 0.5 + hat_width * 0.22,
        s2y = cy - h * 0.28,
        sr = w * (0.010 + identity.unit_f32(34) * 0.012),
        sr2 = w * (0.008 + identity.unit_f32(35) * 0.010),
        band = color_hex(band),
    );
    format!(
        r##"<polygon points="{x1},{y1} {x2},{y1} {tx},{y2}" fill="{hat}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{band}"/>{stars}<circle cx="{cx}" cy="{cy}" r="{r}" fill="{skin}"/><polygon points="{b1},{b2} {b3},{b2} {bt},{b4}" fill="{beard}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#241e34"/><circle cx="{sx}" cy="{sy}" r="{sr}" fill="{band}"/>"##,
        cx = cx,
        cy = cy,
        x1 = cx - hat_width,
        x2 = cx + hat_width,
        tx = cx + tip_shift,
        y1 = cy - h * 0.08,
        y2 = cy - h * 0.08 - hat_height,
        hat = color_hex(hat),
        bx = cx - w * (0.24 + identity.unit_f32(31) * 0.08),
        by = cy - h * 0.08,
        bw = w * (0.48 + identity.unit_f32(31) * 0.16),
        bh = h * (0.030 + identity.unit_f32(32) * 0.025),
        band = color_hex(band),
        stars = stars,
        r = r,
        skin = color_hex(skin),
        b1 = cx - r * (0.52 + identity.unit_f32(44) * 0.20),
        b2 = cy + h * 0.06,
        b3 = cx + r * (0.52 + identity.unit_f32(45) * 0.20),
        bt = cx + (identity.unit_f32(46) - 0.5) * r * 0.45,
        b4 = cy + h * (0.22 + identity.unit_f32(47) * 0.10),
        beard = color_hex(beard),
        elx = cx - r * 0.36,
        erx = cx + r * 0.36,
        ey = cy - h * 0.03,
        er = r * 0.13,
        pr = r * 0.055,
        sx = cx + tip_shift + r * 0.50,
        sy = cy - h * 0.08 - hat_height,
        sr = r * 0.10,
    )
}
