use crate::asset::delete_texture2d;
use crate::{Image, Size2D};

#[derive(Debug, Clone)]
pub(crate) enum ImgFormat {
   R(u8), //(bit depth)
   RG(u8),
   RGB(u8),
   RGBA(u8),
}

impl ImgFormat {
   pub(crate) fn channels(&self) -> u8 {
      match self {
         ImgFormat::R(_) => 1,
         ImgFormat::RG(_) => 2,
         ImgFormat::RGB(_) => 3,
         ImgFormat::RGBA(_) => 4,
      }
   }
   pub(crate) fn bit_depth(&self) -> u8 {
      *match self {
         ImgFormat::R(bd) => bd,
         ImgFormat::RG(bd) => bd,
         ImgFormat::RGB(bd) => bd,
         ImgFormat::RGBA(bd) => bd,
      }
   }
   pub(crate) fn pixel_size(&self) -> u8 {
      self.channels() * self.bit_depth()
   }

   pub(crate) fn from(channels: u8, bit_depth: u8) -> ImgFormat {
      match channels {
         1 => ImgFormat::R(bit_depth),
         2 => ImgFormat::RG(bit_depth),
         3 => ImgFormat::RGB(bit_depth),
         _ => ImgFormat::RGBA(bit_depth),
      }
   }
}

#[derive(Debug, Clone, Copy)]
pub enum ImgFilter {
   Closest,
   Linear,
}

#[derive(Debug, Clone, Copy)]
pub enum ImgWrap {
   Repeat,
   Extend,
   Clip,
}

#[derive(Clone, Debug)]
pub struct Texture2D {
   pub(crate) id: u32,
   pub(crate) size: Size2D,
   pub(crate) fmt: ImgFormat,
   pub(crate) filter: ImgFilter,
   pub(crate) wrap: ImgWrap,
}

impl Texture2D {
   pub fn size(&self) -> Size2D {
      self.size
   }

   pub fn wrap(&self) -> ImgWrap {
      self.wrap
   }
   pub fn set_wrap(&mut self, wrap: ImgWrap) {
      self.wrap = wrap
   }

   pub fn filter(&self) -> ImgFilter {
      self.filter
   }
   pub fn set_filter(&mut self, filter: ImgFilter) {
      self.filter = filter
   }
   pub fn delete(self) {
      delete_texture2d(self.id)
   }
}
