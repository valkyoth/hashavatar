fn render_alien_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let skin = hsl_to_color(90.0 + identity.unit_f32(0) * 80.0, 0.48, 0.70);
    format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#261832"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#261832"/><circle cx="{}" cy="{}" r="{}" fill="#5e8c58"/>"##,
        cx,
        cy,
        w * 0.18,
        h * 0.28,
        color_hex(skin),
        cx - w * 0.08,
        cy - h * 0.07,
        w * 0.04,
        h * 0.09,
        cx + w * 0.08,
        cy - h * 0.07,
        w * 0.04,
        h * 0.09,
        cx,
        cy + h * 0.03,
        w * 0.012,
    )
}

