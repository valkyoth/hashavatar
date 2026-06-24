fn encode_rgba_image(image: &RgbaImage, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    let mut bytes = SanitizingVec::with_capacity(image.as_raw().len());
    let result = {
        let cursor = Cursor::new(bytes.as_mut_vec());
        encode_into_writer(image, format, cursor)
    };
    match result {
        Ok(()) => Ok(bytes.into_inner()),
        Err(error) => Err(error),
    }
}

fn encode_owned_rgba_image(image: RgbaImage, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    let image = SanitizingRgbaImage::new(image);
    encode_rgba_image(image.as_image(), format)
}

struct SanitizingRgbaImage {
    image: RgbaImage,
}

impl SanitizingRgbaImage {
    fn new(image: RgbaImage) -> Self {
        Self { image }
    }

    fn as_image(&self) -> &RgbaImage {
        &self.image
    }
}

impl Drop for SanitizingRgbaImage {
    fn drop(&mut self) {
        sanitize_rgba_pixels(&mut self.image);
    }
}

fn sanitize_rgba_pixels(image: &mut RgbaImage) {
    let pixels: &mut [u8] = image.as_mut();
    sanitize_bytes(pixels);
}

struct SanitizingVec {
    bytes: Vec<u8>,
}

impl SanitizingVec {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
        }
    }

    #[cfg(feature = "jpeg")]
    fn from_vec(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }

    #[cfg(feature = "jpeg")]
    fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    fn into_inner(mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytes)
    }
}

impl Drop for SanitizingVec {
    fn drop(&mut self) {
        volatile_sanitize_vec(&mut self.bytes);
    }
}

fn encode_into_writer<W: std::io::Write>(
    image: &RgbaImage,
    format: AvatarOutputFormat,
    writer: W,
) -> ImageResult<()> {
    match format {
        AvatarOutputFormat::WebP => WebPEncoder::new_lossless(writer).write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        ),
        #[cfg(feature = "png")]
        AvatarOutputFormat::Png => {
            PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    ExtendedColorType::Rgba8,
                )
        }
        #[cfg(feature = "jpeg")]
        AvatarOutputFormat::Jpeg => {
            let rgb = SanitizingVec::from_vec(rgba_to_rgb_over_white(image));
            JpegEncoder::new_with_quality(writer, 92).write_image(
                rgb.as_slice(),
                image.width(),
                image.height(),
                ExtendedColorType::Rgb8,
            )
        }
        #[cfg(feature = "gif")]
        AvatarOutputFormat::Gif => GifEncoder::new(writer).write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        ),
    }
}

#[cfg(any(feature = "jpeg", test))]
fn rgba_to_rgb_over_white(image: &RgbaImage) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(image.as_raw().len() / 4 * 3);
    for pixel in image.pixels() {
        let [red, green, blue, alpha] = pixel.0;
        let alpha = u32::from(alpha);
        let inverse_alpha = 255 - alpha;
        rgb.push(((u32::from(red) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
        rgb.push(((u32::from(green) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
        rgb.push(((u32::from(blue) * alpha + 255 * inverse_alpha + 127) / 255) as u8);
    }
    rgb
}

