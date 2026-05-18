#![no_main]

use hashavatar::fuzz_draw_polygon_rgba;
use libfuzzer_sys::fuzz_target;

const MAX_FUZZ_DIMENSION: u32 = 512;
const MAX_FUZZ_POINTS: usize = 64;

fn read_u32(data: &[u8], offset: &mut usize) -> Option<u32> {
    let bytes = data.get(*offset..*offset + 4)?;
    *offset += 4;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(data: &[u8], offset: &mut usize) -> Option<i32> {
    let bytes = data.get(*offset..*offset + 4)?;
    *offset += 4;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    let mut offset = 0;
    let width = read_u32(data, &mut offset).unwrap_or(0) % (MAX_FUZZ_DIMENSION + 1);
    let height = read_u32(data, &mut offset).unwrap_or(0) % (MAX_FUZZ_DIMENSION + 1);
    let point_count = usize::from(data[offset]) % (MAX_FUZZ_POINTS + 1);
    offset += 1;
    let color_seed = data[offset];
    offset += 1;

    let mut points = Vec::with_capacity(point_count);
    for _ in 0..point_count {
        let Some(x) = read_i32(data, &mut offset) else {
            break;
        };
        let Some(y) = read_i32(data, &mut offset) else {
            break;
        };
        points.push((x, y));
    }

    fuzz_draw_polygon_rgba(
        width,
        height,
        &points,
        [
            color_seed,
            color_seed.wrapping_mul(37),
            color_seed.wrapping_add(113),
            data.len() as u8,
        ],
    );
});
