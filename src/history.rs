use std::io::{Write, BufReader, BufRead};
use std::fs::OpenOptions;
use rev_lines::RevLines;

use crate::executor;

pub fn get_lines_num() -> usize {
    let path = executor::get_env("HOME".to_string());
    match OpenOptions::new().read(true).open(path+&"/.shsh_history") {
        Ok(f) => {
            BufReader::new(f).lines().count()
        },
        Err(_) => panic!("Error with getting history, cannot open file"),
    }
}

pub struct History {
    history_string: String,
}

impl History {
    pub fn add_to_string(&mut self, s: String) {
        self.history_string += &s;
    }

    pub fn write_history(&self) {
        let path = executor::get_env("HOME".to_string());
        match OpenOptions::new().create(true).append(true).open(path+&"/.shsh_history") {
            Ok(mut f) => {
                match writeln!(&mut f, "{}", &self.history_string) {
                    Err(e) => panic!("Problem with writing to file, {}", e),
                    _ => (),
                }
            },
            Err(_) => panic!("Error with writing history, cannot open file"),
        };
    }

    pub fn get_history(&self, pos: i32) -> String {
        let path = executor::get_env("HOME".to_string());
        match OpenOptions::new().read(true).open(path+&"/.shsh_history") {
            Ok(f) => {
                let mut rev_lines = RevLines::new(f);
                return rev_lines.nth(pos as usize).unwrap().unwrap();
            },
            Err(_) => panic!("Error with getting history, cannot open file"),
        }
    }
}

pub fn init() -> History {
    let history_string = String::new();

    History { history_string }
}
