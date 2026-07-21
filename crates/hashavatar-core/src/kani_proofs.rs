use crate::{
    CatRequest, MAX_DIMENSION, MIN_DIMENSION, RGBA8_BYTES_PER_PIXEL,
    fixed::Fixed,
    scene::{Color, Command, Scene},
};

#[kani::proof]
fn request_dimension_admission_is_exact() {
    let width = u32::from(kani::any::<u16>());
    let height = u32::from(kani::any::<u16>());
    let admitted = CatRequest::new(width, height, kani::any::<u64>(), b"").is_ok();
    let expected = (MIN_DIMENSION..=MAX_DIMENSION).contains(&width)
        && (MIN_DIMENSION..=MAX_DIMENSION).contains(&height);
    assert_eq!(admitted, expected);
}

#[kani::proof]
fn unit_fixed_conversion_stays_in_closed_interval() {
    let value = kani::any::<u16>();
    if let Ok(fixed) = Fixed::from_unit_u16(value) {
        assert!(fixed >= Fixed::ZERO);
        assert!(fixed <= Fixed::from_integer(1).unwrap_or(Fixed::ZERO));
    }
}

#[kani::proof]
fn fixed_lerp_stays_between_small_positive_bounds() {
    let minimum_integer = i32::from(kani::any::<u8>());
    let span = i32::from(kani::any::<u8>());
    let maximum_integer = minimum_integer.saturating_add(span);
    if let (Ok(minimum), Ok(maximum)) = (
        Fixed::from_integer(minimum_integer),
        Fixed::from_integer(maximum_integer),
    ) {
        if let Ok(value) = Fixed::lerp(minimum, maximum, kani::any::<u16>()) {
            assert!(value >= minimum);
            assert!(value <= maximum);
        }
    }
}

#[kani::proof]
fn pixel_center_is_inside_its_pixel() {
    let pixel = u32::from(kani::any::<u16>()) % MAX_DIMENSION;
    if let Ok(center) = Fixed::pixel_center(pixel) {
        assert_eq!(
            center.floor(),
            i32::try_from(pixel).map_err(|_| crate::CatError::NumericRange)
        );
        assert_eq!(
            center.ceil(),
            i32::try_from(pixel.saturating_add(1)).map_err(|_| crate::CatError::NumericRange)
        );
    }
}

#[kani::proof]
fn validated_scene_report_has_exact_rgba_size() {
    let width = MIN_DIMENSION + (u32::from(kani::any::<u8>()) % 32);
    let height = MIN_DIMENSION + (u32::from(kani::any::<u8>()) % 32);
    if let Ok(mut scene) = Scene::new(width, height) {
        if scene.push(Command::Fill(Color::rgb(1, 2, 3))).is_ok() {
            if let Ok(report) = scene.validate() {
                let expected = usize::try_from(width)
                    .unwrap_or(0)
                    .saturating_mul(usize::try_from(height).unwrap_or(0))
                    .saturating_mul(RGBA8_BYTES_PER_PIXEL);
                assert_eq!(report.rgba_bytes(), expected);
            }
        }
    }
}
