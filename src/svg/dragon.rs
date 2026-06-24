use super::*;

pub(crate) fn render_dragon_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.57;
    let rx = w * 0.27;
    let ry = h * 0.23;
    let scale = hsl_to_color(105.0 + identity.unit_f32(1) * 70.0, 0.46, 0.46);
    let belly = hsl_to_color(70.0 + identity.unit_f32(2) * 35.0, 0.42, 0.72);
    let horn = hsl_to_color(40.0 + identity.unit_f32(3) * 20.0, 0.34, 0.84);
    let flame = hsl_to_color(14.0 + identity.unit_f32(4) * 25.0, 0.78, 0.56);
    let left_horn = format!(
        "{},{} {},{} {},{}",
        cx - rx / 2.0,
        cy - ry,
        cx - rx / 4.0,
        cy - ry * 1.60,
        cx - rx / 8.0,
        cy - ry
    );
    let right_horn = format!(
        "{},{} {},{} {},{}",
        cx + rx / 2.0,
        cy - ry,
        cx + rx / 4.0,
        cy - ry * 1.60,
        cx + rx / 8.0,
        cy - ry
    );
    let spike1 = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.27,
        cy - ry,
        cx - rx * 0.20,
        cy - ry * 1.25,
        cx - rx * 0.13,
        cy - ry
    );
    let spike2 = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.07,
        cy - ry,
        cx,
        cy - ry * 1.25,
        cx + rx * 0.07,
        cy - ry
    );
    let spike3 = format!(
        "{},{} {},{} {},{}",
        cx + rx * 0.13,
        cy - ry,
        cx + rx * 0.20,
        cy - ry * 1.25,
        cx + rx * 0.27,
        cy - ry
    );
    format!(
        r##"<polygon points="{lh}" fill="{horn}"/><polygon points="{rh}" fill="{horn}"/><ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{scale}"/><ellipse cx="{cx}" cy="{my}" rx="{mrx}" ry="{mry}" fill="{belly}"/><circle cx="{lx}" cy="{ey}" r="{ew}" fill="#ffffff"/><circle cx="{rx2}" cy="{ey}" r="{ew}" fill="#ffffff"/><circle cx="{lx}" cy="{ey}" r="{pr}" fill="#183022"/><circle cx="{rx2}" cy="{ey}" r="{pr}" fill="#183022"/><circle cx="{nx1}" cy="{ny}" r="{nr}" fill="#183022"/><circle cx="{nx2}" cy="{ny}" r="{nr}" fill="#183022"/><polygon points="{s1}" fill="{flame}"/><polygon points="{s2}" fill="{flame}"/><polygon points="{s3}" fill="{flame}"/>"##,
        lh = left_horn,
        rh = right_horn,
        horn = color_hex(horn),
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        scale = color_hex(scale),
        my = cy + ry * 0.25,
        mrx = rx * 0.50,
        mry = ry * 0.34,
        belly = color_hex(belly),
        lx = cx - rx / 3.0,
        rx2 = cx + rx / 3.0,
        ey = cy - ry / 5.0,
        ew = rx * 0.10,
        pr = rx * 0.05,
        nx1 = cx - rx / 7.0,
        nx2 = cx + rx / 7.0,
        ny = cy + ry / 3.0,
        nr = rx * 0.04,
        s1 = spike1,
        s2 = spike2,
        s3 = spike3,
        flame = color_hex(flame),
    )
}
