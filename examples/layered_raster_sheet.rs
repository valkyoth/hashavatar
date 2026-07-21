//! Generates a PPM corpus for alpha.4 accessory and expression review.

use std::{env, fs};

use hashavatar::{
    AvatarAccessory, AvatarBackground, AvatarExpression, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const TILE: usize = 68;
    let families: Vec<_> = AvatarKind::ALL
        .iter()
        .copied()
        .filter(|kind| kind.capabilities().has_face_anchors())
        .collect();
    let columns = AvatarAccessory::ALL.len() + AvatarExpression::ALL.len();
    let width = columns * TILE;
    let height = families.len() * TILE;
    let mut pixels = vec![255_u8; width * height * 3];

    for (row, kind) in families.iter().copied().enumerate() {
        for (column, accessory) in AvatarAccessory::ALL.iter().copied().enumerate() {
            let style = AvatarStyle::new(kind, AvatarBackground::Light, AvatarShape::Square)
                .with_accessory(accessory)?;
            render_tile(&mut pixels, width, row, column, style)?;
        }
        for (index, expression) in AvatarExpression::ALL.iter().copied().enumerate() {
            let style = AvatarStyle::new(kind, AvatarBackground::Light, AvatarShape::Square)
                .with_expression(expression);
            render_tile(
                &mut pixels,
                width,
                row,
                AvatarAccessory::ALL.len() + index,
                style,
            )?;
        }
    }

    let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
    ppm.extend_from_slice(&pixels);
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "hashavatar-layered.ppm".to_owned());
    fs::write(&output, ppm)?;
    println!("wrote {output}");
    Ok(())
}

fn render_tile(
    destination: &mut [u8],
    sheet_width: usize,
    row: usize,
    column: usize,
    style: AvatarStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    const AVATAR: usize = 64;
    const TILE: usize = 68;
    let image = AvatarRequest::new(64, 64, 23, b"layered-visual-corpus", style)?
        .prepare()?
        .render_rgba()?;
    let tile_x = column * TILE + 2;
    let tile_y = row * TILE + 2;
    for (y, source_row) in image.pixels().chunks_exact(AVATAR * 4).enumerate() {
        for (x, source) in source_row.chunks_exact(4).enumerate() {
            let Some(channels) = source.get(..4) else {
                continue;
            };
            let destination_offset = ((tile_y + y) * sheet_width + tile_x + x) * 3;
            let Some(pixel) = destination.get_mut(destination_offset..destination_offset + 3)
            else {
                continue;
            };
            let alpha = u32::from(channels.get(3).copied().unwrap_or_default());
            for (output, input) in pixel.iter_mut().zip(channels.iter().take(3)) {
                *output =
                    u8::try_from((u32::from(*input) * alpha + 255 * (255 - alpha) + 127) / 255)
                        .unwrap_or_default();
            }
        }
    }
    Ok(())
}
