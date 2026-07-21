#![no_main]

use core::fmt::Write;

use hashavatar::{CatRequest, SvgOptions};
use libfuzzer_sys::fuzz_target;

struct BoundedWriter {
    remaining: usize,
}

impl Write for BoundedWriter {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        if value.len() > self.remaining {
            return Err(core::fmt::Error);
        }
        self.remaining -= value.len();
        Ok(())
    }
}

fuzz_target!(|data: &[u8]| {
    let limit = data
        .get(..2)
        .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
        .map_or(0, |bytes| usize::from(u16::from_le_bytes(bytes)));
    let text = String::from_utf8_lossy(data.get(2..).unwrap_or_default());
    let Ok(request) = CatRequest::new(64, 64, 0, b"svg-fuzz") else {
        return;
    };
    let Ok(prepared) = request.prepare() else {
        return;
    };
    if let Ok(options) = SvgOptions::document("fuzz", &text, &text) {
        if let Ok(svg) = prepared.render_svg_with(options) {
            assert!(roxmltree::Document::parse(&svg).is_ok());
        }
        let _ = prepared.write_svg(&mut BoundedWriter { remaining: limit }, options);
    }
});
