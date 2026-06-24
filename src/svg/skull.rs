fn render_skull_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.51 + identity.unit_f32(19) * 0.07);
    let rx = w * (0.18 + identity.unit_f32(17) * 0.07);
    let ry = h * (0.18 + identity.unit_f32(18) * 0.07);
    let bone = hsl_to_color(
        28.0 + identity.unit_f32(16) * 34.0,
        0.08 + identity.unit_f32(22) * 0.10,
        0.82 + identity.unit_f32(23) * 0.12,
    );
    let crack = color_hex(hsl_to_color(
        20.0 + identity.unit_f32(24) * 40.0,
        0.06,
        0.22 + identity.unit_f32(25) * 0.12,
    ));
    let teeth = 3 + (identity.byte(32) % 4) as usize;
    let mut tooth_markup = String::new();
    for tooth in 0..teeth {
        let x = cx - rx * 0.34 + tooth as f32 * (rx * 0.68 / teeth.max(1) as f32);
        tooth_markup.push_str(&format!(
            r##"<line x1="{x}" y1="{ty1}" x2="{x}" y2="{ty2}" stroke="{crack}" stroke-width="3"/>"##,
            ty1 = cy + ry * 0.52,
            ty2 = cy + ry * (0.86 + identity.unit_f32(46 + tooth) * 0.25),
        ));
    }
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{bone}"/><rect x="{jx}" y="{jy}" width="{jw}" height="{jh}" fill="{bone}"/><ellipse cx="{elx}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{crack}"/><ellipse cx="{erx2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{crack}"/><polygon points="{cx},{ny} {nx1},{ny2} {nx2},{ny2}" fill="{crack}"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" fill="{crack}"/>{tooth_markup}<line x1="{cx1}" y1="{cy1}" x2="{cx2}" y2="{cy2}" stroke="{crack}" stroke-width="3"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        bone = color_hex(bone),
        crack = crack,
        jx = cx - rx * 0.45,
        jy = cy + ry * 0.50,
        jw = rx * (0.82 + identity.unit_f32(26) * 0.34),
        jh = ry * (0.34 + identity.unit_f32(27) * 0.28),
        elx = cx - rx * 0.34,
        erx2 = cx + rx * 0.34,
        ey = cy - ry * 0.20,
        erx = rx * (0.18 + identity.unit_f32(29) * 0.08),
        ery = ry * (0.25 + identity.unit_f32(30) * 0.12),
        ny = cy,
        nx1 = cx - rx * 0.10,
        nx2 = cx + rx * 0.10,
        ny2 = cy + ry * (0.32 + identity.unit_f32(30) * 0.16),
        mx = cx - rx * 0.48,
        my = cy + ry * 0.50,
        mw = rx * 0.96,
        mh = h * 0.02,
        tooth_markup = tooth_markup,
        cx1 = cx - rx * 0.15,
        cy1 = cy - ry * 0.45,
        cx2 = cx + (identity.unit_f32(42) - 0.5) * rx * 0.40,
        cy2 = cy + ry * 0.10,
    )
}

