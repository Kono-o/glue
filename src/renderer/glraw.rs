use crate::renderer::ImgFormat;
use crate::{Cull, PolyMode, Size2D};
use crate::{Image, ImgFilter, ImgWrap, RGBA};
use cgmath::Matrix;

use gl::types::*;
use khronos_egl as egl;
use std::ffi::c_void;
use std::ptr;

pub(crate) const GL_SPV_EXTENSION: &str = "GL_ARB_gl_spirv";
pub(crate) const SPIRV_EXTENSIONS: &str = "GL_ARB_spirv_extensions";

#[derive(Debug)]
pub enum GLError {
   NoDisplay,
   InitFailed(String),
   NoActiveContext,
   CouldParseVersion(String),
   SPIRVNotFound,
   ChooseConfigFailed(String),
   NoSuitableConfig,
   BindApiFailed(String),
   CreateSurfaceFailed(String),
   CreateContextFailed(String),
   MakeCurrentFailed(String),
   RenderTime(String),
}

pub struct GL {
   pub(crate) display: egl::Display,
   pub(crate) context: egl::Context,
   pub(crate) surface: egl::Surface,
   pub(crate) glsl_ver: String,
   pub(crate) device: String,
}

impl GL {
   pub(crate) fn load(width: i32, height: i32) -> Result<GL, GLError> {
      let egl = egl::Instance::new(egl::Static);

      // Get default display
      let display = unsafe {
         match egl.get_display(egl::DEFAULT_DISPLAY) {
            None => return Err(GLError::NoDisplay),
            Some(d) => d,
         }
      };

      let _version = match egl.initialize(display) {
         Err(e) => return Err(GLError::InitFailed(e.to_string())),
         Ok((v1, v2)) => (v1, v2),
      };

      // Choose config
      let attribs = [
         egl::SURFACE_TYPE,
         egl::PBUFFER_BIT,
         egl::RENDERABLE_TYPE,
         egl::OPENGL_BIT,
         egl::RED_SIZE,
         8,
         egl::GREEN_SIZE,
         8,
         egl::BLUE_SIZE,
         8,
         egl::ALPHA_SIZE,
         8,
         egl::DEPTH_SIZE,
         24,
         egl::NONE,
      ];

      let mut configs = Vec::with_capacity(1);
      match egl.choose_config(display, &attribs, &mut configs) {
         Err(e) => return Err(GLError::ChooseConfigFailed(format!("{:?}", e))),
         Ok(_) => {}
      }

      if configs.is_empty() {
         return Err(GLError::NoSuitableConfig);
      }
      let config = configs[0];

      // Bind OpenGL API
      match egl.bind_api(egl::OPENGL_API) {
         Err(e) => return Err(GLError::BindApiFailed(format!("{:?}", e))),
         Ok(_) => {}
      }

      // Create pbuffer surface
      let pbuffer_attribs = [egl::WIDTH, width, egl::HEIGHT, height, egl::NONE];

      let surface = match egl.create_pbuffer_surface(display, config, &pbuffer_attribs) {
         Err(e) => return Err(GLError::CreateSurfaceFailed(format!("{:?}", e))),
         Ok(s) => s,
      };

      // Create context
      let context_attribs = [
         egl::CONTEXT_MAJOR_VERSION,
         3,
         egl::CONTEXT_MINOR_VERSION,
         3,
         egl::CONTEXT_OPENGL_PROFILE_MASK,
         egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
         egl::NONE,
      ];

      let context = match egl.create_context(display, config, None, &context_attribs) {
         Err(e) => return Err(GLError::CreateContextFailed(format!("{:?}", e))),
         Ok(c) => c,
      };

      // Make context current
      match egl.make_current(display, Some(surface), Some(surface), Some(context)) {
         Err(e) => return Err(GLError::MakeCurrentFailed(format!("{:?}", e))),
         Ok(_) => {}
      }

      // Load GL functions
      gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

      // Fetch GL info
      let glsl_ver = unsafe {
         let ptr = gl::GetString(gl::SHADING_LANGUAGE_VERSION);
         if ptr.is_null() {
            return Err(GLError::CouldParseVersion(
               "GLSL version is null".to_string(),
            ));
         }
         let cstr = std::ffi::CStr::from_ptr(ptr as *const i8);
         match cstr.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => return Err(GLError::CouldParseVersion(e.to_string())),
         }
      };

