use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

pub fn resolve(base: &str, path: &str) -> String {
    let base_path = if base.ends_with('/') {
        Path::new(&base[..base.len() - 1])
    } else {
        Path::new(base).parent().unwrap()
    };
    let path_path = Path::new(path);

    if path_path.is_relative() {
        String::from(base_path.join(path_path).to_str().unwrap())
    } else {
        String::from(path_path.to_path_buf().to_str().unwrap())
    }
}

pub fn resolve_uri(base: &str, path: &str) -> String {
    let base_path = if base.ends_with('/') {
        Path::new(&base[..base.len() - 1])
    } else {
        Path::new(base).parent().unwrap()
    };

    if path.starts_with('/') {
        return String::from(base_path.join(&path[1..]).to_str().unwrap());
    }

    let path_path = Path::new(path);

    if path_path.is_relative() {
        String::from(base_path.join(path_path).to_str().unwrap())
    } else {
        String::from(path_path.to_path_buf().to_str().unwrap())
    }
}

pub fn create(path: &str) -> io::Result<()> {
    if File::open(path).is_ok() {
        return Ok(());
    }

    let parent = Path::new(path).parent().unwrap().to_str().unwrap();
    if !fs::metadata(parent).is_ok() {
        fs::create_dir_all(parent)?;
    };

    let mut file = File::create(&path)?;
    file.write_all(b"")?;

    Ok(())
}
