use super::*;

#[kani::proof]
fn avatar_spec_new_preserves_supported_dimension_contract() {
    let width = u32::from(kani::any::<u16>());
    let height = u32::from(kani::any::<u16>());
    let seed = kani::any::<u64>();

    match AvatarSpec::new(width, height, seed) {
        Ok(spec) => {
            assert!(spec.width() >= MIN_AVATAR_DIMENSION);
            assert!(spec.height() >= MIN_AVATAR_DIMENSION);
            assert!(spec.width() <= MAX_AVATAR_DIMENSION);
            assert!(spec.height() <= MAX_AVATAR_DIMENSION);
            assert!(spec.pixel_count() <= MAX_AVATAR_PIXELS);
            assert!(spec.rgba_buffer_len() <= MAX_AVATAR_RGBA_BYTES);
            assert_eq!(spec.seed(), seed);
        }
        Err(error) => {
            assert_eq!(error.width(), width);
            assert_eq!(error.height(), height);
            assert!(
                width < MIN_AVATAR_DIMENSION
                    || height < MIN_AVATAR_DIMENSION
                    || width > MAX_AVATAR_DIMENSION
                    || height > MAX_AVATAR_DIMENSION
            );
        }
    }
}

#[kani::proof]
fn render_resource_budget_uses_saturating_memory_math() {
    let spec = AvatarSpec::default();
    let concurrent_renders = usize::from(kani::any::<u8>());
    let budget = spec.render_resource_budget(concurrent_renders);

    assert_eq!(budget.spec(), spec);
    assert_eq!(budget.concurrent_renders(), concurrent_renders);
    assert_eq!(budget.raw_rgba_bytes_per_render(), spec.rgba_buffer_len());
    assert_eq!(
        budget.raw_rgba_bytes_for_concurrent_renders(),
        spec.rgba_buffer_len().saturating_mul(concurrent_renders)
    );
    assert_eq!(
        AvatarRenderResourceBudget::max_supported_raw_rgba_bytes_for_concurrent_renders(
            concurrent_renders
        ),
        MAX_AVATAR_RGBA_BYTES.saturating_mul(concurrent_renders)
    );
}

#[kani::proof]
fn resource_budget_memory_division_never_divides_by_zero() {
    let width = MIN_AVATAR_DIMENSION
        + (u32::from(kani::any::<u16>()) % (MAX_AVATAR_DIMENSION - MIN_AVATAR_DIMENSION + 1));
    let height = MIN_AVATAR_DIMENSION
        + (u32::from(kani::any::<u16>()) % (MAX_AVATAR_DIMENSION - MIN_AVATAR_DIMENSION + 1));
    let memory_budget_bytes = usize::from(kani::any::<u16>());

    if let Ok(spec) = AvatarSpec::new(width, height, kani::any::<u64>()) {
        let value = AvatarRenderResourceBudget::max_concurrent_renders_for_memory_budget(
            spec,
            memory_budget_bytes,
        );

        assert_eq!(value, memory_budget_bytes / spec.rgba_buffer_len());
    }
}

#[kani::proof]
fn rect_of_size_and_edges_remain_non_zero_and_saturating() {
    let left = i32::from(kani::any::<i16>());
    let top = i32::from(kani::any::<i16>());
    let width = u32::from(kani::any::<u16>());
    let height = u32::from(kani::any::<u16>());

    let rect = Rect::at(left, top).of_size(width, height);

    assert!(rect.width() >= 1);
    assert!(rect.height() >= 1);
    assert!(rect.right() >= rect.left());
    assert!(rect.bottom() >= rect.top());
}

#[kani::proof]
fn rect_intersection_when_present_is_inside_both_inputs() {
    let a = Rect::at(i32::from(kani::any::<i16>()), i32::from(kani::any::<i16>()))
        .of_size(u32::from(kani::any::<u8>()), u32::from(kani::any::<u8>()));
    let b = Rect::at(i32::from(kani::any::<i16>()), i32::from(kani::any::<i16>()))
        .of_size(u32::from(kani::any::<u8>()), u32::from(kani::any::<u8>()));

    if let Some(intersection) = a.intersect(b) {
        assert!(intersection.width() >= 1);
        assert!(intersection.height() >= 1);
        assert!(intersection.left() >= a.left());
        assert!(intersection.top() >= a.top());
        assert!(intersection.right() <= a.right());
        assert!(intersection.bottom() <= a.bottom());
        assert!(intersection.left() >= b.left());
        assert!(intersection.top() >= b.top());
        assert!(intersection.right() <= b.right());
        assert!(intersection.bottom() <= b.bottom());
    }
}
