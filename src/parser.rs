use std;
use std::process::exit;
use std::collections::VecDeque;

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

pub fn parse_input() -> VecDeque<Command> {
    let mut user_input = String::new();

    let _ = match std::io::stdin().read_line(&mut user_input) {
        Ok(bytes) => {
            if bytes == 0 {
                exit(0)
            }
        },
        Err(error) => panic!("Problem parsing user input, {:?}", error),
    };

    let cmd_vec: Vec<_> = user_input.trim().split("|").collect();
    
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
            
            if let Some(index) = args.iter().position(|v| v.contains('>')) {
                command.redirect_file = command.args[index..].concat()[1..].to_string();
                command.args = command.args[..index].to_vec();
            }
        }

        commands.push_back(command);
    }

    commands
}
