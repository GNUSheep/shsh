use std::io;
use std::io::Write;

mod executor;
mod parser;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let command = parser::parse_input();
        executor::exec_command(command)
    }
}
