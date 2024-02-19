use std::io::Write;
use std::fs::OpenOptions;

use crate::executor;

pub struct History {
    history_string: String,
    cursor_index: usize,
}

impl History {
    pub fn add_to_string(&mut self, s: String) {
        self.history_string += &s;
    }

    pub fn set_cursor(&mut self, i: usize) {
        self.cursor_index = i;
    }

    pub fn info(&self) {
        println!("{} {}", self.history_string, self.cursor_index);
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
}

pub fn init() -> History {
    let history_string = String::new();
    let cursor_index = 0;

    History { history_string, cursor_index }
}
