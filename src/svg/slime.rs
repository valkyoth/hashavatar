use super::*;

pub(crate) fn render_slime_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * (0.56 + identity.unit_f32(8) * 0.08);
    let rx = w * (0.20 + identity.unit_f32(6) * 0.10);
    let ry = h * (0.16 + identity.unit_f32(7) * 0.08);
    let slime = hsl_to_color(
        70.0 + identity.unit_f32(4) * 130.0,
        0.44 + identity.unit_f32(9) * 0.22,
        0.46 + identity.unit_f32(10) * 0.18,
    );
    let eye_count = 1 + (identity.byte(49) % 3) as usize;
    let eye_markup = match eye_count {
        1 => format!(
            r##"<circle cx="{cx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{cx}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            ey = cy - ry * 0.20,
            er = rx * 0.14,
            pr = rx * 0.055,
        ),
        2 => format!(
            r##"<circle cx="{lx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{rx2}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#203018"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            lx = cx - rx * 0.34,
            rx2 = cx + rx * 0.34,
            ey = cy - ry * 0.22,
            er = rx * 0.12,
            pr = rx * 0.050,
        ),
        _ => format!(
            r##"<circle cx="{lx}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{cx}" cy="{ey2}" r="{er2}" fill="#f8ffec"/><circle cx="{rx2}" cy="{ey}" r="{er}" fill="#f8ffec"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#203018"/><circle cx="{cx}" cy="{ey2}" r="{pr}" fill="#203018"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#203018"/>"##,
            lx = cx - rx * 0.34,
            rx2 = cx + rx * 0.34,
            ey = cy - ry * 0.26,
            ey2 = cy - ry * 0.14,
            er = rx * 0.11,
            er2 = rx * 0.095,
            pr = rx * 0.045,
        ),
    };
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{slime}"/><rect x="{dx1}" y="{cy}" width="{dw1}" height="{dh1}" fill="{slime}"/><rect x="{dx2}" y="{cy}" width="{dw2}" height="{dh2}" fill="{slime}"/><rect x="{dx3}" y="{cy}" width="{dw3}" height="{dh3}" fill="{slime}"/>{eye_markup}<rect x="{mx}" y="{my}" width="{mw}" height="{mh}" rx="{mr}" fill="#305228"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        slime = color_hex(slime),
        dx1 = cx - rx * 0.66,
        dx2 = cx - rx * 0.14,
        dx3 = cx + rx * 0.34,
        dw1 = rx * (0.24 + identity.unit_f32(14) * 0.12),
        dw2 = rx * (0.20 + identity.unit_f32(15) * 0.14),
        dw3 = rx * (0.22 + identity.unit_f32(16) * 0.14),
        dh1 = ry * (0.62 + identity.unit_f32(22) * 0.60),
        dh2 = ry * (0.42 + identity.unit_f32(23) * 0.55),
        dh3 = ry * (0.54 + identity.unit_f32(24) * 0.62),
        eye_markup = eye_markup,
        mx = cx - rx * 0.42,
        my = cy + ry * 0.40,
        mw = rx * 0.84,
        mh = h * 0.02,
        mr = w * 0.01,
    )
}
