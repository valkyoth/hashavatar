use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let svg = AvatarBuilder::for_id("user@example.com")
        .size(256, 256)
        .namespace("tenant-a", "v2")
        .kind(AvatarKind::Robot)
        .background(AvatarBackground::Transparent)
        .accessory(AvatarAccessory::Glasses)
        .color(AvatarColor::Gold)
        .expression(AvatarExpression::Happy)
        .shape(AvatarShape::Circle)
        .render_svg()?;

    println!("{svg}");
    Ok(())
}
