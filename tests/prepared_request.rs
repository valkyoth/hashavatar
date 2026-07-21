//! Alpha.5 prepared-request, public-key, budget, and reusable-buffer contracts.

use hashavatar::{
    AvatarBackground, AvatarIdentity, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    ReusableRgbaBuffer,
};

fn style() -> AvatarStyle {
    AvatarStyle::new(
        AvatarKind::Robot,
        AvatarBackground::Ocean,
        AvatarShape::Circle,
    )
}

#[test]
fn owned_identity_builder_binds_public_keys_and_redacts_debug()
-> Result<(), hashavatar::AvatarError> {
    let identity_a = AvatarIdentity::new(b"alpha5-user")?;
    let identity_b = AvatarIdentity::new(b"alpha5-user")?;
    assert_eq!(identity_a.cache_key()?, identity_b.cache_key()?);
    assert!(!format!("{identity_a:?}").contains("alpha5-user"));

    let prepared_a = AvatarRequest::builder(identity_a)
        .size(128, 96)
        .style_variant(17)
        .style(style())
        .prepare()?;
    let prepared_b = AvatarRequest::builder(identity_b)
        .size(128, 96)
        .style_variant(17)
        .style(style())
        .prepare()?;
    assert_eq!(prepared_a.asset_key(), prepared_b.asset_key());
    assert_eq!(
        prepared_a.identity_cache_key(),
        prepared_b.identity_cache_key()
    );
    assert_eq!(
        prepared_a.resource_budget().canonical_rgba_bytes(),
        128 * 96 * 4
    );
    Ok(())
}

#[test]
fn reusable_buffer_retains_capacity_and_matches_owned_output() -> Result<(), hashavatar::AvatarError>
{
    let small = AvatarRequest::new(64, 64, 3, b"scratch-user", style())?.prepare()?;
    let large = AvatarRequest::new(256, 192, 3, b"scratch-user", style())?.prepare()?;
    let expected = small.render_rgba()?.pixel_digest()?;
    let mut scratch = ReusableRgbaBuffer::new();

    small.render_reusing(&mut scratch)?;
    assert_eq!(scratch.pixel_digest()?, expected);
    large.render_reusing(&mut scratch)?;
    let large_capacity = scratch.capacity();
    small.render_reusing(&mut scratch)?;
    assert_eq!(scratch.pixel_digest()?, expected);
    assert_eq!(scratch.capacity(), large_capacity);

    scratch.clear();
    assert_eq!(scratch.dimensions(), (0, 0));
    assert!(scratch.pixels().is_empty());
    Ok(())
}

#[test]
fn asset_key_changes_with_pixel_affecting_request_fields() -> Result<(), hashavatar::AvatarError> {
    let first = AvatarRequest::new(64, 64, 1, b"asset-user", style())?.prepare()?;
    let second = AvatarRequest::new(64, 64, 2, b"asset-user", style())?.prepare()?;
    let third = AvatarRequest::new(65, 64, 1, b"asset-user", style())?.prepare()?;
    assert_ne!(first.asset_key(), second.asset_key());
    assert_ne!(first.asset_key(), third.asset_key());
    Ok(())
}
