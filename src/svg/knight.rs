fn render_knight_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.55;
    let rx = w * (0.20 + identity.unit_f32(4) * 0.05);
    let ry = h * (0.24 + identity.unit_f32(5) * 0.05);
    let steel = hsl_to_color(205.0 + identity.unit_f32(1) * 45.0, 0.12, 0.66);
    let dark = hsl_to_color(215.0 + identity.unit_f32(2) * 45.0, 0.14, 0.22);
    let plume = hsl_to_color(identity.unit_f32(3) * 360.0, 0.58, 0.54);
    format!(
        r##"<ellipse cx="{cx}" cy="{hy}" rx="{rx}" ry="{ry}" fill="{steel}"/><rect x="{x}" y="{y}" width="{rw}" height="{rh}" fill="{steel}"/><rect x="{vx}" y="{vy}" width="{vw}" height="{vh}" fill="{dark}"/><rect x="{s1x}" y="{vy}" width="{sw}" height="{vh}" fill="#ffffff" opacity="0.35"/><rect x="{s2x}" y="{vy}" width="{sw}" height="{vh}" fill="#ffffff" opacity="0.35"/><line x1="{cx}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="#ffffff" stroke-width="3" opacity="0.5"/><polygon points="{cx},{py1} {px2},{py2} {px3},{py3}" fill="{plume}"/>"##,
        cx = cx,
        hy = cy - ry / 5.0,
        rx = rx,
        ry = ry,
        steel = color_hex(steel),
        x = cx - rx,
        y = cy - ry / 5.0,
        rw = rx * 2.0,
        rh = ry * 1.20,
        vx = cx - rx * 0.75,
        vy = cy - ry / 5.0,
        vw = rx * 1.50,
        vh = ry * 0.20,
        dark = color_hex(dark),
        s1x = cx - rx * 0.34,
        s2x = cx + rx * 0.22,
        sw = rx * 0.10,
        ly1 = cy - ry,
        ly2 = cy + ry,
        py1 = cy - ry,
        px2 = cx - rx * 0.20,
        py2 = cy - ry * 1.50,
        px3 = cx + rx * 0.25,
        py3 = cy - ry * 1.32,
        plume = color_hex(plume),
    )
}

