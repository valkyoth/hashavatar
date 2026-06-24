fn render_coffee_cup_svg(spec: AvatarSpec, identity: &AvatarIdentity) -> String {
    let w = spec.width as f32;
    let h = spec.height as f32;
    let cx = w / 2.0;
    let cy = h * 0.58;
    let cw = w * 0.38;
    let ch = h * 0.32;
    let cup = hsl_to_color(20.0 + identity.unit_f32(1) * 35.0, 0.42, 0.60);
    let coffee = hsl_to_color(24.0 + identity.unit_f32(2) * 18.0, 0.42, 0.26);
    format!(
        r##"<rect x="{x}" y="{y}" width="{cw}" height="{ch}" fill="{cup}"/><ellipse cx="{cx}" cy="{top}" rx="{rx}" ry="{ery}" fill="{coffee}"/><ellipse cx="{hx}" cy="{hy}" rx="{hrx}" ry="{hry}" fill="none" stroke="{cup}" stroke-width="5"/><rect x="{px}" y="{py}" width="{pw}" height="{ph}" fill="#50372a" opacity="0.7"/><path d="M {s1} {sy} L {s1e} {sy2} M {s2} {sy} L {s2e} {sy2} M {s3} {sy} L {s3e} {sy2}" stroke="#786252" stroke-width="3" opacity="0.55" stroke-linecap="round"/>"##,
        x = cx - cw / 2.0,
        y = cy - ch / 2.0,
        cw = cw,
        ch = ch,
        cup = color_hex(cup),
        cx = cx,
        top = cy - ch / 2.0,
        rx = cw / 2.0,
        ery = ch / 7.0,
        coffee = color_hex(coffee),
        hx = cx + cw / 2.0,
        hy = cy - ch / 10.0,
        hrx = cw / 4.0,
        hry = ch / 4.0,
        px = cx - cw * 0.60,
        py = cy + ch / 2.0,
        pw = cw * 1.20,
        ph = ch / 8.0,
        s1 = cx - cw / 5.0,
        s2 = cx,
        s3 = cx + cw / 5.0,
        sy = cy - ch,
        s1e = cx - cw / 10.0,
        s2e = cx + cw / 10.0,
        s3e = cx + cw * 0.30,
        sy2 = cy - ch * 1.33,
    )
}

