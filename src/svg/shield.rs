fn render_shield_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.55;
    let rx = w * 0.25;
    let ry = h * 0.32;
    let metal = hsl_to_color(210.0 + identity.unit_f32(1) * 45.0, 0.28, 0.58);
    let accent = hsl_to_color(identity.unit_f32(2) * 360.0, 0.50, 0.50);
    let light = hsl_to_color(210.0 + identity.unit_f32(3) * 35.0, 0.18, 0.82);
    let outer = format!(
        "{},{} {},{} {},{} {},{} {},{}",
        cx - rx,
        cy - ry,
        cx + rx,
        cy - ry,
        cx + rx * 0.80,
        cy + ry * 0.25,
        cx,
        cy + ry,
        cx - rx * 0.80,
        cy + ry * 0.25
    );
    let left = format!(
        "{},{} {},{} {},{} {},{}",
        cx - rx,
        cy - ry,
        cx,
        cy - ry,
        cx,
        cy + ry,
        cx - rx * 0.80,
        cy + ry * 0.25
    );
    format!(
        r##"<polygon points="{outer}" fill="{metal}"/><polygon points="{left}" fill="{light}"/><rect x="{vx}" y="{vy}" width="{vw}" height="{vh}" fill="{accent}"/><rect x="{hx}" y="{hy}" width="{hw}" height="{hh}" fill="{accent}"/>"##,
        outer = outer,
        metal = color_hex(metal),
        left = left,
        light = color_hex(light),
        vx = cx - rx * 0.125,
        vy = cy - ry * 0.75,
        vw = rx * 0.25,
        vh = ry * 1.20,
        hx = cx - rx * 0.67,
        hy = cy - ry * 0.20,
        hw = rx * 1.34,
        hh = ry * 0.25,
        accent = color_hex(accent),
    )
}