      let device = unsafe {
         let ptr = gl::GetString(gl::RENDERER);
         if ptr.is_null() {
            return Err(GLError::CouldParseVersion("renderer is null".to_string()));
         }
         let cstr = std::ffi::CStr::from_ptr(ptr as *const i8);
         match cstr.to_str() {
            Ok(s) => s.to_string(),
            Err(e) => return Err(GLError::CouldParseVersion(e.to_string())),
         }
      };

      Ok(Self {
         display,
         context,
         surface,
         glsl_ver,
         device,
      })
   }

   fn make_current(&self) -> Result<(), GLError> {
      let egl = egl::Instance::new(egl::Static);
      match egl.make_current(
         self.display,
         Some(self.surface),
         Some(self.surface),
         Some(self.context),
      ) {
         Err(e) => Err(GLError::MakeCurrentFailed(format!("{:?}", e))),
         Ok(_) => Ok(()),
      }
   }
}

impl Drop for GL {
   fn drop(&mut self) {
      let egl = egl::Instance::new(egl::Static);
      let _ = egl.make_current(self.display, None, None, None);
      let _ = egl.destroy_context(self.display, self.context);
      let _ = egl.destroy_surface(self.display, self.surface);
      let _ = egl.terminate(self.display);
   }
}

impl GL {
   pub(crate) fn clear(&self) {
      unsafe {
         gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
      }
   }

   pub(crate) fn set_clear(&self, color: RGBA) {
      unsafe {
         gl::ClearColor(color.0, color.1, color.2, color.3);
      }
   }
   pub(crate) fn resize(&self, size: Size2D) {
      unsafe {
         gl::Viewport(0, 0, size.w as i32, size.h as i32);
      }
   }
   pub(crate) fn poly_mode(&self, mode: PolyMode) {
      unsafe {
         match mode {
            PolyMode::WireFrame => gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE),
            PolyMode::Filled => gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL),
            PolyMode::Points => {
               gl::PointSize(10.0);
               gl::PolygonMode(gl::FRONT_AND_BACK, gl::POINT)
            }
         }
      }
   }
   pub(crate) fn enable_msaa(&self, enable: bool) {
      unsafe {
         match enable {
            true => gl::Enable(gl::MULTISAMPLE),
            false => gl::Disable(gl::MULTISAMPLE),
         }
      }
   }
   pub(crate) fn enable_depth(&self, enable: bool) {
      unsafe {
         match enable {
            true => gl::Enable(gl::DEPTH_TEST),
            false => gl::Disable(gl::DEPTH_TEST),
         }
      }
   }
   pub(crate) fn enable_alpha(&self, enable: bool) {
      unsafe {
         match enable {
            true => {
               gl::Enable(gl::BLEND);
               gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }
            false => gl::Disable(gl::BLEND),
         }
      }
   }
   pub(crate) fn enable_cull(&self, enable: bool) {
      unsafe {
         match enable {
            true => {
               gl::Enable(gl::CULL_FACE);
               gl::CullFace(gl::BACK);
            }
            false => gl::Disable(gl::CULL_FACE),
         }
      }
   }
   pub(crate) fn set_cull_face(&self, face: Cull) {
      unsafe {
         match face {
            Cull::Clock => gl::FrontFace(gl::CW),
            Cull::AntiClock => gl::FrontFace(gl::CCW),
         }
      }
   }
   pub(crate) fn set_point_size(&self, size: f32) {
      unsafe {
         gl::PointSize(size);
      }
   }
   pub(crate) fn set_wire_width(&self, width: f32) {
      unsafe { gl::LineWidth(width) }
   }
}
