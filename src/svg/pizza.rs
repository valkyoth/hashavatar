use super::*;

pub(crate) fn render_pizza_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.53;
    let half_w = w * (0.22 + identity.unit_f32(5) * 0.06);
    let slice_h = h * (0.44 + identity.unit_f32(6) * 0.07);
    let top_y = cy - slice_h / 2.0;
    let tip_y = cy + slice_h / 2.0;
    let crust = hsl_to_color(30.0 + identity.unit_f32(1) * 28.0, 0.54, 0.58);
    let cheese = hsl_to_color(45.0 + identity.unit_f32(2) * 16.0, 0.74, 0.70);
    let sauce = hsl_to_color(8.0 + identity.unit_f32(3) * 16.0, 0.62, 0.48);
    let topping = hsl_to_color(350.0 + identity.unit_f32(4) * 22.0, 0.54, 0.46);
    format!(
        r##"<polygon points="{x1},{ty} {x2},{ty} {cx},{tip}" fill="{cheese}"/><polygon points="{sx1},{sy} {sx2},{sy} {cx},{stip}" fill="{sauce}" opacity="0.38"/><ellipse cx="{cx}" cy="{ty}" rx="{half_w}" ry="{crh}" fill="{crust}"/><circle cx="{p1x}" cy="{p1y}" r="{pr}" fill="{topping}"/><circle cx="{p2x}" cy="{p2y}" r="{pr2}" fill="{topping}"/><circle cx="{p3x}" cy="{p3y}" r="{pr3}" fill="{topping}"/>"##,
        cx = cx,
        x1 = cx - half_w,
        x2 = cx + half_w,
        ty = top_y,
        tip = tip_y,
        cheese = color_hex(cheese),
        sx1 = cx - half_w * 0.78,
        sx2 = cx + half_w * 0.78,
        sy = top_y + slice_h * 0.20,
        stip = tip_y - slice_h * 0.10,
        sauce = color_hex(sauce),
        half_w = half_w,
        crh = h * 0.035,
        crust = color_hex(crust),
        p1x = cx - half_w * 0.35,
        p2x = cx + half_w * 0.28,
        p3x = cx,
        p1y = top_y + slice_h * 0.28,
        p2y = top_y + slice_h * 0.40,
        p3y = top_y + slice_h * 0.62,
        pr = w * (0.025 + identity.unit_f32(14) * 0.012),
        pr2 = w * (0.026 + identity.unit_f32(15) * 0.012),
        pr3 = w * (0.024 + identity.unit_f32(16) * 0.012),
        topping = color_hex(topping),
    )
}
