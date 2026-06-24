use super::*;

pub(crate) fn render_rocket_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.52;
    let body_w = w * (0.18 + identity.unit_f32(5) * 0.05);
    let body_h = h * (0.42 + identity.unit_f32(6) * 0.08);
    let top_y = cy - body_h / 2.0;
    let bottom_y = cy + body_h / 2.0;
    let hull = hsl_to_color(200.0 + identity.unit_f32(1) * 50.0, 0.12, 0.88);
    let trim = hsl_to_color(identity.unit_f32(2) * 360.0, 0.58, 0.54);
    let window = hsl_to_color(185.0 + identity.unit_f32(3) * 70.0, 0.54, 0.72);
    let flame = hsl_to_color(20.0 + identity.unit_f32(4) * 30.0, 0.86, 0.58);
    let mut windows = String::new();
    for index in 0..(1 + (identity.byte(7) % 2) as usize) {
        let wy = top_y + body_h / 3.0 + index as f32 * body_w;
        windows.push_str(&format!(
            r##"<circle cx="{cx}" cy="{wy}" r="{wr}" fill="{trim}"/><circle cx="{cx}" cy="{wy}" r="{ir}" fill="{window}"/>"##,
            wr = body_w * 0.22,
            ir = body_w * 0.15,
            trim = color_hex(trim),
            window = color_hex(window),
        ));
    }
    format!(
        r##"<polygon points="{nx1},{ny1} {nx2},{ny1} {cx},{ny2}" fill="{trim}"/><rect x="{bx}" y="{by}" width="{bw}" height="{bh}" fill="{hull}"/><ellipse cx="{cx}" cy="{by}" rx="{erx}" ry="{ery}" fill="{hull}"/><polygon points="{lf1},{lfy1} {lf2},{lfy2} {lf3},{lfy3}" fill="{trim}"/><polygon points="{rf1},{lfy1} {rf2},{lfy2} {rf3},{lfy3}" fill="{trim}"/>{windows}<polygon points="{fx1},{fy1} {fx2},{fy1} {cx},{fy2}" fill="{flame}"/>"##,
        cx = cx,
        nx1 = cx - body_w / 2.0,
        nx2 = cx + body_w / 2.0,
        ny1 = top_y + body_w / 2.0,
        ny2 = top_y - body_w / 2.0,
        trim = color_hex(trim),
        bx = cx - body_w / 2.0,
        by = top_y + body_w / 2.0,
        bw = body_w,
        bh = body_h - body_w / 2.0,
        hull = color_hex(hull),
        erx = body_w / 2.0,
        ery = body_w / 5.0,
        lf1 = cx - body_w / 2.0,
        lf2 = cx - body_w,
        lf3 = cx - body_w / 2.0,
        rf1 = cx + body_w / 2.0,
        rf2 = cx + body_w,
        rf3 = cx + body_w / 2.0,
        lfy1 = bottom_y - body_w / 2.0,
        lfy2 = bottom_y + body_w / 3.0,
        lfy3 = bottom_y,
        windows = windows,
        fx1 = cx - body_w / 4.0,
        fx2 = cx + body_w / 4.0,
        fy1 = bottom_y,
        fy2 = bottom_y + h * (0.10 + identity.unit_f32(8) * 0.08),
        flame = color_hex(flame),
    )
}
