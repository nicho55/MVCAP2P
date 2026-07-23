use image::imageops::FilterType;
use image::DynamicImage;
use shared::limits;

pub enum AssetKind {
    Token,
    Map,
}

pub struct TranscodeResult {
    pub data: Vec<u8>,
    pub thumb: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub enum TranscodeError {
    InvalidImage,
    TooLarge { bytes: usize, max: usize },
}

impl std::fmt::Display for TranscodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscodeError::InvalidImage => write!(f, "imagem inválida"),
            TranscodeError::TooLarge { bytes, max } => {
                write!(f, "arquivo muito grande: {bytes} bytes (máximo: {max})")
            }
        }
    }
}

pub fn transcode(raw: &[u8], kind: AssetKind) -> Result<TranscodeResult, TranscodeError> {
    let (max_px, max_bytes) = match kind {
        AssetKind::Token => (limits::MAX_TOKEN_PX, limits::MAX_TOKEN_BYTES),
        AssetKind::Map => (limits::MAX_MAP_PX, limits::MAX_MAP_BYTES),
    };

    let img = image::load_from_memory(raw).map_err(|_| TranscodeError::InvalidImage)?;

    let img = fit_within(img, max_px);

    let data = encode_webp(&img);
    if data.len() > max_bytes {
        return Err(TranscodeError::TooLarge {
            bytes: data.len(),
            max: max_bytes,
        });
    }

    let thumb_img = img.resize(limits::THUMB_PX, limits::THUMB_PX, FilterType::Triangle);
    let thumb = encode_webp(&thumb_img);

    Ok(TranscodeResult {
        width: img.width(),
        height: img.height(),
        data,
        thumb,
    })
}

fn fit_within(img: DynamicImage, max_px: u32) -> DynamicImage {
    if img.width() <= max_px && img.height() <= max_px {
        return img;
    }
    img.resize(max_px, max_px, FilterType::Lanczos3)
}

fn encode_webp(img: &DynamicImage) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::WebP)
        .expect("WebP encode");
    buf.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_png(w: u32, h: u32) -> Vec<u8> {
        let img = DynamicImage::new_rgba8(w, h);
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        buf.into_inner()
    }

    #[test]
    fn token_transcode_fits_within_limits() {
        let png = make_png(512, 512);
        let result = transcode(&png, AssetKind::Token).unwrap();
        assert!(result.width <= limits::MAX_TOKEN_PX);
        assert!(result.height <= limits::MAX_TOKEN_PX);
        assert!(result.data.len() <= limits::MAX_TOKEN_BYTES);
    }

    #[test]
    fn thumbnail_is_generated() {
        let png = make_png(256, 256);
        let result = transcode(&png, AssetKind::Token).unwrap();
        assert!(!result.thumb.is_empty());
        let thumb = image::load_from_memory(&result.thumb).unwrap();
        assert!(thumb.width() <= limits::THUMB_PX);
        assert!(thumb.height() <= limits::THUMB_PX);
    }

    #[test]
    fn map_transcode_resizes_large_image() {
        let png = make_png(4096, 4096);
        let result = transcode(&png, AssetKind::Map).unwrap();
        assert!(result.width <= limits::MAX_MAP_PX);
        assert!(result.height <= limits::MAX_MAP_PX);
    }

    #[test]
    fn invalid_bytes_rejected() {
        let garbage = vec![0u8; 100];
        assert!(matches!(
            transcode(&garbage, AssetKind::Token),
            Err(TranscodeError::InvalidImage)
        ));
    }
}
