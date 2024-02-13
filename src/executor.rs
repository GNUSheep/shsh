use std::process::{Command, exit};
use std::path::Path;
use std::env;

use crate::parser;

fn get_env(name: String) -> String {
    match env::var(name) {
        Ok(value) => return value,
        Err(_) => return " ".to_string(),
    }
}

pub fn exec_command(mut cmd: parser::Command) {
    match cmd.name.as_str() {
        "cd" => {
            let mut path = get_env("HOME".to_string());
            if path == "" {
                path = "/".to_string();
            }

            match cmd.args.len() {
                0 => (),
                1 => {
                    path = cmd.args.pop().unwrap();
                }
                _ => {
                    println!("To many directions provided; see cd man");
                    return
                }
            };

            let path = Path::new(&path);

            if let Err(_) = env::set_current_dir(&path) {
                println!("Problem with changing directory");
            }

            return 
        }
        "" => return,
        "exit" => exit(0),
        _ => (),
    }

    let child = Command::new(cmd.name).args(cmd.args).spawn();

    match child {
        Ok(child) => {
            let _ = child
                .wait_with_output()
                .expect("Problem with waiting for child");
        }
        Err(err) => println!("Problem with executing command: {}", err),
    };
}
