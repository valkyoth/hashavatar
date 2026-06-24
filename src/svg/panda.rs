use super::*;

pub(crate) fn render_panda_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.56 + identity.unit_f32(5) * 0.04);
    let rx = w * (0.24 + identity.unit_f32(6) * 0.05);
    let ry = h * (0.22 + identity.unit_f32(7) * 0.04);
    let white = hsl_to_color(36.0 + identity.unit_f32(1) * 18.0, 0.10, 0.92);
    let black = hsl_to_color(210.0 + identity.unit_f32(2) * 28.0, 0.10, 0.18);
    let er = rx * (0.28 + identity.unit_f32(8) * 0.08);
    format!(
        r##"<circle cx="{lelx}" cy="{eary}" r="{er}" fill="{black}"/><circle cx="{rerx}" cy="{eary}" r="{er}" fill="{black}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{white}"/><ellipse cx="{plx}" cy="{py}" rx="{prx}" ry="{pry}" fill="{black}"/><ellipse cx="{prx2}" cy="{py}" rx="{prx}" ry="{pry}" fill="{black}"/><circle cx="{plx}" cy="{py}" r="{eye}" fill="#f8f8f4"/><circle cx="{prx2}" cy="{py}" r="{eye}" fill="#f8f8f4"/><ellipse cx="{cx}" cy="{ny}" rx="{nrx}" ry="{nry}" fill="{black}"/><path d="M {mx1} {my} q {mq} {md} {me} 0 M {mx2} {my} q {mq} {md} {me} 0" stroke="{black}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        white = color_hex(white),
        black = color_hex(black),
        lelx = cx - rx * 0.75,
        rerx = cx + rx * 0.75,
        eary = cy - ry * 0.75,
        er = er,
        plx = cx - rx * 0.33,
        prx2 = cx + rx * 0.33,
        py = cy - ry * 0.08,
        prx = rx * (0.20 + identity.unit_f32(9) * 0.05),
        pry = ry * (0.26 + identity.unit_f32(10) * 0.05),
        eye = rx * 0.055,
        ny = cy + ry * 0.20,
        nrx = rx * 0.09,
        nry = ry * 0.12,
        mx1 = cx - rx * 0.08,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.26,
        mq = rx * 0.12,
        md = ry * 0.24,
        me = rx * 0.22,
    )
}
