use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::PathBuf;

pub(crate) enum IOError {
    NoPerms,
    Missing,
    NotValid,
    Unsupported(String),
    CouldNotMake,
    CouldNotRead,
    CouldNotWrite,
    Unknown,
}

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

pub(crate) fn write_str_to_disk(path: &str, name: &str, content: &str) -> Result<(), IOError> {
    write_bytes_to_disk(path, name, content.as_bytes())
}

pub(crate) fn write_bytes_to_disk(path: &str, name: &str, content: &[u8]) -> Result<(), IOError> {
    let pathbuf = PathBuf::from(path);
    if !pathbuf.exists() {
        //return NEResult::ER(NEError::file_missing(path));
        match fs::create_dir_all(path) {
            Err(_) => return Err(IOError::CouldNotMake),
            Ok(_) => {}
        };
    }

    let file_path = format!("{}{}", path, name);
    let mut file = match fs::File::create(&file_path) {
        Ok(f) => f,
        Err(_) => {
            return Err(IOError::CouldNotMake);
        }
    };
    match file.write_all(content) {
        Ok(_) => Ok(()),
        Err(_) => Err(IOError::CouldNotWrite),
    }
}

pub(crate) fn read_as_bytes(path: &str) -> Result<Vec<u8>, IOError> {
    let mut contents: Vec<u8> = Vec::new();

    let mut err;
    match fs::File::open(&path) {
        Ok(mut file) => match file.read_to_end(&mut contents) {
            Ok(_) => return Ok(contents),
            Err(e) => err = e,
        },
        Err(e) => {
            err = e;
        }
    }

    let io_err = match err.kind() {
        ErrorKind::NotFound | ErrorKind::InvalidInput => IOError::NotValid,
        ErrorKind::PermissionDenied => IOError::NoPerms,
        _ => IOError::Unknown,
    };
    Err(io_err)
}

pub(crate) fn read_as_string(path: &str) -> Result<String, IOError> {
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

    let io_err = match err.kind() {
        ErrorKind::NotFound | ErrorKind::InvalidInput => IOError::NotValid,
        ErrorKind::PermissionDenied => IOError::NoPerms,
        _ => IOError::Unknown,
    };
    Err(io_err)
}
