//! Encodes one prepared avatar through the facade's default WebP provider.

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    formats::{AvatarOutputFormat, encode},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let style = AvatarStyle::new(
        AvatarKind::Robot,
        AvatarBackground::Ocean,
        AvatarShape::Circle,
    );
    let prepared = AvatarRequest::new(256, 256, 0, b"encoded-example", style)?.prepare()?;
    let encoded = encode(&prepared, AvatarOutputFormat::WebP)?;
    println!(
        "{} bytes, {}, key {}",
        encoded.bytes().len(),
        encoded.metadata().media_type(),
        encoded.metadata().semantic_key(),
    );
    Ok(())
}
