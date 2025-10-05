use crate::*;
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use std::ffi::CString;

enum GLSL {
   ParsedCompute(String),
   ParsedPipeline { v_src: String, f_src: String },
   FailedPipeline { v_missing: bool, f_missing: bool },
}
impl GLSL {
   fn parse(src: &str, typ: ShaderType) -> GLSL {
      let mut v_src = String::new();
      let mut f_src = String::new();

      if typ.is_compute() {
         return GLSL::ParsedCompute(src.to_string());
      }

      let glsl_lines = src.lines();

      let (mut v_found, mut f_found) = (false, false);
      let mut cur_src = &mut v_src;

      for line in glsl_lines {
         let line = line.trim();
         match line {
            "//v" | "//V" | "//vert" | "//VERT" | "//vertex" | "//VERTEX" | "// v" | "// V"
            | "// vert" | "// VERT" | "// vertex" | "// VERTEX" => {
               cur_src = &mut v_src;
               v_found = true;
            }
            "//f" | "//F" | "//frag" | "//FRAG" | "//fragment" | "//FRAGMENT" | "// f" | "// F"
            | "// frag" | "// FRAG" | "// fragment" | "// FRAGMENT" => {
               cur_src = &mut f_src;
               f_found = true;
            }
            _ => {
               cur_src.push_str(line);
               cur_src.push_str("\n")
            }
         }
      }
      let (mut v_missing, mut f_missing) = (false, false);
      if v_src.is_empty() || !v_found {
         v_missing = true
      }
      if f_src.is_empty() || !f_found {
         f_missing = true
      }

      match v_missing || f_missing {
         true => GLSL::FailedPipeline {
            v_missing,
            f_missing,
         },
         false => GLSL::ParsedPipeline { v_src, f_src },
      }
   }

   fn is_missing(&self) -> (bool, bool) {
      match self {
         GLSL::ParsedPipeline { .. } => (true, true),
         GLSL::FailedPipeline {
            v_missing,
            f_missing,
         } => (*v_missing, *f_missing),
         GLSL::ParsedCompute(_) => (true, true),
      }
   }
}

pub enum ShaderType {
   Pipeline,
   Compute,
}

impl ShaderType {
   pub(crate) fn is_compute(&self) -> bool {
      match self {
         ShaderType::Pipeline => false,
         ShaderType::Compute => true,
      }
   }
}

pub enum ShaderFile {
   Pipe { v_src: String, f_src: String },
   Comp(String),
}

impl ShaderFile {
   pub fn from_path(path: &str, typ: ShaderType) -> Result<ShaderFile, GLueError> {
      let wierd_err = Err(GLueError::from(
         GLueErrorKind::WierdFile,
         &format!("wierd file {path}"),
      ));
      match file::name(path) {
         None => return wierd_err,
         Some(n) => n,
      };

      match file::ex(path) {
         None => return wierd_err,
         Some(ex) => match ex.to_lowercase().as_str() {
            "glsl" | "comp" | "shader" | "vert" | "frag" => ex,
            _ => return wierd_err,
         },
      };

      if file::exists_on_disk(path) {
         let src = match file::read_as_string(path) {
            Err(e) => return Err(e),
            Ok(s) => s,
         };
         ShaderFile::from_src(&src, typ)
      } else {
         Err(GLueError::from(
            GLueErrorKind::Missing,
            &format!("missing file {path}"),
         ))
      }
   }

   pub fn from_vert_frag_src(v_src: &str, f_src: &str) -> ShaderFile {
      ShaderFile::Pipe {
         v_src: v_src.to_string(),
         f_src: f_src.to_string(),
      }
   }

   pub fn from_src(src: &str, typ: ShaderType) -> Result<ShaderFile, GLueError> {
      let glsl = GLSL::parse(&src, typ);
      match glsl {
         GLSL::FailedPipeline {
            v_missing,
            f_missing,
         } => {
            let missing = match (v_missing, f_missing) {
               (true, true) => "vert + frag",
               (true, _) => "vert",
               _ => "frag",
            };
            Err(GLueError::from(
               GLueErrorKind::MissingSrc,
               &format!("missing {missing}"),
            ))
         }

         GLSL::ParsedPipeline { v_src, f_src } => Ok(ShaderFile::Pipe { v_src, f_src }),
         GLSL::ParsedCompute(src) => Ok(ShaderFile::Comp(src)),
      }
   }

