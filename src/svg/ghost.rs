fn render_ghost_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.53 + identity.unit_f32(5) * 0.06);
    let rx = w * (0.19 + identity.unit_f32(3) * 0.08);
    let ry = h * (0.21 + identity.unit_f32(4) * 0.08);
    let body = hsl_to_color(
        190.0 + identity.unit_f32(1) * 55.0,
        0.10 + identity.unit_f32(7) * 0.10,
        0.94 + identity.unit_f32(8) * 0.04,
    );
    let eye_style = if identity.byte(19).is_multiple_of(2) {
        (w * 0.026, h * 0.054)
    } else {
        (w * 0.038, h * 0.038)
    };
    let mouth = if identity.byte(20) % 3 == 1 {
        format!(
            r##"<ellipse cx="{cx}" cy="{my}" rx="{mrx}" ry="{mry}" fill="#8da0b2"/>"##,
            my = cy + h * 0.08,
            mrx = w * 0.035,
            mry = h * 0.045,
        )
    } else {
        format!(
            r##"<path d="M {mx1} {my} q {cq} {cyq} {ce} 0 M {mx2} {my} q {cq} {cyq} {ce} 0" stroke="#8da0b2" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
            mx1 = cx - w * 0.03,
            mx2 = cx + w * 0.03,
            my = cy + h * 0.08,
            cq = w * 0.04,
            cyq = if identity.byte(20) % 3 == 2 {
                0.0
            } else {
                h * 0.05
            },
            ce = w * 0.06,
        )
    };
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{body}"/><rect x="{x}" y="{cy}" width="{rw}" height="{rh}" fill="{body}"/><circle cx="{c1}" cy="{scy}" r="{sr1}" fill="{body}"/><circle cx="{c2}" cy="{scy}" r="{sr2}" fill="{body}"/><circle cx="{c3}" cy="{scy}" r="{sr3}" fill="{body}"/><ellipse cx="{lx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/><ellipse cx="{rx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#30384a"/>{mouth}"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        body = color_hex(body),
        x = cx - rx,
        rw = rx * 2.0,
        rh = ry * (0.82 + identity.unit_f32(12) * 0.28),
        c1 = cx - rx * 0.70,
        c2 = cx,
        c3 = cx + rx * 0.70,
        scy = cy + ry * 1.36,
        sr1 = rx * (0.18 + identity.unit_f32(13) * 0.12),
        sr2 = rx * (0.18 + identity.unit_f32(14) * 0.12),
        sr3 = rx * (0.18 + identity.unit_f32(15) * 0.12),
        lx = cx - rx * 0.36,
        rx2 = cx + rx * 0.36,
        ey = cy - ry * 0.25,
        erx = eye_style.0,
        ery = eye_style.1,
        mouth = mouth,
    )
}

