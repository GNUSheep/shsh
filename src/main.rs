use std::io::Write;
use std::io;

mod parser;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let command = parser::parse_input();
        println!("{}, {:?}", command.name, command.args);
    }
}
