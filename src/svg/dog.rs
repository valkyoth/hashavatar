use super::*;

pub(crate) fn render_dog_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.56;
    let fur = hsl_to_color(18.0 + identity.unit_f32(5) * 45.0, 0.42, 0.60);
    let ear = hsl_to_color(18.0 + identity.unit_f32(6) * 30.0, 0.44, 0.40);
    let muzzle = hsl_to_color(34.0, 0.18, 0.92);
    format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}"/><circle cx="{}" cy="{}" r="{}" fill="#fff"/><circle cx="{}" cy="{}" r="{}" fill="#241a14"/><circle cx="{}" cy="{}" r="{}" fill="#fff"/><circle cx="{}" cy="{}" r="{}" fill="#241a14"/><ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="#2d2422"/><path d="M {} {} q {} {} {} 0 M {} {} q {} {} {} 0" stroke="#2d2422" stroke-width="3" fill="none" stroke-linecap="round"/>"##,
        cx - w * 0.14,
        cy - h * 0.03,
        w * 0.09,
        h * 0.18,
        color_hex(ear),
        cx + w * 0.14,
        cy - h * 0.03,
        w * 0.09,
        h * 0.18,
        color_hex(ear),
        cx,
        cy,
        w * 0.26,
        h * 0.24,
        color_hex(fur),
        cx,
        cy + h * 0.08,
        w * 0.12,
        h * 0.07,
        color_hex(muzzle),
        cx - w * 0.08,
        cy - h * 0.05,
        w * 0.03,
        cx - w * 0.08,
        cy - h * 0.05,
        w * 0.015,
        cx + w * 0.08,
        cy - h * 0.05,
        w * 0.03,
        cx + w * 0.08,
        cy - h * 0.05,
        w * 0.015,
        cx,
        cy + h * 0.06,
        w * 0.035,
        h * 0.026,
        cx - w * 0.03,
        cy + h * 0.09,
        w * 0.05,
        h * 0.05,
        w * 0.10,
        cx + w * 0.03,
        cy + h * 0.09,
        w * 0.05,
        h * 0.05,
        w * 0.10,
    )
}
