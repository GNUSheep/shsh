use std::io::{self, Write};
use std::process::exit;
use std::collections::VecDeque;
use std::fs;

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyModifiers, Event},
    execute,
    cursor::{MoveTo, MoveLeft, MoveRight, position},
    terminal::{ClearType, Clear, size, DisableLineWrap, EnableLineWrap},
};
use regex::{Captures, Regex};

use crate::history;
use crate::executor;
use crate::autocompletion;

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub redirect_file: String,
}

impl Command {
    pub fn new() -> Self {
        let name = String::new();
        let args: Vec<String> = vec![];
        let redirect_file = String::new();

        Self { name, args, redirect_file }
    }
}

fn print_text(text: &String, with_prompt: bool, with_clear: bool, with_newline_before: bool) {
    let pos = get_cursor_position();

    if with_clear {
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("Problem with deleting char");
    }
    execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
    if with_newline_before && with_prompt {
        print!("\n$ {}", text);
    }else if with_prompt {
        print!("$ {}", text); 
    }

    if !with_prompt {
        print!("{}", text)
    }

    io::stdout().flush().unwrap();
}

fn clear_input(text: &String, begin_pos: [u16; 2], prompt: char) {
    let (col, _) = size().unwrap();

    let mut len_write = 0;
    let mut i = 0;
    let mut offset = 0;
    while len_write != text.len() {
        let pos = get_cursor_position();
        
        execute!(std::io::stdout(), MoveTo(0, begin_pos[1] + i as u16)).expect("Problem with moving cursor");
        
        len_write = offset;
        
        offset = if offset + usize::from(col - pos[0]) < text.len() {
            offset + usize::from(col - pos[0])    
        }else {
            offset + (text.len() - offset)
        };
       
        execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("Problem with deleting char");
    
        i += 1;
    }
    
    execute!(std::io::stdout(), MoveTo(0, begin_pos[1])).expect("Problem with moving cursor");
    print!("{} ", prompt);
    execute!(std::io::stdout(), MoveTo(2, begin_pos[1])).expect("Problem with moving cursor");
} 

fn render_text(text: String, mut begin_pos: [u16; 2], prompt: char) -> [u16; 2] {
    let mut cur_pos = get_cursor_position();
    let (col, row) = size().unwrap();

    clear_input(&text, begin_pos, prompt);

    let mut offset = if usize::from(col - begin_pos[0]) < text.len() {
        usize::from(col - begin_pos[0])
    }else {
        text.len()
    };

    let mut len_write = 0;
    while len_write != text.len() {
        print!("{}", &text[len_write..offset]);
        
        let pos = get_cursor_position();
        execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("Problem with moving cursor");
        let pos = get_cursor_position();

        len_write = offset;
        
        offset = if offset + usize::from(col - pos[0]) < text.len() {
            offset + usize::from(col - pos[0])    
        }else {
            offset + (text.len() - offset)
        };
    }
        
    execute!(std::io::stdout(), MoveTo(cur_pos[0], cur_pos[1])).expect("Problem with moving cursor");
    io::stdout().flush().unwrap();

    begin_pos
}

pub fn get_cursor_position() -> [u16; 2] {
    let pos = position().expect("Problem while getting cursor pos");

    [pos.0, pos.1]
}

fn split_user_input(input: &mut String) -> Vec<String> {
    let mut splited_input: Vec<String> = vec![];
    let mut word = String::new();

    let mut d_qoutes_occured = false;
    for c in input.chars() {
        match c {
            ' ' => {
                if !d_qoutes_occured {
                    if !word.is_empty() {
                        splited_input.push(word.clone());
                        word.clear();
                    }
                }else{
                    word.push(' ');
                }
            }
            '"' => d_qoutes_occured = !d_qoutes_occured,
            _ => word.push(c),
        }
    }

    if !word.is_empty() {
        splited_input.push(word.clone());
    }

    splited_input
}

