use std::process::Command;

use crate::parser;

pub fn exec_command(cmd: parser::Command) {
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
