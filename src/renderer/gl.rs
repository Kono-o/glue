use crate::asset::ATTRInfo;
use crate::renderer::handles::DrawMode;
use crate::renderer::{ShaderType, TexFormat};
use crate::{ATTRType, Cull, PolyMode, Size2D};
use crate::{TexFilter, TexWrap, TextureFile, RGBA};
use cgmath::{Matrix, Matrix4, Vector2};

use gl::types::*;
use khronos_egl as egl;
use std::ffi::{c_void, CString};
use std::ptr;

const TEX: u32 = gl::TEXTURE_2D;
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

    pub(crate) fn bind_shader(&self, prog_id: u32) {
        unsafe { gl::UseProgram(prog_id) }
    }
    pub(crate) fn unbind_program(&self) {
        unsafe { gl::UseProgram(0) }
    }

    pub(crate) fn bind_texture_at(&self, tex_id: u32, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(TEX, tex_id);
        }
    }
    pub(crate) fn unbind_texture(&self) {
        unsafe {
            gl::BindTexture(TEX, 0);
        }
    }

    pub(crate) fn bind_layouts(&self, v_id: u32) {
        unsafe {
            gl::BindVertexArray(v_id);
        }
    }
    pub(crate) fn bind_buffer(&self, id: u32) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
        }
    }
    pub(crate) fn unbind_layouts(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
    pub(crate) fn unbind_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }

    pub(crate) fn bind_index_buffer(&self, id: u32) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
        }
    }
    pub(crate) fn unbind_index_buffer(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    //SHADERS
    pub(crate) fn create_src_shader(&self, src: &str, typ: ShaderType) -> Result<u32, GLError> {
        let src = match CString::new(src) {
            Err(e) => return Err(GLError::RenderTime(e.to_string())),
            Ok(s) => s,
        };
        unsafe {
            let shader_id = gl::CreateShader(gl_match_shader_type(&typ));
            gl::ShaderSource(shader_id, 1, &src.as_ptr(), ptr::null());
            gl::CompileShader(shader_id);

            match gl_shader_compile_failure(shader_id) {
                Ok(()) => Ok(shader_id as u32),
                Err(e) => Err(e),
            }
        }
    }

    pub(crate) fn delete_shader(&self, id: u32) {
        unsafe { gl::DeleteShader(id) }
    }

    pub(crate) fn create_src_program(&self, vert: &str, frag: &str) -> Result<u32, GLError> {
        unsafe {
            let program_id = gl::CreateProgram();
            let v_shader_id = match self.create_src_shader(vert, ShaderType::Vert) {
                Err(e) => return Err(e),
                Ok(vs_id) => vs_id,
            };
            let f_shader_id = match self.create_src_shader(frag, ShaderType::Frag) {
                Err(e) => return Err(e),
                Ok(fs_id) => fs_id,
            };

            gl::AttachShader(program_id, v_shader_id);
            gl::AttachShader(program_id, f_shader_id);
            gl::LinkProgram(program_id);

            match gl_program_link_failure(program_id) {
                Err(e) => Err(e),
                Ok(()) => {
                    self.delete_shader(v_shader_id);
                    self.delete_shader(f_shader_id);
                    Ok(program_id as u32)
                }
            }
        }
    }
    pub(crate) fn delete_program(&self, id: u32) {
        unsafe { gl::DeleteProgram(id) }
    }

    pub(crate) fn create_texture(&self, tex: &TextureFile) -> u32 {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            self.bind_texture_at(id, 0);

            let wrap = gl_match_tex_wrap(&tex.wrap);
            let (min_fil, mag_fil) = gl_match_tex_filter(&tex.filter);

            gl::TexParameteri(TEX, gl::TEXTURE_WRAP_S, wrap);
            gl::TexParameteri(TEX, gl::TEXTURE_WRAP_T, wrap);
            gl::TexParameteri(TEX, gl::TEXTURE_MIN_FILTER, min_fil);
            gl::TexParameteri(TEX, gl::TEXTURE_MAG_FILTER, mag_fil);

            let (base, size) = gl_match_tex_fmt(&tex.fmt);
            let (width, height) = (tex.size.w as GLsizei, tex.size.h as GLsizei);

            gl::TexImage2D(
                TEX,
                0,
                size as GLint,
                width,
                height,
                0,
                base,
                gl::UNSIGNED_BYTE,
                &tex.bytes[0] as *const u8 as *const c_void,
            );
            gl::GenerateMipmap(TEX);
            self.unbind_texture()
        }
        id
    }
    pub(crate) fn delete_texture(&self, id: u32) {
        unsafe {
            gl::DeleteTextures(1, &id);
        }
    }

    pub(crate) fn get_uni_location(&self, id: u32, name: &str) -> u32 {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(id, c_name.as_ptr());
            if location == -1 {
                panic!("uniform '{name}' does not exist!");
            } else {
                location as u32
            }
        }
    }

    pub(crate) fn set_uni_i32(&self, id: u32, name: &str, int: i32) {
        unsafe {
            let loc = self.get_uni_location(id, name) as GLint;
            gl::Uniform1i(loc, int)
        }
    }

    pub(crate) fn set_uni_u32(&self, id: u32, name: &str, uint: u32) {
        unsafe {
            let loc = self.get_uni_location(id, name) as GLint;
            gl::Uniform1ui(loc, uint)
        }
    }

    pub(crate) fn set_uni_m4f32(&self, id: u32, name: &str, matrix: Matrix4<f32>) {
        unsafe {
            let loc = self.get_uni_location(id, name) as GLint;
            gl::UniformMatrix4fv(loc, 1, gl::FALSE, matrix.as_ptr())
        }
    }

    pub(crate) fn set_uni_vec2f32(&self, id: u32, name: &str, vec2: Vector2<f32>) {
        unsafe {
            let loc = self.get_uni_location(id, name) as GLint;
            gl::Uniform2f(loc, vec2.x, vec2.y)
        }
    }

    //BUFFERS
    pub(crate) fn create_buffer(&self) -> (u32, u32) {
        let (mut v_id, mut b_id): (u32, u32) = (0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut v_id);
            gl::GenBuffers(1, &mut b_id);
        }
        (v_id, b_id)
    }
    pub(crate) fn set_attr_layout(
        &self,
        attr: &ATTRInfo,
        attr_id: u32,
        stride: usize,
        local_offset: usize,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                attr_id,
                attr.elem_count as GLint,
                gl_match_attr_type(&attr.typ),
                gl::FALSE,
                stride as GLsizei,
                match local_offset {
                    0 => ptr::null(),
                    _ => local_offset as *const c_void,
                },
            );
            gl::EnableVertexAttribArray(attr_id);
        }
    }
    pub(crate) fn fill_buffer(&self, id: u32, buffer: &Vec<u8>) {
        unsafe {
            self.bind_buffer(id);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                buffer.len() as GLsizeiptr,
                &buffer[0] as *const u8 as *const c_void,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    pub(crate) fn fill_index_buffer(&self, id: u32, buffer: &Vec<u32>) {
        unsafe {
            self.bind_index_buffer(id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (buffer.len() * size_of::<GLint>()) as GLsizeiptr,
                &buffer[0] as *const u32 as *const c_void,
                gl::DYNAMIC_DRAW,
            );
        }
    }

    pub(crate) fn delete_buffer(&self, v_id: u32, b_id: u32) {
        unsafe {
            gl::DeleteVertexArrays(1, &v_id);
            gl::DeleteBuffers(1, &b_id);
        }
    }

    pub(crate) fn create_index_buffer(&self) -> u32 {
        let mut id: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        id
    }
    pub(crate) fn delete_index_buffer(&self, id: u32) {
        unsafe {
            gl::DeleteBuffers(1, &id);
        }
    }

    //DRAW
    pub(crate) fn clear(&self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
    pub(crate) fn draw_indexed(&self, draw_mode: &DrawMode, index_count: u32) {
        let draw_mode = gl_match_draw_mode(draw_mode);
        unsafe {
            gl::DrawElements(
                draw_mode,
                index_count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
    pub(crate) fn draw_array(&self, draw_mode: &DrawMode, vert_count: u32) {
        let draw_mode = gl_match_draw_mode(draw_mode);
        unsafe {
            gl::DrawArrays(draw_mode, 0, vert_count as GLsizei);
        }
    }
}

fn gl_match_draw_mode(dm: &DrawMode) -> GLenum {
    match dm {
        DrawMode::Points => gl::POINTS,
        DrawMode::Lines => gl::LINES,
        DrawMode::Triangles => gl::TRIANGLES,
        DrawMode::Strip => gl::TRIANGLE_STRIP,
    }
}
fn gl_match_shader_type(t: &ShaderType) -> GLenum {
    match t {
        ShaderType::Vert => gl::VERTEX_SHADER,
        ShaderType::Frag => gl::FRAGMENT_SHADER,
    }
}
fn gl_match_tex_fmt(tf: &TexFormat) -> (GLenum, GLenum) {
    let (base, bd) = match tf {
        TexFormat::R(bd) => (gl::RED, bd),
        TexFormat::RG(bd) => (gl::RG, bd),
        TexFormat::RGB(bd) => (gl::RGB, bd),
        TexFormat::RGBA(bd) => (gl::RGBA, bd),
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
fn gl_match_tex_filter(tf: &TexFilter) -> (GLint, GLint) {
    let (min, max) = match tf {
        TexFilter::Closest => (gl::NEAREST_MIPMAP_NEAREST, gl::NEAREST),
        TexFilter::Linear => (gl::LINEAR_MIPMAP_LINEAR, gl::LINEAR),
    };
    (min as GLint, max as GLint)
}
fn gl_match_tex_wrap(tf: &TexWrap) -> GLint {
    let wrap = match tf {
        TexWrap::Repeat => gl::REPEAT,
        TexWrap::Extend => gl::CLAMP_TO_EDGE,
        TexWrap::Clip => gl::CLAMP_TO_BORDER,
    };
    wrap as GLint
}
fn gl_match_attr_type(attr_type: &ATTRType) -> GLenum {
    match attr_type {
        ATTRType::I8 => gl::BYTE,
        ATTRType::U8 => gl::UNSIGNED_BYTE,
        ATTRType::I16 => gl::SHORT,
        ATTRType::U16 => gl::UNSIGNED_SHORT,
        ATTRType::I32 => gl::INT,
        ATTRType::U32 => gl::UNSIGNED_INT,
        ATTRType::F32 => gl::FLOAT,
        ATTRType::F64 => gl::DOUBLE,
    }
}

unsafe fn gl_shader_compile_failure(shader: GLuint) -> Result<(), GLError> {
    let mut success = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        let mut log_len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
        let mut log = Vec::new();
        log.resize(log_len as usize - 1, 0);

        gl::GetShaderInfoLog(
            shader,
            log_len as GLsizei,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );
        let log = std::str::from_utf8(&log)
            .unwrap_or("unreachable-log")
            .to_string();
        Err(GLError::RenderTime(format!("shader compile failed: {log}")))
    } else {
        Ok(())
    }
}

unsafe fn gl_program_link_failure(program: GLuint) -> Result<(), GLError> {
    let mut success = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
        let mut log_len = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
        let mut log = Vec::new();
        log.resize(log_len as usize - 1, 0);

        gl::GetProgramInfoLog(
            program,
            log_len as GLsizei,
            ptr::null_mut(),
            log.as_mut_ptr() as *mut GLchar,
        );
        let log = std::str::from_utf8(&log)
            .unwrap_or("unreachable-log")
            .to_string();
        Err(GLError::RenderTime(format!("program link failed: {log}")))
    } else {
        Ok(())
    }
}
