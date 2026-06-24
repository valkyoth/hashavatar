fn render_monster_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let skin = hsl_to_color(
        identity.unit_f32(0) * 360.0,
        0.52 + identity.unit_f32(1) * 0.20,
        0.50 + identity.unit_f32(2) * 0.16,
    );
    let shade = hsl_to_color(
        identity.unit_f32(3) * 360.0,
        0.40 + identity.unit_f32(4) * 0.16,
        0.26 + identity.unit_f32(5) * 0.08,
    );
    let eyes = 1 + (identity.byte(12) % 3) as usize;
    let eye_spacing = if eyes == 1 {
        0.0
    } else {
        w * 0.22 / (eyes - 1) as f32
    };
    let eye_start = cx - eye_spacing * (eyes.saturating_sub(1) as f32) / 2.0;
    let mut eye_markup = String::new();
    for index in 0..eyes {
        let ex = eye_start + eye_spacing * index as f32;
        eye_markup.push_str(&format!(
            r##"<ellipse cx="{ex}" cy="{ey}" rx="{erx}" ry="{ery}" fill="#fcf8ec"/><ellipse cx="{ex}" cy="{ey}" rx="{prx}" ry="{pry}" fill="#18141c"/>"##,
            ey = cy - h * 0.08,
            erx = w * 0.038,
            ery = h * 0.042,
            prx = w * 0.012,
            pry = h * 0.030,
        ));
    }

    let horns = if identity.byte(18).is_multiple_of(2) {
        format!(
            r#"<polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/>"#,
            cx - w * 0.18,
            cy - h * 0.18,
            cx - w * 0.22,
            cy - h * 0.34,
            cx - w * 0.08,
            cy - h * 0.14,
            color_hex(shade),
            cx + w * 0.18,
            cy - h * 0.18,
            cx + w * 0.22,
            cy - h * 0.34,
            cx + w * 0.08,
            cy - h * 0.14,
            color_hex(shade),
        )
    } else {
        format!(
            r#"<polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/><polygon points="{},{}, {},{}, {},{}" fill="{}"/>"#,
            cx - w * 0.12,
            cy - h * 0.14,
            cx - w * 0.08,
            cy - h * 0.30,
            cx - w * 0.02,
            cy - h * 0.14,
            color_hex(shade),
            cx,
            cy - h * 0.15,
            cx,
            cy - h * 0.32,
            cx + w * 0.05,
            cy - h * 0.15,
            color_hex(shade),
            cx + w * 0.12,
            cy - h * 0.14,
            cx + w * 0.08,
            cy - h * 0.30,
            cx + w * 0.02,
            cy - h * 0.14,
            color_hex(shade),
        )
    };

    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{skin}"/>{horns}<circle cx="{sx1}" cy="{sy1}" r="{sr}" fill="{shade}" fill-opacity="0.55"/><circle cx="{sx2}" cy="{sy2}" r="{sr2}" fill="{shade}" fill-opacity="0.55"/>{eye_markup}<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="{mr}" fill="#301218"/><polygon points="{tx1},{ty1}, {tx2},{ty1}, {txm1},{ty2}" fill="#fcf8ec"/><polygon points="{tx3},{ty1}, {tx4},{ty1}, {txm2},{ty2}" fill="#fcf8ec"/>"##,
        cx = cx,
        cy = cy,
        rx = w * 0.24,
        ry = h * 0.23,
        skin = color_hex(skin),
        horns = horns,
        shade = color_hex(shade),
        sx1 = cx - w * 0.12,
        sy1 = cy - h * 0.02,
        sr = w * 0.034,
        sx2 = cx + w * 0.14,
        sy2 = cy + h * 0.07,
        sr2 = w * 0.026,
        eye_markup = eye_markup,
        mx = cx - w * 0.14,
        my = cy + h * 0.08,
        mw = w * 0.28,
        mh = h * 0.09,
        mr = w * 0.02,
        tx1 = cx - w * 0.10,
        tx2 = cx - w * 0.06,
        txm1 = cx - w * 0.08,
        tx3 = cx + w * 0.06,
        tx4 = cx + w * 0.10,
        txm2 = cx + w * 0.08,
        ty1 = cy + h * 0.08,
        ty2 = cy + h * 0.16,
    )
}

