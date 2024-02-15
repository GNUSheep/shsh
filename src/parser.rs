use std;
use std::process::exit;
use std::collections::VecDeque;

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    fn new() -> Self {
        let name = String::new();
        let args: Vec<String> = vec![];

        Self { name, args }
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
        }

        commands.push_back(command);
    }

    commands
}
