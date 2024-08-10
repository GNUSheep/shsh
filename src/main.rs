use std::io;
use std::io::Write;
use crossterm::cursor::MoveTo;
use ctrlc;
use parser::get_cursor_position;
use crossterm::execute;

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
        let pos = get_cursor_position();
        
        execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
        print!("$ ");
        io::stdout().flush().unwrap();
        execute!(std::io::stdout(), MoveTo(2, pos[1])).expect("Problem with moving cursor");
        
        let command = parser::parse_input(&completion);
        println!();
        executor::exec_command(command);
    }
}
