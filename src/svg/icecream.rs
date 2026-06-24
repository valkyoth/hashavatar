use super::*;

pub(crate) fn render_icecream_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.55;
    let scoop_r = w * (0.18 + identity.unit_f32(4) * 0.06);
    let cone_w = w * (0.24 + identity.unit_f32(5) * 0.05);
    let cone_h = h * (0.32 + identity.unit_f32(6) * 0.06);
    let scoop_y = cy - scoop_r / 2.0;
    let cone_top = scoop_y + scoop_r / 2.0;
    let scoop = hsl_to_color(identity.unit_f32(1) * 360.0, 0.42, 0.76);
    let cone = hsl_to_color(32.0 + identity.unit_f32(2) * 22.0, 0.50, 0.64);
    let waffle = hsl_to_color(28.0 + identity.unit_f32(3) * 22.0, 0.42, 0.45);
    format!(
        r##"<polygon points="{x1},{ct} {x2},{ct} {cx},{cb}" fill="{cone}"/><line x1="{lx1}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="{waffle}" stroke-width="3"/><line x1="{lx2}" y1="{ly1}" x2="{cx}" y2="{ly2}" stroke="{waffle}" stroke-width="3"/><circle cx="{cx}" cy="{sy}" r="{sr}" fill="{scoop}"/><circle cx="{d1x}" cy="{d1y}" r="{dr1}" fill="{scoop}"/><circle cx="{d2x}" cy="{d2y}" r="{dr2}" fill="{scoop}"/><circle cx="{c1x}" cy="{c1y}" r="{chip}" fill="{waffle}"/><circle cx="{c2x}" cy="{c2y}" r="{chip}" fill="{waffle}"/>"##,
        cx = cx,
        x1 = cx - cone_w / 2.0,
        x2 = cx + cone_w / 2.0,
        ct = cone_top,
        cb = cone_top + cone_h,
        cone = color_hex(cone),
        lx1 = cx - cone_w / 3.0,
        lx2 = cx + cone_w / 3.0,
        ly1 = cone_top + cone_h / 8.0,
        ly2 = cone_top + cone_h * 0.75,
        waffle = color_hex(waffle),
        sy = scoop_y,
        sr = scoop_r,
        scoop = color_hex(scoop),
        d1x = cx - scoop_r / 2.0,
        d1y = scoop_y + scoop_r / 3.0,
        dr1 = scoop_r / 5.0,
        d2x = cx + scoop_r / 3.0,
        d2y = scoop_y + scoop_r / 2.0,
        dr2 = scoop_r / 6.0,
        c1x = cx - scoop_r * 0.25,
        c1y = scoop_y - scoop_r * 0.18,
        c2x = cx + scoop_r * 0.18,
        c2y = scoop_y + scoop_r * 0.12,
        chip = w * 0.010,
    )
}
