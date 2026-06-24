fn render_bird_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let plumage = hsl_to_color(identity.unit_f32(7) * 360.0, 0.42, 0.62);
    let wing = hsl_to_color(20.0 + identity.unit_f32(8) * 160.0, 0.32, 0.46);
    let beak = hsl_to_color(32.0 + identity.unit_f32(9) * 26.0, 0.82, 0.58);
    format!(
        r##"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{plumage}"/><ellipse cx="{lx}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{wing}"/><ellipse cx="{rx2}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{wing}"/><polygon points="{cx},{cy} {bx},{by} {cx},{by2}" fill="{beak}"/><circle cx="{elx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{erx}" cy="{ey}" r="{er}" fill="#fff"/><circle cx="{elx}" cy="{ey}" r="{pr}" fill="#181822"/><circle cx="{erx}" cy="{ey}" r="{pr}" fill="#181822"/>"##,
        cx = cx,
        cy = cy,
        r = w * 0.22,
        plumage = color_hex(plumage),
        lx = cx - w * 0.12,
        rx2 = cx + w * 0.12,
        wy = cy + h * 0.04,
        wrx = w * 0.08,
        wry = h * 0.12,
        wing = color_hex(wing),
        bx = cx + w * 0.12,
        by = cy + h * 0.04,
        by2 = cy + h * 0.10,
        beak = color_hex(beak),
        elx = cx - w * 0.07,
        erx = cx + w * 0.07,
        ey = cy - h * 0.05,
        er = w * 0.028,
        pr = w * 0.012,
    )
}

