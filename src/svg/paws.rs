use super::*;

pub(crate) fn render_paws_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let paw = hsl_to_color(identity.unit_f32(1) * 360.0, 0.38, 0.62);
    let pad = hsl_to_color(330.0 + identity.unit_f32(3) * 20.0, 0.40, 0.74);
    let cx = w * 0.52;
    let cy = h * 0.60;
    format!(
        r##"<ellipse cx="{cx}" cy="{cy}" rx="{prx}" ry="{pry}" fill="{paw}"/><ellipse cx="{cx}" cy="{py2}" rx="{padrx}" ry="{padry}" fill="{pad}"/><ellipse cx="{t1x}" cy="{ty1}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t2x}" cy="{ty2}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t3x}" cy="{ty2}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t4x}" cy="{ty1}" rx="{trx}" ry="{try_}" fill="{paw}"/><ellipse cx="{t1x}" cy="{ty1a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/><ellipse cx="{t2x}" cy="{ty2a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/><ellipse cx="{t3x}" cy="{ty2a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/><ellipse cx="{t4x}" cy="{ty1a}" rx="{padrx2}" ry="{padry2}" fill="{pad}"/>"##,
        cx = cx,
        cy = cy,
        prx = w * 0.13,
        pry = h * 0.15,
        paw = color_hex(paw),
        py2 = cy + h * 0.015,
        padrx = w * 0.09,
        padry = h * 0.10,
        pad = color_hex(pad),
        t1x = cx - w * 0.12,
        t2x = cx - w * 0.04,
        t3x = cx + w * 0.04,
        t4x = cx + w * 0.12,
        ty1 = cy - h * 0.16,
        ty2 = cy - h * 0.14,
        ty1a = cy - h * 0.15,
        ty2a = cy - h * 0.13,
        trx = w * 0.035,
        try_ = h * 0.05,
        padrx2 = w * 0.022,
        padry2 = h * 0.032,
    )
}
