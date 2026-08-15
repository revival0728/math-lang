use libloading::{Library, Symbol};
use std::path::PathBuf;

use crate::rmapi::{RMExport, RMExportFun};

pub trait ModSystem: Default {
    fn read_mls(&mut self, path: PathBuf) -> Result<String, ()>;
    fn read_lib(&mut self, path: PathBuf) -> Result<RMExport, ()>;
}

#[derive(Default)]
pub struct FileSystem {
    lib: Vec<Library>,
}

impl ModSystem for FileSystem {
    fn read_mls(&mut self, path: PathBuf) -> Result<String, ()> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    }
    fn read_lib(&mut self, mut path: PathBuf) -> Result<RMExport, ()> {
        if path.extension().is_none() {
            path.add_extension(match std::env::consts::OS {
                "windows" => "dll",
                "linux" => "so",
                "macos" => "dylib",
                _ => "",
            });
        }
        unsafe {
            let lib = libloading::Library::new(path).map_err(|_| ())?;
            let export_fun: Symbol<RMExportFun> = lib.get(b"export_module\0").map_err(|_| ())?;
            let export = (*export_fun)();
            self.lib.push(lib);
            Ok(export)
        }
    }
}
