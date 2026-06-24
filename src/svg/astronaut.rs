fn render_astronaut_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.54;
    let r = w.min(h) * 0.28;
    let suit = hsl_to_color(205.0 + identity.unit_f32(1) * 35.0, 0.16, 0.90);
    let visor = hsl_to_color(195.0 + identity.unit_f32(2) * 55.0, 0.52, 0.56);
    let trim = hsl_to_color(identity.unit_f32(3) * 360.0, 0.45, 0.55);
    format!(
        r##"<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" fill="{suit}"/><circle cx="{cx}" cy="{cy}" r="{r}" fill="{suit}" stroke="#606e80" stroke-width="3"/><ellipse cx="{cx}" cy="{cy}" rx="{vrx}" ry="{vry}" fill="{visor}"/><rect x="{glx}" y="{gly}" width="{glw}" height="{glh}" fill="#ffffff" opacity="0.35"/><circle cx="{px}" cy="{py}" r="{pr}" fill="{trim}"/>"##,
        sx = cx - r * 0.50,
        sy = cy + r * 0.50,
        sw = r,
        sh = r * 0.60,
        suit = color_hex(suit),
        cx = cx,
        cy = cy,
        r = r,
        vrx = r * 0.67,
        vry = r * 0.50,
        visor = color_hex(visor),
        glx = cx - r / 3.0,
        gly = cy - r / 4.0,
        glw = r / 2.0,
        glh = r / 8.0,
        px = cx + r / 2.0,
        py = cy + r * 0.67,
        pr = r * 0.10,
        trim = color_hex(trim),
    )
}

