use std::io;
use std::io::Write;
use ctrlc;

mod executor;
mod parser;
mod history;

fn main() {
    ctrlc::set_handler(move || {
        println!();
    }).expect("Error setting Ctrl+C handler");

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let command = parser::parse_input();
        executor::exec_command(command);
    }
}
