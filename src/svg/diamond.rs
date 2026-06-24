use super::*;

pub(crate) fn render_diamond_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let rx = w * 0.25;
    let ry = h * 0.30;
    let gem = hsl_to_color(180.0 + identity.unit_f32(1) * 95.0, 0.55, 0.60);
    let highlight = hsl_to_color(190.0 + identity.unit_f32(2) * 70.0, 0.40, 0.82);
    let shade = hsl_to_color(200.0 + identity.unit_f32(3) * 70.0, 0.42, 0.42);
    let outer = format!(
        "{},{} {},{} {},{} {},{} {},{}",
        cx - rx,
        cy - ry / 3.0,
        cx - rx / 2.0,
        cy - ry,
        cx + rx / 2.0,
        cy - ry,
        cx + rx,
        cy - ry / 3.0,
        cx,
        cy + ry
    );
    let left = format!(
        "{},{} {},{} {},{} {},{}",
        cx - rx,
        cy - ry / 3.0,
        cx - rx / 2.0,
        cy - ry,
        cx,
        cy + ry,
        cx - rx / 5.0,
        cy - ry / 3.0
    );
    let right = format!(
        "{},{} {},{} {},{} {},{}",
        cx + rx / 2.0,
        cy - ry,
        cx + rx,
        cy - ry / 3.0,
        cx,
        cy + ry,
        cx + rx / 5.0,
        cy - ry / 3.0
    );
    format!(
        r##"<polygon points="{outer}" fill="{gem}"/><polygon points="{left}" fill="{highlight}"/><polygon points="{right}" fill="{shade}"/><line x1="{l1}" y1="{top}" x2="{cx}" y2="{bottom}" stroke="#ffffff" stroke-width="2" opacity="0.45"/><line x1="{cx}" y1="{top}" x2="{cx}" y2="{bottom}" stroke="#ffffff" stroke-width="2" opacity="0.45"/><line x1="{r1}" y1="{top}" x2="{cx}" y2="{bottom}" stroke="#ffffff" stroke-width="2" opacity="0.45"/>"##,
        outer = outer,
        gem = color_hex(gem),
        left = left,
        right = right,
        highlight = color_hex(highlight),
        shade = color_hex(shade),
        l1 = cx - rx / 2.0,
        r1 = cx + rx / 2.0,
        top = cy - ry,
        bottom = cy + ry,
        cx = cx,
    )
}
