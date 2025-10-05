use crate::{GLueError, GLueErrorKind};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;

pub(crate) fn name(path: &str) -> Option<String> {
   let path = PathBuf::from(&path);
   match path.file_stem() {
      Some(n) => Some(n.to_string_lossy().to_string()),
      None => None,
   }
}

pub(crate) fn ex(path: &str) -> Option<String> {
   let path = PathBuf::from(&path);
   match path.extension() {
      Some(n) => Some(n.to_string_lossy().to_string()),
      None => None,
   }
}

pub(crate) fn exists_on_disk(path: &str) -> bool {
   let path = PathBuf::from(&path);
   path.exists()
}

pub(crate) fn write_str_to_disk(path: &str, name: &str, content: &str) -> Result<(), GLueError> {
   write_bytes_to_disk(path, name, content.as_bytes())
}

pub(crate) fn write_bytes_to_disk(path: &str, name: &str, content: &[u8]) -> Result<(), GLueError> {
   let pathbuf = PathBuf::from(path);

   if !pathbuf.exists() {
      match fs::create_dir_all(path) {
         Err(e) => {
            return Err(GLueError::from(
               GLueErrorKind::CouldNotMake,
               &format!("could not make dir {path} {e}"),
            ));
         }
         Ok(_) => {}
      };
   }

   let file_path = format!("{}{}", path, name);
   let mut file = match fs::File::create(&file_path) {
      Ok(f) => f,
      Err(e) => {
         return Err(GLueError::from(
            GLueErrorKind::CouldNotMake,
            &format!("could not make dir {file_path} {e}"),
         ));
      }
   };
   match file.write_all(content) {
      Ok(_) => Ok(()),
      Err(e) => Err(GLueError::from(
         GLueErrorKind::CouldNotWrite,
         &format!("could not write file {file_path} {e}"),
      )),
   }
}

pub(crate) fn read_as_bytes(path: &str) -> Result<Vec<u8>, GLueError> {
   let mut contents: Vec<u8> = Vec::new();

   let mut err;
   match fs::File::open(&path) {
      Ok(mut file) => match file.read_to_end(&mut contents) {
         Ok(_) => return Ok(contents),
         Err(e) => err = e,
      },
      Err(e) => err = e,
   }

   let glue_err = match err.kind() {
      ErrorKind::NotFound | ErrorKind::InvalidInput => GLueError::from(
         GLueErrorKind::WierdFile,
         &format!("wierd file {path} {err}"),
      ),
      ErrorKind::PermissionDenied => GLueError::from(
         GLueErrorKind::NoPerms,
         &format!("perms denied {path} {err}"),
      ),
      e => GLueError::wtf(&format!("unknown file error {e}")),
   };
   Err(glue_err)
}

pub(crate) fn read_as_string(path: &str) -> Result<String, GLueError> {
   let mut contents = String::new();

   let mut err;
   match fs::File::open(&path) {
      Ok(mut file) => match file.read_to_string(&mut contents) {
         Ok(_) => return Ok(contents),
         Err(e) => err = e,
      },
      Err(e) => {
         err = e;
      }
   }

   let glue_err = match err.kind() {
      ErrorKind::NotFound | ErrorKind::InvalidInput => GLueError::from(
         GLueErrorKind::WierdFile,
         &format!("wierd file {path} {err}"),
      ),
      ErrorKind::PermissionDenied => GLueError::from(
         GLueErrorKind::NoPerms,
         &format!("perms denied {path} {err}"),
      ),
      e => GLueError::wtf(&format!("unknown file error {err}")),
   };
   Err(glue_err)
}
