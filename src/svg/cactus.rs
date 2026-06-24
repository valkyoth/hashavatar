use super::*;

pub(crate) fn render_cactus_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let body_w = w * (0.13 + identity.unit_f32(4) * 0.04);
    let body_h = h * (0.36 + identity.unit_f32(5) * 0.10);
    let top_y = cy - body_h / 2.0;
    let cactus = hsl_to_color(105.0 + identity.unit_f32(1) * 60.0, 0.42, 0.42);
    let flower = hsl_to_color(320.0 + identity.unit_f32(3) * 55.0, 0.58, 0.64);
    format!(
        r##"<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" rx="{br}" fill="{cactus}"/><circle cx="{cx}" cy="{by}" r="{br}" fill="{cactus}"/><rect x="{lax}" y="{lay}" width="{al}" height="{ah}" rx="{ar}" fill="{cactus}"/><circle cx="{lax}" cy="{lcy}" r="{ar}" fill="{cactus}"/><rect x="{rax}" y="{ray}" width="{al}" height="{ah}" rx="{ar}" fill="{cactus}"/><circle cx="{rcx}" cy="{rcy}" r="{ar}" fill="{cactus}"/><line x1="{n1x}" y1="{n1y}" x2="{n2x}" y2="{n2y}" stroke="#f2ffe0" stroke-width="2" opacity="0.7"/><line x1="{n3x}" y1="{n3y}" x2="{n4x}" y2="{n4y}" stroke="#f2ffe0" stroke-width="2" opacity="0.7"/><circle cx="{cx}" cy="{fy}" r="{fr}" fill="{flower}"/>"##,
        bx = cx - body_w / 2.0,
        by = top_y,
        bw = body_w,
        bh = body_h,
        br = body_w / 2.0,
        cactus = color_hex(cactus),
        lax = cx - body_w * 1.45,
        lay = cy - body_h * 0.13,
        lcy = cy - body_h * 0.13 + body_w * 0.25,
        rax = cx + body_w * 0.35,
        ray = cy - body_h * 0.23,
        rcx = cx + body_w * 1.45,
        rcy = cy - body_h * 0.23 + body_w * 0.25,
        al = body_w * 1.10,
        ah = body_w * 0.50,
        ar = body_w * 0.25,
        n1x = cx - body_w * 0.12,
        n1y = top_y + body_h * 0.34,
        n2x = cx - body_w * 0.34,
        n2y = top_y + body_h * 0.29,
        n3x = cx + body_w * 0.10,
        n3y = top_y + body_h * 0.58,
        n4x = cx + body_w * 0.33,
        n4y = top_y + body_h * 0.54,
        fy = top_y - body_w * 0.42,
        fr = body_w * (0.16 + identity.unit_f32(12) * 0.08),
        flower = color_hex(flower),
    )
}