   pub fn compile(self) -> Result<Shader, GLueError> {
      let (src1, src2, is_compute) = match self {
         ShaderFile::Pipe { v_src, f_src } => (v_src, Some(f_src), false),
         ShaderFile::Comp(src) => (src, None, true),
      };

      let id = match link_program(&src1, &src2, is_compute) {
         Err(e) => return Err(e),
         Ok(id) => id,
      };

      let shader = Shader {
         workers: Workers::empty(),
         id,
         is_compute,
         tex_ids: vec![None; TexSlot::total_slots()],
         sbo_ids: vec![None; SBOSlot::total_slots()],
      };
      Ok(shader)
   }
}

fn compile_shader(src: &str, typ: ShaderSrcType) -> Result<u32, GLueError> {
   let src = match CString::new(src) {
      Err(e) => return Err(GLueError::wtf(&format!("c-string failed! {e}"))),
      Ok(s) => s,
   };
   unsafe {
      let shader_id = gl::CreateShader(gl_match_shader_type(&typ));
      gl::ShaderSource(shader_id, 1, &src.as_ptr(), ptr::null());
      gl::CompileShader(shader_id);

      match shader_compile_failure(shader_id, typ) {
         Ok(()) => Ok(shader_id as u32),
         Err(e) => Err(e),
      }
   }
}

fn link_program(src1: &str, src2: &Option<String>, is_compute: bool) -> Result<u32, GLueError> {
   let v = match is_compute {
      false => ShaderSrcType::Vert,
      true => ShaderSrcType::Compute,
   };

   unsafe {
      let program_id = gl::CreateProgram();
      let v_shader_id = match compile_shader(src1, v) {
         Err(e) => return Err(e),
         Ok(vs_id) => vs_id,
      };
      let mut f_shader_id = 0;
      gl::AttachShader(program_id, v_shader_id);
      match src2 {
         None => {}
         Some(frag) => {
            f_shader_id = match compile_shader(frag, ShaderSrcType::Frag) {
               Err(e) => return Err(e),
               Ok(fs_id) => fs_id,
            };
            gl::AttachShader(program_id, f_shader_id);
         }
      }
      gl::LinkProgram(program_id);

      match program_link_failure(program_id) {
         Err(e) => Err(e),
         Ok(()) => {
            delete_shader(v_shader_id);
            if src2.is_some() {
               delete_shader(f_shader_id);
            }
            Ok(program_id as u32)
         }
      }
   }
}

pub fn delete_shader(id: u32) {
   unsafe { gl::DeleteShader(id) }
}

pub(crate) fn delete_program(id: u32) {
   unsafe { gl::DeleteProgram(id) }
}

fn clone_slice_4(bytes: &[u8]) -> [u8; 4] {
   let mut cloned_bytes = [0; 4];
   for i in 0..4 {
      cloned_bytes[i] = bytes[i]
   }
   cloned_bytes
}
fn clone_slice(bytes: &[u8]) -> Vec<u8> {
   let mut cloned_bytes = Vec::new();
   for byte in bytes {
      cloned_bytes.push(*byte)
   }
   cloned_bytes
}
fn u32_to_vec_of_4_u8s(n: u32) -> Vec<u8> {
   let mut vec = Vec::new();
   let bytes = n.u8ify();
   for i in 0..4 {
      if bytes.len() > i {
         vec.push(bytes[i])
      } else {
         vec.push(0)
      }
   }
   vec
}

fn gl_match_shader_type(t: &ShaderSrcType) -> GLenum {
   match t {
      ShaderSrcType::Vert | ShaderSrcType::Compute => gl::VERTEX_SHADER,
      ShaderSrcType::Frag => gl::FRAGMENT_SHADER,
   }
}

unsafe fn shader_compile_failure(shader: GLuint, typ: ShaderSrcType) -> Result<(), GLueError> {
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
      let log = str::from_utf8(&log)
         .unwrap_or("unreachable-log")
         .to_string();
      Err(GLueError::from(
         GLueErrorKind::ShaderCompileFailed,
         &format!(
            "{} shader compile failed: {log}",
            match typ {
               ShaderSrcType::Vert => "vertex",
               ShaderSrcType::Frag => "fragment",
               ShaderSrcType::Compute => "compute",
            }
         ),
      ))
   } else {
      Ok(())
   }
}

unsafe fn program_link_failure(program: GLuint) -> Result<(), GLueError> {
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
      let log = str::from_utf8(&log)
         .unwrap_or("unreachable-log")
         .to_string();
      Err(GLueError::from(
         GLueErrorKind::ProgramLinkFailed,
         &format!("program link failed: {log}"),
      ))
   } else {
      Ok(())
   }
}
