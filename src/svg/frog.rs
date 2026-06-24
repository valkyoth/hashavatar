fn render_frog_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.57 + identity.unit_f32(6) * 0.04);
    let rx = w * (0.24 + identity.unit_f32(4) * 0.06);
    let ry = h * (0.18 + identity.unit_f32(5) * 0.05);
    let green = hsl_to_color(92.0 + identity.unit_f32(1) * 72.0, 0.46, 0.54);
    let dark = hsl_to_color(98.0 + identity.unit_f32(2) * 60.0, 0.40, 0.28);
    let cheek = hsl_to_color(335.0 + identity.unit_f32(3) * 24.0, 0.42, 0.76);
    let er = rx * (0.18 + identity.unit_f32(7) * 0.04);
    format!(
        r##"<circle cx="{elx}" cy="{ey}" r="{er}" fill="{green}"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="{green}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{green}"/><circle cx="{elx}" cy="{ey}" r="{ew}" fill="#fffff5"/><circle cx="{erx}" cy="{ey}" r="{ew}" fill="#fffff5"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="{dark}"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="{dark}"/><circle cx="{clx}" cy="{ccy}" r="{cr}" fill="{cheek}" opacity="0.6"/><circle cx="{crx}" cy="{ccy}" r="{cr}" fill="{cheek}" opacity="0.6"/><path d="M {mx1} {my} q {q1} {qd} {q2} 0 M {mx2} {my} q {q1} {qd} {q2} 0" stroke="{dark}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        green = color_hex(green),
        elx = cx - rx * 0.50,
        erx = cx + rx * 0.50,
        ey = cy - ry,
        er = er,
        ew = er * 0.64,
        pr = er * 0.30,
        dark = color_hex(dark),
        clx = cx - rx * 0.50,
        crx = cx + rx * 0.50,
        ccy = cy + ry * 0.25,
        cr = rx * 0.09,
        cheek = color_hex(cheek),
        mx1 = cx - rx * 0.20,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.22,
        q1 = rx * 0.20,
        qd = ry * 0.35,
        q2 = rx * 0.36,
    )
}

