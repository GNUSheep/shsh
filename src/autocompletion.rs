use std::fs::{self, metadata};
use std::os::unix::fs::PermissionsExt;

use crate::executor;

pub struct Completion {
    cmds: Vec<String>,
}

impl Completion {
    pub fn init() -> Self {
        let cmds: Vec<String> = vec![];
        
        Self { cmds  }
    }

    fn get_binaries(dir: String) -> Vec<String> {
        let mut binaries: Vec<String> = vec![];

        if let Ok(files) = fs::read_dir(dir.clone()) {
            for file in files.flatten() {
                let path = file.path();

                if path.is_file() && path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false) {
                    binaries.push(file.file_name().into_string().unwrap());
                }
            }
        } else {
            println!("Error while reading dir: {}", dir);
        }

        binaries
    }

    pub fn get_cmds(&mut self) {
        let path_env = executor::get_env("PATH".to_string());
        if path_env.is_empty() {
            panic!("Error getting $PATH");
        }

        let path_dirs = path_env.split(":");
        for dir in path_dirs {
            let binaries = Self::get_binaries(dir.to_string());
            self.cmds.extend(binaries);
        }
    }

    pub fn find_cmds_completion(&self, prefix: &String) -> Vec<String> {
        let completions: Vec<String> = self.cmds
            .iter()
            .filter(|&e| e.starts_with(prefix))
            .cloned()
            .collect();

        completions
    }

    pub fn get_paths(&self, dir: &String) -> (Vec<String>, usize, bool)  {        
        match metadata(dir) {
            Ok(md) => {
                if md.is_dir() {
                    return (self.get_dir(dir), 0, true);
                }

                let cur_dir = self.get_dir(&".".to_string());
                (self.find_path_completion(cur_dir, dir), dir.len(), false)
            },
            Err(_) => {
                let cur_dir = self.get_dir(&".".to_string());
                (self.find_path_completion(cur_dir, dir), dir.len(), false)
            }
        }
    }

    pub fn get_dir(&self, dir: &String) -> Vec<String> {
        let mut entries: Vec<String> = vec![];

        if let Ok(files) = fs::read_dir(dir.clone()) {
            for file in files.flatten() {
                let path = file.path();

                if path.is_dir() {
                    entries.push(file.file_name().into_string().unwrap() +"/");
                } else {
                    entries.push(file.file_name().into_string().unwrap());
                }
            }
        } else {
            println!("Error while reading dir: {}", dir);
        }

        entries
    }
    
    pub fn find_path_completion(&self, dirs: Vec<String>, prefix: &String) -> Vec<String> {
        let completions: Vec<String> = dirs
            .iter()
            .filter(|&e| e.starts_with(prefix))
            .cloned()
            .collect();

        completions
    }
}
