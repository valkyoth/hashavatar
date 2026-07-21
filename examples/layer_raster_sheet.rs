//! Generates fixed-path PPM sheets for complete layer-placement review.

use std::{fs, path::Path};

use hashavatar::{
    AvatarAccessory, AvatarBackground, AvatarExpression, AvatarKind, AvatarRequest, AvatarShape,
    AvatarStyle,
};

const AVATAR: usize = 64;
const AVATAR_U32: u32 = 64;
const TILE: usize = 68;
const OUTPUT_DIRECTORY: &str = "target/visual-review";
const ACCESSORY_OUTPUT: &str = "target/visual-review/accessories.ppm";
const EXPRESSION_OUTPUT: &str = "target/visual-review/expressions.ppm";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(Path::new(OUTPUT_DIRECTORY))?;
    let kinds: Vec<_> = AvatarKind::ALL
        .iter()
        .copied()
        .filter(|kind| kind.capabilities().has_face_anchors())
        .collect();

    let accessories = render_sheet(&kinds, 1 + AvatarAccessory::ALL.len(), |kind, column| {
        let style = AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Square);
        if column == 0 {
            Ok(style)
        } else {
            style.with_accessory(
                AvatarAccessory::ALL
                    .get(column - 1)
                    .copied()
                    .ok_or(hashavatar::AvatarError::InvalidScene)?,
            )
        }
    })?;
    let expressions = render_sheet(&kinds, AvatarExpression::ALL.len(), |kind, column| {
        let expression = AvatarExpression::ALL
            .get(column)
            .copied()
            .ok_or(hashavatar::AvatarError::InvalidScene)?;
        Ok(
            AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Square)
                .with_expression(expression),
        )
    })?;
    fs::write(Path::new(ACCESSORY_OUTPUT), accessories)?;
    fs::write(Path::new(EXPRESSION_OUTPUT), expressions)?;
    println!("wrote {ACCESSORY_OUTPUT} and {EXPRESSION_OUTPUT}");
    Ok(())
}

fn render_sheet<F>(
    kinds: &[AvatarKind],
    columns: usize,
    style_for: F,
) -> Result<Vec<u8>, Box<dyn std::error::Error>>
where
    F: Fn(AvatarKind, usize) -> Result<AvatarStyle, hashavatar::AvatarError>,
{
    let width = columns * TILE;
    let height = kinds.len() * TILE;
    let mut pixels = vec![255_u8; width * height * 3];
    for (row, kind) in kinds.iter().copied().enumerate() {
        for column in 0..columns {
            let style = style_for(kind, column).map_err(|error| {
                std::io::Error::other(format!(
                    "{} column {column} failed to resolve style: {error}",
                    kind.as_str()
                ))
            })?;
            let image = AvatarRequest::new(AVATAR_U32, AVATAR_U32, 7, b"layer-review-sheet", style)
                .map_err(|error| {
                    std::io::Error::other(format!(
                        "{} column {column} failed to build: {error}",
                        kind.as_str()
                    ))
                })?
                .prepare()
                .map_err(|error| {
                    std::io::Error::other(format!(
                        "{} column {column} failed to prepare: {error}",
                        kind.as_str()
                    ))
                })?
                .render_rgba()
                .map_err(|error| {
                    std::io::Error::other(format!(
                        "{} column {column} failed to render: {error}",
                        kind.as_str()
                    ))
                })?;
            blit(
                &mut pixels,
                width,
                column * TILE + 2,
                row * TILE + 2,
                image.pixels(),
            );
        }
    }
    let mut ppm = format!("P6\n{width} {height}\n255\n").into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok(ppm)
}

fn blit(destination: &mut [u8], width: usize, x: usize, y: usize, source: &[u8]) {
    for (row, source_row) in source.chunks_exact(AVATAR * 4).enumerate() {
        for (column, pixel) in source_row.chunks_exact(4).enumerate() {
            let [red, green, blue, alpha] = pixel else {
                continue;
            };
            let alpha = u32::from(*alpha);
            let offset = ((y + row) * width + x + column) * 3;
            if let Some(output) = destination.get_mut(offset..offset + 3) {
                for (output_channel, source_channel) in output.iter_mut().zip([*red, *green, *blue])
                {
                    *output_channel = u8::try_from(
                        (u32::from(source_channel) * alpha + 255 * (255 - alpha) + 127) / 255,
                    )
                    .unwrap_or_default();
                }
            }
        }
    }
}
