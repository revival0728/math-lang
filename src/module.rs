use std::path::PathBuf;

pub trait ModSystem: Default + Clone {
    fn read(&self, path: PathBuf) -> Result<String, ()>;
}

#[derive(Default, Clone)]
pub struct FileSystem {}

impl ModSystem for FileSystem {
    fn read(&self, path: PathBuf) -> Result<String, ()> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }
}
