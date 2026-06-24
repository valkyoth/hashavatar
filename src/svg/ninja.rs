fn render_ninja_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let r = w.min(h) * 0.28;
    let cloth = hsl_to_color(220.0 + identity.unit_f32(1) * 50.0, 0.18, 0.14);
    let skin = hsl_to_color(28.0 + identity.unit_f32(2) * 18.0, 0.42, 0.72);
    let band = hsl_to_color(identity.unit_f32(3) * 360.0, 0.56, 0.50);
    let tie = format!(
        "{},{} {},{} {},{}",
        cx + r * 0.80,
        cy - r * 0.67,
        cx + r * 1.40,
        cy - r,
        cx + r,
        cy - r * 0.33
    );
    format!(
        r##"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{cloth}"/><rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="{skin}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{band}"/><ellipse cx="{ex1}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#141820"/><ellipse cx="{ex2}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#141820"/><polygon points="{tie}" fill="{band}"/>"##,
        cx = cx,
        cy = cy,
        r = r,
        cloth = color_hex(cloth),
        sx = cx - r * 0.60,
        sy = cy - r * 0.25,
        sw = r * 1.20,
        sh = r * 0.50,
        skin = color_hex(skin),
        bx = cx - r,
        by = cy - r * 0.67,
        bw = r * 2.0,
        bh = r * 0.17,
        band = color_hex(band),
        ex1 = cx - r / 3.0,
        ex2 = cx + r / 3.0,
        ey = cy - r * 0.08,
        erx = r * 0.11,
        ery = r * 0.07,
        tie = tie,
    )
}

