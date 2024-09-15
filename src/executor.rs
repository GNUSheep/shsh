use std::process::{Command, exit, Stdio, Child};
use std::collections::VecDeque;
use std::path::Path;
use std::fs::File;
use std::env;

use regex::Regex;

use crate::parser;

pub fn get_env(name: String) -> String {
    match env::var(name) {
        Ok(value) => return value,
        Err(_) => return " ".to_string(),
    }
}

pub fn exec_command(mut cmds: VecDeque<parser::Command>) {
    let mut prev_cmd = None;

    while let Some(mut cmd) = cmds.pop_front() {
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
            "ls" => cmd.args.push("--color=auto".to_string()),
            "grep" => cmd.args.push("--color=auto".to_string()),
            "export" => {
                let pattern = Regex::new("[A-Za-z0-9]+=[A-Za-z0-9]+").unwrap();
                for arg in cmd.args {
                    if pattern.is_match(&arg) {
                        let args: Vec<_> = arg.split('=').collect();

                        env::set_var(args[0], args[1]);
                    }else {
                        println!("Wrong command usage");
                        return
                    }
                }
                return
            }
            "" => return,
            "exit" => exit(0),
            _ => (),
        }

        let stdin = prev_cmd.map_or(Stdio::inherit(), |out: Child| Stdio::from(out.stdout.unwrap()));

        let mut stdout = Stdio::inherit();
        if !cmds.is_empty() {
            stdout = Stdio::piped();
        }

        if !cmd.redirect_file.is_empty() {
            match File::create(cmd.redirect_file) {
                Ok(file) => {
                    stdout = Stdio::from(file);
                }
                Err(_) => println!("Problem with redirecting to file"),
            }
        }

        let child = Command::new(cmd.name).args(cmd.args).stdout(stdout).stdin(stdin).spawn();

        match child {
            Ok(mut child) => {
                child
                    .wait()
                    .expect("Problem with waiting for child");
                prev_cmd = Some(child);

            }
            Err(err) => {
                println!("Problem with executing command: {}", err);
                prev_cmd = None;
            }
        };
    }
}
