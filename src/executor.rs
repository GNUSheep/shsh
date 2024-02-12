use std::process::Command;
use std::path::Path;
use std::env;

use crate::parser;

pub fn exec_command(cmd: parser::Command) {
    match cmd.name.as_str() {
        "cd" => {
            let mut path = Path::new("/");
            match cmd.args.len() {
                0 => (),
                1 => {
                    path = Path::new(&cmd.args[0]);
                }
                _ => {
                    println!("To many directions provided; see cd man");
                    return
                }
            };
            if let Err(_) = env::set_current_dir(&path) {
                println!("Problem with changing directory");
            }

            return 
        }
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
