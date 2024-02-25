use std::io;
use std::io::Write;
use ctrlc;

mod executor;
mod parser;
mod history;
mod autocompletion;

fn main() {
    ctrlc::set_handler(move || {
        println!();
    }).expect("Error setting Ctrl+C handler");

    let mut completion = autocompletion::Completion::init();
    completion.get_cmds();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command = parser::parse_input(&completion);
        println!();
        executor::exec_command(command);
    }
}
