fn render_cupcake_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let cup_w = w * (0.26 + identity.unit_f32(5) * 0.07);
    let cup_h = h * (0.22 + identity.unit_f32(6) * 0.05);
    let frx = cup_w * (0.58 + identity.unit_f32(7) * 0.10);
    let fry = h * (0.13 + identity.unit_f32(8) * 0.04);
    let wrapper = hsl_to_color(28.0 + identity.unit_f32(1) * 35.0, 0.46, 0.62);
    let frosting = hsl_to_color(identity.unit_f32(2) * 360.0, 0.38, 0.78);
    let cherry = hsl_to_color(345.0 + identity.unit_f32(4) * 22.0, 0.66, 0.50);
    let by = cy - fry / 2.0;
    format!(
        r##"<polygon points="{x1},{cy} {x2},{cy} {x3},{yb} {x4},{yb}" fill="{wrapper}"/><line x1="{sx1}" y1="{cy}" x2="{sx2}" y2="{yb}" stroke="#fff4d6" stroke-width="3" opacity="0.45"/><line x1="{sx3}" y1="{cy}" x2="{sx4}" y2="{yb}" stroke="#fff4d6" stroke-width="3" opacity="0.45"/><ellipse cx="{cx}" cy="{f1y}" rx="{frx}" ry="{fry}" fill="{frosting}"/><ellipse cx="{cx}" cy="{f2y}" rx="{frx2}" ry="{fry2}" fill="{frosting}"/><ellipse cx="{cx}" cy="{f3y}" rx="{frx3}" ry="{fry3}" fill="{frosting}"/><rect x="{spx1}" y="{spy1}" width="{spw}" height="{sph}" fill="#f05f7e"/><rect x="{spx2}" y="{spy2}" width="{spw}" height="{sph}" fill="#5fb6f0"/><rect x="{spx3}" y="{spy3}" width="{spw}" height="{sph}" fill="#f0d15f"/><circle cx="{cx}" cy="{chy}" r="{chr}" fill="{cherry}"/>"##,
        cx = cx,
        cy = cy,
        x1 = cx - cup_w / 2.0,
        x2 = cx + cup_w / 2.0,
        x3 = cx + cup_w / 3.0,
        x4 = cx - cup_w / 3.0,
        yb = cy + cup_h,
        wrapper = color_hex(wrapper),
        sx1 = cx - cup_w * 0.25,
        sx2 = cx - cup_w * 0.16,
        sx3 = cx + cup_w * 0.25,
        sx4 = cx + cup_w * 0.16,
        f1y = by,
        f2y = by - fry * 0.50,
        f3y = by - fry,
        frx = frx,
        fry = fry,
        frx2 = frx * 0.78,
        fry2 = fry * 0.72,
        frx3 = frx * 0.56,
        fry3 = fry * 0.62,
        frosting = color_hex(frosting),
        spx1 = cx - frx * 0.35,
        spx2 = cx + frx * 0.10,
        spx3 = cx - frx * 0.02,
        spy1 = by - fry * 0.50,
        spy2 = by - fry * 0.88,
        spy3 = by - fry * 0.12,
        spw = w * 0.035,
        sph = h * 0.012,
        chy = by - fry,
        chr = w * 0.035,
        cherry = color_hex(cherry),
    )
}

