//! Generates an SVG contact sheet for alpha.3 catalog review.

use std::{env, fmt::Write, fs};

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle, SvgOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TILE: u32 = 112;
    const COLUMNS: u32 = 6;
    let rows = u32::try_from(AvatarKind::ALL.len())?.div_ceil(COLUMNS);
    let mut sheet = String::new();
    write!(
        sheet,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        COLUMNS * TILE,
        rows * TILE,
        COLUMNS * TILE,
        rows * TILE,
    )?;
    sheet.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>");

    for (index, kind) in AvatarKind::ALL.iter().copied().enumerate() {
        let index = u32::try_from(index)?;
        let x = (index % COLUMNS) * TILE + 8;
        let y = (index / COLUMNS) * TILE + 8;
        let style = AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Squircle);
        let prepared = AvatarRequest::new(88, 88, 7, b"catalog-sheet", style)?.prepare()?;
        let fragment = prepared.render_svg_with(SvgOptions::fragment(kind.as_str())?)?;
        write!(sheet, "<g transform=\"translate({x} {y})\">{fragment}</g>")?;
        write!(
            sheet,
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#17191f\">{}</text>",
            x + 44,
            y + 100,
            kind.as_str(),
        )?;
    }
    sheet.push_str("</svg>");

    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "hashavatar-catalog.svg".to_owned());
    fs::write(&output, sheet)?;
    println!("wrote {output}");
    Ok(())
}
