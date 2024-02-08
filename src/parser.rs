use std;

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    fn new() -> Self {
        let name = String::new();
        let args: Vec<String> = vec![];

        Self{name, args}
    }
}

pub fn parse_input() -> Command {
    let mut user_input = String::new();

    let _ = match std::io::stdin().read_line(&mut user_input) {
        Ok(_) => (),
        Err(error) => panic!("Problem parsing user input, {:?}", error),
    };

    let mut command: Command = Command::new();
    if let Some((name, args)) = user_input.split_whitespace().collect::<Vec<_>>().split_first() {
        command.name = name.to_string();
        command.args = args.iter().map(|v| v.to_string()).collect();
    }

    command
}