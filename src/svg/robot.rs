fn render_robot_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let metal = hsl_to_color(205.0 + identity.unit_f32(3) * 25.0, 0.16, 0.74);
    let trim = hsl_to_color(205.0 + identity.unit_f32(4) * 22.0, 0.18, 0.46);
    let light = hsl_to_color(60.0 + identity.unit_f32(5) * 110.0, 0.78, 0.66);
    let head_w = w * 0.48;
    let head_h = h * 0.38;
    let x = cx - head_w / 2.0;
    let y = cy - head_h / 2.0;
    format!(
        r##"<line x1="{cx}" y1="{a1}" x2="{cx}" y2="{a2}" stroke="{trim}" stroke-width="4"/><circle cx="{cx}" cy="{a1}" r="{ar}" fill="{light}"/><rect x="{x}" y="{y}" width="{head_w}" height="{head_h}" rx="14" fill="{metal}" stroke="{trim}" stroke-width="4"/><ellipse cx="{ex1}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{light}"/><ellipse cx="{ex2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="{light}"/><rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="6" fill="#2f3c48"/><circle cx="{bx1}" cy="{cy}" r="{br}" fill="{trim}"/><circle cx="{bx2}" cy="{cy}" r="{br}" fill="{trim}"/>"##,
        cx = cx,
        a1 = y - h * 0.10,
        a2 = y,
        ar = w * 0.02,
        x = x,
        y = y,
        head_w = head_w,
        head_h = head_h,
        metal = color_hex(metal),
        trim = color_hex(trim),
        light = color_hex(light),
        ex1 = cx - head_w * 0.24,
        ex2 = cx + head_w * 0.24,
        ey = cy - head_h * 0.14,
        erx = w * 0.055,
        ery = h * 0.04,
        mx = cx - head_w * 0.18,
        my = cy + head_h * 0.12,
        mw = head_w * 0.36,
        mh = head_h * 0.10,
        bx1 = x + head_w * 0.1,
        bx2 = x + head_w * 0.9,
        br = w * 0.02,
    )
}

