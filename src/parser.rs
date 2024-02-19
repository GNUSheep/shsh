use std::io::{self, Write};
use std::process::exit;
use std::collections::VecDeque;

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyModifiers, Event},
    execute,
    cursor::{MoveTo, position},
    terminal::{ClearType, Clear},
};

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub redirect_file: String,
}

impl Command {
    fn new() -> Self {
        let name = String::new();
        let args: Vec<String> = vec![];
        let redirect_file = String::new();

        Self { name, args, redirect_file }
    }
}

fn get_cursor_position() -> [u16; 2] {
    let pos = position().expect("Problem while getting cursor pos");

    [pos.0, pos.1]
}

fn get_line() -> String {
    let mut user_input = String::new();

    crossterm::terminal::enable_raw_mode().expect("Problem with entering raw mode");

    while let Event::Key(KeyEvent { code, modifiers, .. }) = event::read().unwrap() {
        match code {
            KeyCode::Enter => {
                crossterm::terminal::disable_raw_mode().expect("Problem with disabling raw mode");
                return user_input.to_string();
            }
            KeyCode::Backspace => {
                let pos = get_cursor_position();

                if pos[0] > 2 {
                    user_input.remove((pos[0] - 3) as usize);
                    execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("Problem with deleting char");
                    execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                    print!("$ {}", user_input);
                    io::stdout().flush().unwrap();
                }
            }
            KeyCode::Char(c) => {
                print!("{}", c);
                io::stdout().flush().unwrap();

                user_input.push(c)
            }
            _ => {}
        }

        if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('d') {
            exit(0);
        }
    }

    user_input.trim().to_string()
}

fn parse_multiline() -> String {
    let mut arg = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let user_input = get_line();

        if let Some(c) = user_input.chars().last() {
            if c != '\\' {
                arg += &user_input.to_string();
                break
            }
            arg += &(user_input[..user_input.len()-1].to_string());
        }  
    }
    arg
}

pub fn parse_input() -> VecDeque<Command> {
    let user_input = get_line();

    let cmd_vec: Vec<_> = user_input.split("|").collect();
    
    let mut commands: VecDeque<Command> = Default::default();
    for cmd in cmd_vec {
        let mut command: Command = Command::new();

        if let Some((name, args)) = cmd
            .split_whitespace()
            .collect::<Vec<_>>()
            .split_first()
        {
            command.name = name.to_string();
            command.args = args.iter().map(|v| v.to_string()).collect();

            if let Some(last_value) = args.last() {
                if let Some(c) = last_value.chars().last() {
                    if c == '\\' {
                        let args_len = command.args.len();
                        command.args[args_len-1] = command.args[args_len-1].trim_end_matches('\\').to_string();
                        command.args[args_len-1] += &parse_multiline();
                    }
                }
            }

            if let Some(index) = command.args.iter().position(|v| v.contains('>')) {
                if command.args[index].len() == 1 {
                    command.redirect_file = command.args[index..index+2].concat()[1..].to_string();
                    command.args.remove(index+1);
                }else{
                    command.redirect_file = command.args[index][1..].to_string();
                }
                command.args.remove(index);
            }
        }

        commands.push_back(command);
    }

    commands
}
