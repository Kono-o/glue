use crate::asset::FileError;
use crate::renderer::ImgFormat;
use crate::{ImgFilter, ImgWrap, Size2D, Texture2D};
use gl::types::{GLenum, GLint, GLsizei};
use image::{ColorType, GenericImageView};
use std::ffi::c_void;

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
      let (color, (w, h), rgba32f) = match image::open(path) {
         Ok(i) => (i.color(), i.dimensions(), i.into_rgba32f()),
         Err(e) => return Err(FileError::InvalidImage(path.to_string())),
      };

      let bytes = rgba32f
         .as_raw()
         .iter()
         .flat_map(|&f| f.to_ne_bytes())
         .collect::<Vec<u8>>();

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

   pub fn ship(self) -> Texture2D {
      let id = create_texture2d(&self);
      Texture2D {
         id,
         size: self.size,
         fmt: self.fmt,
         filter: self.filter,
         wrap: self.wrap,
      }
   }
}

const TEX: u32 = gl::TEXTURE_2D;

pub(crate) fn create_texture2d(img: &Image) -> u32 {
   let mut id = 0;
   unsafe {
      gl::GenTextures(1, &mut id);
      bind_texture2d_sampler_at(id, 0);

      let wrap = match_tex_wrap(&img.wrap);
      let (min_fil, mag_fil) = match_tex_filter(&img.filter);

      gl::TexParameteri(TEX, gl::TEXTURE_MIN_FILTER, min_fil);
      gl::TexParameteri(TEX, gl::TEXTURE_MAG_FILTER, mag_fil);
      gl::TexParameteri(TEX, gl::TEXTURE_WRAP_S, wrap);
      gl::TexParameteri(TEX, gl::TEXTURE_WRAP_T, wrap);

      let (base, size) = match_tex_fmt(&img.fmt);
      let (width, height) = (img.size.w as GLsizei, img.size.h as GLsizei);

      gl::TexImage2D(
         TEX,
         0,
         size as GLint,
         width,
         height,
         0,
         base,
         gl::UNSIGNED_BYTE,
         &img.bytes[0] as *const u8 as *const c_void,
      );
      gl::GenerateMipmap(TEX);
      unbind_texture2d()
   }
   id
}

pub(crate) fn bind_texture2d_sampler_at(tex_id: u32, slot: u32) {
   unsafe {
      gl::ActiveTexture(gl::TEXTURE0 + slot);
      gl::BindTexture(TEX, tex_id);
   }
}
pub(crate) fn bind_image_texture2d_at(tex_id: u32, slot: u32) {
   unsafe {
      gl::BindImageTexture(slot, tex_id, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA8);
   }
}

pub(crate) fn unbind_texture2d() {
   unsafe {
      gl::BindTexture(TEX, 0);
   }
}

pub(crate) fn delete_texture2d(id: u32) {
   unsafe {
      gl::DeleteTextures(1, &id);
   }
}

fn match_tex_fmt(tf: &ImgFormat) -> (GLenum, GLenum) {
   let (base, bd) = match tf {
      ImgFormat::R(bd) => (gl::RED, bd),
      ImgFormat::RG(bd) => (gl::RG, bd),
      ImgFormat::RGB(bd) => (gl::RGB, bd),
      ImgFormat::RGBA(bd) => (gl::RGBA, bd),
   };
   let sized = match (base, bd) {
      (gl::RED, 16) => gl::R16,
      (gl::RG, 16) => gl::RG16,
      (gl::RGB, 16) => gl::RGB16,
      (gl::RGBA, 16) => gl::RGBA16,

      (gl::RED, _) => gl::R8,
      (gl::RG, _) => gl::RG8,
      (gl::RGB, _) => gl::RGB8,
      (gl::RGBA, _) => gl::RGBA8,

      _ => gl::RGB8,
   };
   (base, sized)
}
fn match_tex_filter(tf: &ImgFilter) -> (GLint, GLint) {
   let (min, max) = match tf {
      ImgFilter::Closest => (gl::NEAREST_MIPMAP_NEAREST, gl::NEAREST),
      ImgFilter::Linear => (gl::LINEAR_MIPMAP_LINEAR, gl::LINEAR),
   };
   (min as GLint, max as GLint)
}
fn match_tex_wrap(tf: &ImgWrap) -> GLint {
   let wrap = match tf {
      ImgWrap::Repeat => gl::REPEAT,
      ImgWrap::Extend => gl::CLAMP_TO_EDGE,
      ImgWrap::Clip => gl::CLAMP_TO_BORDER,
   };
   wrap as GLint
}