fn get_line(mut begin_pos: [u16; 2], history: &mut history::History, completion: &autocompletion::Completion, prompt: char) -> String {
    let mut user_input = String::new();

    let mut history_index: i32 = -1;

    crossterm::terminal::enable_raw_mode().expect("Problem with entering raw mode");

    execute!(std::io::stdout(), DisableLineWrap).expect("Problem with disabling line wrap");
                        
    let (mut col, _) = size().unwrap();

    let mut tab_counter = 0;
    let mut tab_cmd_complete = String::new();

    let mut offset = 0;

    loop {
        let event = event::read().unwrap();
        match event {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('d') {
                    render_text("exit\n".to_string(), begin_pos, prompt);

                    crossterm::terminal::disable_raw_mode().expect("Problem with disabling raw mode");
                    execute!(std::io::stdout(), EnableLineWrap).expect("Problem with enabling line wrap");
                    exit(0);
                }

                if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('a') {
                    let pos = get_cursor_position();
                    execute!(std::io::stdout(), MoveTo(2, pos[1])).expect("Problem with moving cursor");
                    continue;
                } 

                if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
                    user_input.clear();
                    return user_input;
                } 

                match code {
                    KeyCode::Enter => {
                        crossterm::terminal::disable_raw_mode().expect("Problem with disabling raw mode");
                        execute!(std::io::stdout(), EnableLineWrap).expect("Problem with enabling line wrap");
                        return user_input.to_string();
                    }
                    KeyCode::Backspace => {
                        let pos = get_cursor_position();
                        tab_counter = 0;

                        if pos[0] - 1 < 2 {
                           continue 
                        }

                        user_input.remove((pos[0] - 3) as usize);

                        begin_pos = render_text(user_input.clone(), begin_pos, prompt);        
                        execute!(std::io::stdout(), MoveTo(pos[0] - 1, pos[1])).expect("Problem with moving cursor");

                    }
                    KeyCode::Up => {
                        let pos = get_cursor_position();
                        tab_counter = 0;

                        clear_input(&user_input, begin_pos, prompt);

                        let lines_num = history::get_lines_num();
                        if lines_num as i32 <= history_index + 1 {
                            user_input = "".to_string();
                            history_index = (lines_num) as i32;
                        }else {
                            history_index += 1;
                            while user_input == history.get_history(history_index) {
                                history_index += 1;
                            }
                            user_input = history.get_history(history_index);
                        }
                        begin_pos = render_text(user_input.clone(), begin_pos, prompt);
                        execute!(std::io::stdout(), MoveTo(user_input.len() as u16 + 2, pos[1])).expect("Problem with moving cursor");
                    }
                    KeyCode::Down => {
                        let pos = get_cursor_position();
                        tab_counter = 0;

                        clear_input(&user_input, begin_pos, prompt);
                        
                        if history_index - 1 < 0 {
                            user_input = "".to_string();
                            history_index = -1;
                        }else{
                            history_index -= 1;
                            while user_input == history.get_history(history_index) {
                                history_index -= 1;
                            }
                            user_input = history.get_history(history_index);
                        }
                        begin_pos = render_text(user_input.clone(), begin_pos, prompt);
                        execute!(std::io::stdout(), MoveTo(user_input.len() as u16 + 2, pos[1])).expect("Problem with moving cursor");
                    }
                    KeyCode::Left => {
                        let pos = get_cursor_position();

                        if pos[0] > 2 {
                            execute!(std::io::stdout(), MoveLeft(1)).expect("Problem with moving cursor");
                        }
                    }
                    KeyCode::Right => {
                        let pos = get_cursor_position();

                        if user_input.len() + 1 >= (pos[0]) as usize {                        
                            execute!(std::io::stdout(), MoveRight(1)).expect("Problem with moving cursor");
                        } 

                    }
                    KeyCode::Tab => {
                        tab_counter += 1;

                        if !user_input.contains(" ") {
                            if tab_counter == 1 {
                                tab_cmd_complete = user_input.clone();
                            }

                            if tab_counter > 1 {
                                let mut cmds = completion.find_completion(&tab_cmd_complete);
                                cmds.sort();

                                if cmds.len() == 0 {
                                    continue
                                }

                                if tab_counter == cmds.len() + 2 {
                                    tab_counter = 2;
                                } 
                                user_input = cmds[tab_counter - 2].clone();

                                print_text(&user_input, true, true, false);
                                continue
                            }

                            let mut cmds = completion.find_completion(&user_input);

                            if cmds.len() == 1 {
                                user_input = cmds[0].clone();
                                print_text(&user_input, true, true, false);
                                continue
                            }

                            if cmds.len() == 0 {
                                continue
                            }

                            cmds.sort();
                            let cmds_chunks: Vec<_> = cmds.chunks(2).collect(); 

                            let max_cmds_length = cmds.iter().map(|s| s.len()).max().unwrap_or(0);
                            
                            print_text(&user_input, true, true, false);
                            print_text(&"\n".to_string(), false, false, false);
                            for chunk in cmds_chunks {
                                let pos = get_cursor_position();
                                execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                                for cmd in chunk {
                                    print!("{:<len$}\t", cmd, len = max_cmds_length);
                                }
                                print!("\n");
                                io::stdout().flush().unwrap();
                            }
                            print_text(&user_input, true, true, false);

                            continue
                        }else {
                            let split: Vec<_>  = user_input.split_whitespace().collect();
                            let mut path: String = ".".to_string();
                            
                            if split.len() != 1 {
                                path = split[split.len()-1].to_string();
                            }

                            if let Ok(metadata) = fs::metadata(path.clone()) {
                                if !metadata.is_dir() {
                                    continue
                                }
                            }else {
                                continue
                            }
                            let dirs_completion = completion.get_paths(path);

                            print!("\n");
                            io::stdout().flush().unwrap();

                            let pos = get_cursor_position();
                            execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                            
                            for dir in dirs_completion {
                                print!("{}  ", dir);
                            }
                            print!("\n");
                            io::stdout().flush().unwrap();
                            
                            execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("Problem with moving cursor");
                            print_text(&user_input, true, true, false);

                            continue

                        }
                    }
                    KeyCode::Char(c) => {
                        let pos = get_cursor_position();

                        tab_counter = 0;

                        if usize::from(pos[0]) + (offset * usize::from(col)) <= user_input.len() + 1 {
                            user_input.insert(usize::from(pos[0])-2+(offset * usize::from(col)), c);       
                        }else {
                            user_input.push(c);
                        }

                        begin_pos = render_text(user_input.clone(), begin_pos, prompt);
                        execute!(std::io::stdout(), MoveTo(pos[0]+1, pos[1])).expect("Problem with moving cursor");
                    }
                    _ => {}
                }            
            }
            _ => {},
        }
    }
}

