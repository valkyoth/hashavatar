use std::io::Write;

use sanitization::wipe;

use crate::FormatError;

pub(crate) struct CountingWriter<'a, W> {
    inner: &'a mut W,
    written: usize,
}

impl<'a, W> CountingWriter<'a, W> {
    pub(crate) const fn new(inner: &'a mut W) -> Self {
        Self { inner, written: 0 }
    }

    pub(crate) const fn written(&self) -> usize {
        self.written
    }
}

impl<W: Write> Write for CountingWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(bytes)?;
        self.written = self
            .written
            .checked_add(written)
            .ok_or_else(|| std::io::Error::other("encoded output length overflow"))?;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub(crate) struct SanitizingWriter {
    bytes: Vec<u8>,
}

impl SanitizingWriter {
    pub(crate) fn try_with_capacity(capacity: usize) -> Result<Self, FormatError> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| FormatError::Allocation)?;
        Ok(Self { bytes })
    }

    pub(crate) fn into_inner(mut self) -> Vec<u8> {
        core::mem::take(&mut self.bytes)
    }
}

impl Write for SanitizingWriter {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let required = self
            .bytes
            .len()
            .checked_add(input.len())
            .ok_or_else(|| std::io::Error::other("encoded output length overflow"))?;
        if required > self.bytes.capacity() {
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(required)
                .map_err(std::io::Error::other)?;
            replacement.extend_from_slice(&self.bytes);
            wipe::vec(&mut self.bytes);
            self.bytes = replacement;
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for SanitizingWriter {
    fn drop(&mut self) {
        wipe::vec(&mut self.bytes);
    }
}
