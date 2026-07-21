#![no_main]

use std::io::Write;

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarRequest, AvatarShape, AvatarStyle,
    formats::{AvatarOutputFormat, encode_to_writer},
};
use libfuzzer_sys::fuzz_target;

struct QuotaWriter {
    remaining: usize,
}

impl Write for QuotaWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::Error::other("fuzz writer quota exhausted"));
        }
        let written = bytes.len().min(self.remaining);
        self.remaining = self.remaining.saturating_sub(written);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fuzz_target!(|data: &[u8]| {
    let kind = AvatarKind::from_byte(data.first().copied().unwrap_or_default());
    let background = AvatarBackground::from_byte(data.get(1).copied().unwrap_or_default());
    let shape = AvatarShape::from_byte(data.get(2).copied().unwrap_or_default());
    let format = AvatarOutputFormat::ALL
        .iter()
        .copied()
        .nth(usize::from(data.get(3).copied().unwrap_or_default()) % AvatarOutputFormat::ALL.len())
        .unwrap_or(AvatarOutputFormat::WebP);
    let identity = data.get(4..data.len().min(132)).unwrap_or_default();
    let style = AvatarStyle::new(kind, background, shape);
    let Ok(prepared) = AvatarRequest::new(64, 64, 0, identity, style)
        .and_then(AvatarRequest::prepare)
    else {
        return;
    };
    let quota = usize::from(data.get(132).copied().unwrap_or(u8::MAX)) * 16;
    let _ = encode_to_writer(&prepared, format, &mut QuotaWriter { remaining: quota });
});
