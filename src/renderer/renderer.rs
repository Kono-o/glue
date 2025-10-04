use crate::renderer::glraw::{GL, GLError};
use crate::{CamProj, Camera, RGBA, Size2D};

#[derive(Copy, Clone)]
pub enum PolyMode {
   Points,
   WireFrame,
   Filled,
}
#[derive(Copy, Clone)]
pub enum Cull {
   Clock,
   AntiClock,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum ShaderSrcType {
   Vert,
   Frag,
}

#[derive(Debug)]
pub enum GPUError {
   GLError(GLError),
}

#[derive(Debug)]
pub enum GLueErrorKind {
   //UNKNOWN
   SomethingWentWrong,
   //SHADERS
   ShaderCompileFailed,
   ProgramLinkFailed,
}

#[derive(Debug)]
pub struct GLueError {
   msg: String,
   kind: GLueErrorKind,
}

impl GLueError {
   pub fn wtf(msg: &str) -> Self {
      GLueError {
         msg: msg.to_string(),
         kind: GLueErrorKind::SomethingWentWrong,
      }
   }
   pub fn from(kind: GLueErrorKind, msg: &str) -> Self {
      GLueError {
         msg: msg.to_string(),
         kind,
      }
   }
}

pub struct GPU {
   pub(crate) gl: GL,
   pub(crate) cam: Camera,
   pub(crate) poly_mode: PolyMode,
   pub(crate) cull_face: Cull,
   pub(crate) bg_color: RGBA,
   pub(crate) msaa: bool,
   pub(crate) msaa_samples: u32,
   pub(crate) culling: bool,
}

impl GPU {
   pub fn new() -> Result<GPU, GPUError> {
      let cam = Camera::new(Size2D::from(10, 10), CamProj::Ortho);
      let bg_color = RGBA::grey(0.5);
      let gl = match GL::load(10, 10) {
         Err(e) => return Err(GPUError::GLError(e)),
         Ok(gl) => gl,
      };

      let mut renderer = GPU {
         gl,
         cam,
         bg_color,
         msaa: true,
         culling: true,
         msaa_samples: 4,
         cull_face: Cull::AntiClock,
         poly_mode: PolyMode::Filled,
      };
      renderer.set_msaa(true);
      renderer.set_culling(true);
      renderer.set_wire_width(2.0);
      renderer.set_bg_color(bg_color);
      renderer.gl.enable_alpha(true);
      Ok(renderer)
   }
   pub fn version(&self) -> &str {
      &self.gl.glsl_ver
   }
   pub fn name(&self) -> &str {
      &self.gl.device
   }

   pub(crate) fn set_size(&mut self, size: Size2D) {
      self.gl.resize(size);
   }
   fn clear(&self) {
      self.gl.clear()
   }

   pub fn set_msaa_samples(&mut self, samples: u32) {
      self.msaa_samples = samples
   }
   pub fn set_bg_color(&mut self, color: RGBA) {
      self.bg_color = color;
      self.gl.set_clear(color);
   }
   pub fn set_poly_mode(&mut self, mode: PolyMode) {
      self.poly_mode = mode;
      self.gl.poly_mode(mode);
   }
   pub fn toggle_wireframe(&mut self) {
      let new_poly_mode = match self.poly_mode {
         PolyMode::WireFrame => PolyMode::Filled,
         _ => PolyMode::WireFrame,
      };
      self.set_poly_mode(new_poly_mode);
   }
   pub fn set_msaa(&mut self, enable: bool) {
      self.msaa = enable;
      self.gl.enable_msaa(enable);
   }
   pub fn set_point_size(&self, size: f32) {
      self.gl.set_point_size(size)
   }
   pub fn toggle_msaa(&mut self) {
      self.msaa = !self.msaa;
      self.gl.enable_msaa(self.msaa)
   }
   pub fn set_culling(&mut self, enable: bool) {
      if self.culling != enable {
         self.toggle_culling()
      }
      self.gl.enable_cull(enable);
   }
   pub fn toggle_culling(&mut self) {
      self.culling = !self.culling;
      self.gl.enable_cull(self.culling);
   }
   pub fn set_cull_face(&mut self, cull_face: Cull) {
      self.cull_face = cull_face;
      self.gl.set_cull_face(self.cull_face)
   }
   pub fn flip_cull_face(&mut self) {
      self.cull_face = match self.cull_face {
         Cull::Clock => Cull::AntiClock,
         Cull::AntiClock => Cull::Clock,
      };
      self.gl.set_cull_face(self.cull_face);
   }
   pub fn set_wire_width(&mut self, width: f32) {
      self.gl.set_wire_width(width);
   }
}
