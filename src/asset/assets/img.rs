use crate::asset::FileError;
use crate::renderer::ImgFormat;
use crate::{ImgFilter, ImgWrap, Size2D};
use image::{ColorType, GenericImageView};

#[derive(Debug)]
pub struct Image {
   pub(crate) bytes: Vec<u8>,
   pub(crate) size: Size2D,
   pub(crate) fmt: ImgFormat,

   pub(crate) filter: ImgFilter,
   pub(crate) wrap: ImgWrap,
}

impl Image {
   pub fn from_path(path: &str) -> Result<Image, FileError> {
      let (color, (w, h), bytes) = match image::open(path) {
         Ok(i) => (i.color(), i.dimensions(), i.into_bytes()),
         Err(e) => return Err(FileError::InvalidImage(path.to_string())),
      };
      let fmt = match color {
         ColorType::L8 => ImgFormat::R(8),
         ColorType::La8 => ImgFormat::RG(8),
         ColorType::Rgb8 => ImgFormat::RGB(8),
         ColorType::Rgba8 => ImgFormat::RGBA(8),

         ColorType::L16 => ImgFormat::R(16),
         ColorType::La16 => ImgFormat::RG(16),
         ColorType::Rgb16 => ImgFormat::RGB(16),
         ColorType::Rgba16 => ImgFormat::RGBA(16),

         ColorType::Rgb32F => ImgFormat::RGB(32),
         ColorType::Rgba32F => ImgFormat::RGBA(32),
         _ => return Err(FileError::InvalidImage(path.to_string())),
      };
      let filter = ImgFilter::Closest;
      let wrap = ImgWrap::Clip;
      Ok(Image {
         bytes,
         size: Size2D::from(w, h),
         fmt,
         filter,
         wrap,
      })
   }

   pub fn set_wrap(&mut self, wrap: ImgWrap) {
      self.wrap = wrap
   }
   pub fn set_filter(&mut self, filter: ImgFilter) {
      self.filter = filter
   }

   pub fn pixel_count(&self) -> usize {
      let (channels, bits) = match self.fmt {
         ImgFormat::R(b) => (1, b),
         ImgFormat::RG(b) => (2, b),
         ImgFormat::RGB(b) => (3, b),
         ImgFormat::RGBA(b) => (4, b),
      };

      let bytes_per_pixel = (channels as usize) * (bits as usize / 8);
      if bytes_per_pixel == 0 {
         return 0;
      }

      self.bytes.len() / bytes_per_pixel
   }
}
