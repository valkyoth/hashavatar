use super::*;

pub(crate) fn render_penguin_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let rx = w * 0.25;
    let ry = h * 0.34;
    let black = hsl_to_color(210.0 + identity.unit_f32(1) * 30.0, 0.22, 0.18);
    let white = hsl_to_color(205.0 + identity.unit_f32(2) * 25.0, 0.16, 0.94);
    let orange = hsl_to_color(32.0 + identity.unit_f32(3) * 18.0, 0.72, 0.58);
    let beak_points = format!(
        "{},{} {},{} {},{}",
        cx - rx * 0.14,
        cy - ry * 0.16,
        cx + rx * 0.14,
        cy - ry * 0.16,
        cx,
        cy
    );
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{black}"/><ellipse cx="{cx}" cy="{by}" rx="{brx}" ry="{bry}" fill="{white}"/><ellipse cx="{lx}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{black}"/><ellipse cx="{rx2}" cy="{wy}" rx="{wrx}" ry="{wry}" fill="{black}"/><circle cx="{ex1}" cy="{ey}" r="{er}" fill="#0a0f14"/><circle cx="{ex2}" cy="{ey}" r="{er}" fill="#0a0f14"/><polygon points="{bp}" fill="{orange}"/><ellipse cx="{fx1}" cy="{fy}" rx="{frx}" ry="{fry}" fill="{orange}"/><ellipse cx="{fx2}" cy="{fy}" rx="{frx}" ry="{fry}" fill="{orange}"/>"##,
        cx = cx,
        cy = cy,
        rx = rx,
        ry = ry,
        black = color_hex(black),
        by = cy + ry / 6.0,
        brx = rx * 0.60,
        bry = ry * 0.67,
        white = color_hex(white),
        lx = cx - rx,
        rx2 = cx + rx,
        wy = cy + ry * 0.10,
        wrx = rx * 0.25,
        wry = ry * 0.50,
        ex1 = cx - rx / 3.0,
        ex2 = cx + rx / 3.0,
        ey = cy - ry / 3.0,
        er = rx * 0.10,
        bp = beak_points,
        orange = color_hex(orange),
        fx1 = cx - rx / 3.0,
        fx2 = cx + rx / 3.0,
        fy = cy + ry,
        frx = rx * 0.25,
        fry = ry * 0.10,
    )
}