fn parse_multiline(mut begin_pos: [u16; 2], cmd_history: &mut history::History, completion: &autocompletion::Completion) -> String {
    let mut arg = String::new();
    loop {
        begin_pos[1] += 1;
        let user_input = get_line(begin_pos.clone(), cmd_history, completion, '>');
        
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

pub fn parse_input(completion: &autocompletion::Completion) -> VecDeque<Command> {
    let mut cmd_history = history::init();

    let pos = get_cursor_position();
    
    let mut user_input = get_line(pos, &mut cmd_history, completion, '$');

    if user_input.is_empty() {
        return Default::default();
    }

    cmd_history.add_to_string(user_input.clone());

    if let Some(c) = user_input.trim().chars().last() {
        if c == '\\' {
            user_input.pop();
            user_input += &parse_multiline(pos, &mut cmd_history, completion);
        }
    }
 
    let cmd_vec: Vec<_> = user_input.split("|").collect();

    let mut commands: VecDeque<Command> = Default::default();
    for cmd in cmd_vec {
        let mut command: Command = Command::new();

        let command_splited = split_user_input(&mut (cmd.to_string()));

        command.name = command_splited[0].clone();
        command.args = command_splited[1..].to_vec();

        let pattern = Regex::new("[$^][A-Za-z0-9]+").unwrap();

        command.name = pattern.replace_all(&command.name, |c: &Captures| {
            format!("{}", executor::get_env((&c[0])[1..].to_string()))
        }).to_string();

        for arg in &mut command.args {
            *arg = pattern.replace_all(arg, |c: &Captures| {
                format!("{}", executor::get_env((&c[0])[1..].to_string()))
            }).to_string();
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

        commands.push_back(command);
    }

    cmd_history.write_history();

    commands
}
