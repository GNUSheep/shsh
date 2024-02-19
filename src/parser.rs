use std::io;
use std::io::Write;
use std::process::exit;
use std::collections::VecDeque;

use crate::history;

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

fn get_line() -> String {
    let mut user_input = String::new();

    let _ = match std::io::stdin().read_line(&mut user_input) {
        Ok(bytes) => {
            if bytes == 0 {
                exit(0)
            }
        },
        Err(error) => panic!("Problem parsing user input, {:?}", error),
    };

    user_input.trim().to_string()
}

fn parse_multiline(cmd_history: &mut history::History) -> String {
    let mut arg = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let user_input = get_line();
        
        cmd_history.add_to_string(user_input.clone());
        
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

    let mut cmd_history = history::init();
    cmd_history.add_to_string(user_input.clone());
    cmd_history.set_cursor(user_input.len());

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
                        command.args[args_len-1] += &parse_multiline(&mut cmd_history);
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

    cmd_history.write_history();

    commands
}
