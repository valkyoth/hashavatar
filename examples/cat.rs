//! Prepares one Cat scene and executes both canonical alpha.1 outputs.

use hashavatar::CatRequest;

fn main() -> Result<(), hashavatar::CatError> {
    let prepared = CatRequest::new(256, 256, 0, b"example-user")?.prepare()?;
    let image = prepared.render_rgba()?;
    let svg = prepared.render_svg()?;

    assert_eq!(image.pixels().len(), prepared.scene_report().rgba_bytes());
    print!("{svg}");
    Ok(())
}
