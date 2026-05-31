use hashavatar::prelude::*;

fn main() -> Result<(), AvatarError> {
    let key = AvatarBuilder::for_id("user@example.com")
        .namespace("tenant-a", "v2")
        .cache_key()?;

    println!("{key}");
    Ok(())
}
