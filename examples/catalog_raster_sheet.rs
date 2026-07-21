//! Generates a dependency-free PPM contact sheet for complete catalog review.

use std::fs;

use hashavatar::{AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const OUTPUT: &str = "hashavatar-catalog.ppm";
    const AVATAR: usize = 88;
    const TILE: usize = 96;
    const COLUMNS: usize = 6;
    let rows = AvatarKind::ALL.len().div_ceil(COLUMNS);
    let width = COLUMNS * TILE;
    let height = rows * TILE;
    let mut pixels = vec![255_u8; width * height * 3];

    for (index, kind) in AvatarKind::ALL.iter().copied().enumerate() {
        let style = AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Squircle);
        let image = AvatarRequest::new(88, 88, 7, b"catalog-sheet", style)?
            .prepare()?
            .render_rgba()?;
        let tile_x = (index % COLUMNS) * TILE + 4;
        let tile_y = (index / COLUMNS) * TILE + 4;
        for (row, source_row) in image.pixels().chunks_exact(AVATAR * 4).enumerate() {
            for (column, source) in source_row.chunks_exact(4).enumerate() {
                let Some(red) = source.first().copied() else {
                    continue;
                };
                let Some(green) = source.get(1).copied() else {
                    continue;
                };
                let Some(blue) = source.get(2).copied() else {
                    continue;
                };
                let Some(alpha) = source.get(3).copied() else {
                    continue;
                };
                let destination = ((tile_y + row) * width + tile_x + column) * 3;
                let end = destination + 3;
                let Some(pixel) = pixels.get_mut(destination..end) else {
                    continue;
                };
                let blend = |channel: u8| {
                    let alpha = u32::from(alpha);
                    u8::try_from((u32::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255)
                        .unwrap_or_default()
                };
                pixel.copy_from_slice(&[blend(red), blend(green), blend(blue)]);
            }
        }
    }

    let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
    ppm.extend_from_slice(&pixels);
    fs::write(OUTPUT, ppm)?;
    println!("wrote {OUTPUT}");
    Ok(())
}
