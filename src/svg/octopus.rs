fn render_octopus_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.54 + identity.unit_f32(5) * 0.05);
    let rx = w * (0.21 + identity.unit_f32(3) * 0.06);
    let ry = h * (0.20 + identity.unit_f32(4) * 0.06);
    let body = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.58);
    let shade = hsl_to_color(identity.unit_f32(2) * 360.0, 0.34, 0.38);
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{body}"/><rect x="{t1x}" y="{ty}" width="{tw}" height="{th1}" rx="{tr}" fill="{body}"/><rect x="{t2x}" y="{ty}" width="{tw}" height="{th2}" rx="{tr}" fill="{body}"/><rect x="{t3x}" y="{ty}" width="{tw}" height="{th3}" rx="{tr}" fill="{body}"/><rect x="{t4x}" y="{ty}" width="{tw}" height="{th4}" rx="{tr}" fill="{body}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fffff8"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fffff8"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#1c1a26"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#1c1a26"/><path d="M {mx1} {my} q {mq} {md} {me} 0 M {mx2} {my} q {mq} {md} {me} 0" stroke="{shade}" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        body = color_hex(body),
        t1x = cx - rx * 0.82,
        t2x = cx - rx * 0.28,
        t3x = cx + rx * 0.20,
        t4x = cx + rx * 0.68,
        ty = cy + ry * 0.45,
        tw = rx * 0.14,
        th1 = ry * (0.50 + identity.unit_f32(7) * 0.30),
        th2 = ry * (0.42 + identity.unit_f32(8) * 0.30),
        th3 = ry * (0.45 + identity.unit_f32(9) * 0.30),
        th4 = ry * (0.50 + identity.unit_f32(10) * 0.30),
        tr = rx * 0.07,
        elx = cx - rx / 3.0,
        erx = cx + rx / 3.0,
        ey = cy - ry / 6.0,
        er = rx / 9.0,
        pr = rx / 20.0,
        mx1 = cx - rx * 0.12,
        mx2 = cx + rx * 0.02,
        my = cy + ry * 0.30,
        mq = rx * 0.14,
        md = ry * 0.22,
        me = rx * 0.24,
        shade = color_hex(shade),
    )
}

