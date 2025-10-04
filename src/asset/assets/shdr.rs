use crate::asset::file::IOError;
use crate::asset::util;
use crate::*;

#[derive(Debug)]
pub enum FileError {
   //FILE
   WierdFile(String),
   Missing(String),
   IOError(IOError),
   //SHADER
   VertEmpty,
   FragEmpty,
   //OBJ
   NonTriangle(String),
   //PNG
   InvalidImage(String),
}

enum GLSL {
   ParsedCompute(String),
   ParsedPipeline { v_src: String, f_src: String },
   FailedPipeline { v_missing: bool, f_missing: bool },
}
impl GLSL {
   fn parse(src: &str, typ: ShaderMode) -> GLSL {
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

pub enum ShaderMode {
   Pipeline,
   Compute,
}

impl ShaderMode {
   pub(crate) fn is_compute(&self) -> bool {
      match self {
         ShaderMode::Pipeline => false,
         ShaderMode::Compute => true,
      }
   }
}

pub enum ShaderFile {
   Pipe { v_src: String, f_src: String },
   Comp(String),
}

impl ShaderFile {
   pub fn from_path(path: &str, typ: ShaderMode) -> Result<ShaderFile, FileError> {
      match file::name(path) {
         None => return Err(FileError::WierdFile(path.to_string())),
         Some(n) => n,
      };

      match file::ex(path) {
         None => return Err(FileError::WierdFile(path.to_string())),
         Some(ex) => match ex.to_lowercase().as_str() {
            "glsl" | "comp" | "shader" | "vert" | "frag" => ex,
            _ => return Err(FileError::WierdFile(path.to_string())),
         },
      };

      if file::exists_on_disk(path) {
         let src = match file::read_as_string(path) {
            Err(e) => return Err(FileError::IOError(e)),
            Ok(s) => s,
         };
         ShaderFile::from_src(&src, typ)
      } else {
         Err(FileError::Missing(path.to_string()))
      }
   }

   pub fn from_vert_frag_src(v_src: &str, f_src: &str) -> ShaderFile {
      ShaderFile::Pipe {
         v_src: v_src.to_string(),
         f_src: f_src.to_string(),
      }
   }

   pub fn from_src(src: &str, typ: ShaderMode) -> Result<ShaderFile, FileError> {
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
            let missing_str = format!("missing {missing}");
            Err(FileError::Missing(missing_str))
         }

         GLSL::ParsedPipeline { v_src, f_src } => Ok(ShaderFile::Pipe { v_src, f_src }),
         GLSL::ParsedCompute(src) => Ok(ShaderFile::Comp(src)),
      }
   }
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
