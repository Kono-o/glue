use crate::asset::FileError;
use crate::renderer::TexFormat;
use crate::{Size2D, TexFilter, TexWrap};
use image::{ColorType, GenericImageView};

#[derive(Debug)]
pub struct TextureFile {
    pub(crate) bytes: Vec<u8>,
    pub(crate) size: Size2D,
    pub(crate) fmt: TexFormat,

    pub(crate) filter: TexFilter,
    pub(crate) wrap: TexWrap,
}

impl TextureFile {
    fn from_path(path: &str) -> Result<TextureFile, FileError> {
        let (color, (w, h), bytes) = match image::open(path) {
            Ok(i) => (i.color(), i.dimensions(), i.into_bytes()),
            Err(e) => return Err(FileError::InvalidImage(path.to_string())),
        };
        let fmt = match color {
            ColorType::L8 => TexFormat::R(8),
            ColorType::La8 => TexFormat::RG(8),
            ColorType::Rgb8 => TexFormat::RGB(8),
            ColorType::Rgba8 => TexFormat::RGBA(8),

            ColorType::L16 => TexFormat::R(16),
            ColorType::La16 => TexFormat::RG(16),
            ColorType::Rgb16 => TexFormat::RGB(16),
            ColorType::Rgba16 => TexFormat::RGBA(16),

            ColorType::Rgb32F => TexFormat::RGB(32),
            ColorType::Rgba32F => TexFormat::RGBA(32),
            _ => return Err(FileError::InvalidImage(path.to_string())),
        };
        let filter = TexFilter::Closest;
        let wrap = TexWrap::Clip;
        Ok(TextureFile {
            bytes,
            size: Size2D::from(w, h),
            fmt,
            filter,
            wrap,
        })
    }

    pub fn set_wrap(&mut self, wrap: TexWrap) {
        self.wrap = wrap
    }
    pub fn set_filter(&mut self, filter: TexFilter) {
        self.filter = filter
    }

    pub fn pixel_count(&self) -> usize {
        let (channels, bits) = match self.fmt {
            TexFormat::R(b) => (1, b),
            TexFormat::RG(b) => (2, b),
            TexFormat::RGB(b) => (3, b),
            TexFormat::RGBA(b) => (4, b),
        };

        let bytes_per_pixel = (channels as usize) * (bits as usize / 8);
        if bytes_per_pixel == 0 {
            return 0;
        }

        self.bytes.len() / bytes_per_pixel
    }
}
