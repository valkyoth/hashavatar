use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let avatar = AvatarBuilder::for_id("user@example.com")
        .namespace("tenant-a", "v2")
        .size(256, 256)
        .kind(AvatarKind::Robot)
        .background(AvatarBackground::Transparent);

    println!("identity={}", avatar.identity_cache_key()?);
    println!("avatar={}", avatar.avatar_asset_key()?);
    println!(
        "webp={}",
        avatar.encoded_asset_key(AvatarOutputFormat::WebP)?
    );
    println!(
        "deployment-webp={}",
        avatar.encoded_asset_key_for_build(
            AvatarOutputFormat::WebP,
            // Demonstration only. Production must hash the resolved encoder
            // build inputs described by EncoderBuildId::from_bytes.
            EncoderBuildId::from_bytes([0x42; 32]),
        )?
    );
    Ok(())
}
