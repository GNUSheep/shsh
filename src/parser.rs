use std::io::{self, Write};
use std::process::exit;
use std::collections::VecDeque;

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

fn clear_input(begin_pos: [u16; 2], prompt: char) {
    execute!(std::io::stdout(), MoveTo(0, begin_pos[1])).expect("Problem with moving cursor");

    execute!(std::io::stdout(), Clear(ClearType::FromCursorDown)).expect("Problem with deleting char");
    print!("{} ", prompt);
    execute!(std::io::stdout(), MoveTo(2, begin_pos[1])).expect("Problem with moving cursor");
}

fn render_text(text: &String, mut cur_pos: [u16; 2], mut offset: usize, leave_the_cursor: bool) -> ([u16; 2], usize) {
    let (col, row) = size().unwrap();

    execute!(std::io::stdout(), MoveTo(cur_pos[0], cur_pos[1])).expect("Problem with moving cursor");
    execute!(std::io::stdout(), Clear(ClearType::FromCursorDown)).expect("Problem with deleting char");

    let mut end_index = if usize::from(col - cur_pos[0]) < text.len() {
        usize::from(col - cur_pos[0])
    }else {
        text.len()
    };

    let mut len_write = 0;
    let mut tmp_offset = 0;
    while len_write != text.len() {
        print!("{}", &text[len_write..end_index]);

        let pos = get_cursor_position();

        if row <= pos[1] + 1 && col - 1 == pos[0] && text.len() != end_index {
            print!("\n");
            offset += 1;
            cur_pos[1] -= 1;
        } else {
            tmp_offset += 1;
        }

        let old_pos = pos;

        execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("Problem with moving cursor");
        let pos = get_cursor_position();

        len_write = end_index;

        end_index = if end_index + usize::from(col - pos[0]) < text.len() {
            end_index + usize::from(col - pos[0])
        }else {
            end_index + (text.len() - end_index)
        };

        if len_write == text.len() && leave_the_cursor {
            execute!(std::io::stdout(), MoveTo(old_pos[0], old_pos[1])).expect("Problem with moving cursor");
            cur_pos = old_pos;
            offset += tmp_offset - 1;
        };
    }

    io::stdout().flush().unwrap();

    (cur_pos, offset)
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

fn get_line(begin_pos: [u16; 2], history: &mut history::History, completion: &autocompletion::Completion, prompt: char, mut offset: usize) -> String {
    let mut user_input = String::new();

    let mut history_index: i32 = -1;

    crossterm::terminal::enable_raw_mode().expect("Problem with entering raw mode");

    execute!(std::io::stdout(), DisableLineWrap).expect("Problem with disabling line wrap");

    let (col, row) = size().unwrap();

    let mut tab_counter = 0;
    let mut over_term = false;
    let mut tab_cmd_complete = String::new();

    loop {
        let event = event::read().unwrap();
        match event {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('d') {
                    render_text(&"exit\n".to_string(), begin_pos, offset, true);

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
                        let mut pos = get_cursor_position();
                        tab_counter = 0;
                        over_term = false;

                        if pos[0] - 1 < 2 {
                           continue
                        }

                        let pos_x_moved = (offset * col as usize + pos[0] as usize) - 3;
                        user_input.remove(pos_x_moved);

                        (pos, offset) = render_text(&user_input[pos_x_moved..].to_string(), [pos[0] - 1, pos[1]], offset, false);

                        execute!(std::io::stdout(), MoveTo(pos[0], pos[1])).expect("Problem with moving cursor");
                    }
                    KeyCode::Up => {
                        tab_counter = 0;
                        over_term = false;

                        offset = 0;
                        clear_input(begin_pos, prompt);

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
                        (_, offset) = render_text(&user_input, begin_pos, offset, true);
                    }
                    KeyCode::Down => {
                        tab_counter = 0;
                        over_term = false;

                        offset = 0;
                        clear_input(begin_pos, prompt);

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
                        (_, offset) = render_text(&user_input, begin_pos, offset, true);
                    }
                    KeyCode::Left => {
                        let pos = get_cursor_position();

                        if pos[0] <= 2 && offset == 0 {
                            continue
                        }

                        if pos[0] == 0 && offset != 0 {
                            execute!(std::io::stdout(), MoveTo(col - 1, pos[1] - 1)).expect("Problem with moving cursor");
                            offset -= 1;
                            continue;
                        }

                        execute!(std::io::stdout(), MoveLeft(1)).expect("Problem with moving cursor");
                    }
                    KeyCode::Right => {
                        let pos = get_cursor_position();

                        if user_input.len() + 1 >= offset * col as usize + (pos[0]) as usize {
                            execute!(std::io::stdout(), MoveRight(1)).expect("Problem with moving cursor");

                            if pos[0] + 1 >= col {
                                execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("Problem with moving cursor");
                                offset += 1;
                            }
                        }
                    }
                    KeyCode::Tab => {
                        tab_counter += 1;

                        if user_input == "" {
                            continue
                        }

                        if tab_counter == 1 {
                            tab_cmd_complete = user_input.clone();
                            let mut cmds = completion.find_completion(&tab_cmd_complete);
                            cmds.sort();

                            if cmds.len() < 1 {
                                continue;
                            }

                            let pos = get_cursor_position();
                        
                            execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");

                            let max_cmd_length = cmds.iter().map(|s| s.len()).max().unwrap_or(0);
                            let cols = col.div_ceil((max_cmd_length+4) as u16) - 1;
                            let rows = cmds.len().div_ceil(cols as usize);

                            over_term = if rows >= (row - pos[1]) as usize {
                                true
                            }else { false };
                            
                            println!();
                            for (i, cmd) in cmds.iter().enumerate() {
                                print!("{:<width$}", cmd, width = max_cmd_length+4);    

                                if (i + 1) % cols as usize == 0 && i != 0 {
                                    println!();
                                    let pos = get_cursor_position();
                                    execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                                }
                            }

                            if over_term {
                                let pos = get_cursor_position();
                                if pos[0] != 0 {
                                    println!();
                                    execute!(std::io::stdout(), MoveTo(0, pos[1]+1)).expect("Problem with moving cursor");
                                }
                                
                                print!("$ ");
                                let pos = get_cursor_position();
                                render_text(&user_input, pos, offset, true);
                            } else {
                                execute!(std::io::stdout(), MoveTo(pos[0], pos[1])).expect("Problem with moving cursor");
                            }
                            
                            io::stdout().flush().unwrap();
                        }

                        if tab_counter > 1 {
                            let mut cmds = completion.find_completion(&tab_cmd_complete);
                            cmds.sort();

                            if cmds.len() == 0 {
                                continue;
                            }

                            let pos = get_cursor_position();
                            execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                            
                            if tab_counter == cmds.len() + 2 {
                                tab_counter = 2;
                            }

                            if over_term {
                                execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("Problem with deleting char");
                                user_input = cmds[tab_counter - 2].clone();
                                print!("{} {}", prompt, user_input);

                                if cmds.len() == 0 {
                                    continue;
                                }

                                io::stdout().flush().unwrap();
                                
                                continue;
                            }

                            execute!(std::io::stdout(), Clear(ClearType::CurrentLine)).expect("Problem with deleting char");
                            user_input = cmds[tab_counter - 2].clone();
                            print!("{} {}", prompt, user_input);

                            if cmds.len() == 0 {
                                continue;
                            }
                            
                            io::stdout().flush().unwrap();
                        }


                        // let mut cmds = completion.find_completion(&user_input);

                        //     if cmds.len() == 1 {
                        //         user_input = cmds[0].clone();
                        //         print_text(&user_input, true, true, false);
                        //         continue
                        //     }

                        //     if cmds.len() == 0 {
                        //         continue
                        //     }

                        //     cmds.sort();
                        //     let cmds_chunks: Vec<_> = cmds.chunks(2).collect();

                        //     let max_cmds_length = cmds.iter().map(|s| s.len()).max().unwrap_or(0);

                        //     print_text(&user_input, true, true, false);
                        //     print_text(&"\n".to_string(), false, false, false);
                        //     for chunk in cmds_chunks {
                        //         let pos = get_cursor_position();
                        //         execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");
                        //         for cmd in chunk {
                        //             print!("{:<len$}\t", cmd, len = max_cmds_length);
                        //         }
                        //         print!("\n");
                        //         io::stdout().flush().unwrap();
                        //     }
                        //     print_text(&user_input, true, true, false);

                        //     continue
                        // }else {
                        //     let split: Vec<_>  = user_input.split_whitespace().collect();
                        //     let mut path: String = ".".to_string();

                        //     if split.len() != 1 {
                        //         path = split[split.len()-1].to_string();
                        //     }

                        //     if let Ok(metadata) = fs::metadata(path.clone()) {
                        //         if !metadata.is_dir() {
                        //             continue
                        //         }
                        //     }else {
                        //         continue
                        //     }
                        //     let dirs_completion = completion.get_paths(path);

                        //     print!("\n");
                        //     io::stdout().flush().unwrap();

                        //     let pos = get_cursor_position();
                        //     execute!(std::io::stdout(), MoveTo(0, pos[1])).expect("Problem with moving cursor");

                        //     for dir in dirs_completion {
                        //         print!("{}  ", dir);
                        //     }
                        //     print!("\n");
                        //     io::stdout().flush().unwrap();

                        //     execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("Problem with moving cursor");
                        //     print_text(&user_input, true, true, false);

                        //     continue

                        // }
                    }
                    KeyCode::Char(c) => {
                        let mut pos = get_cursor_position();

                        tab_counter = 0;
                        over_term = false;
    
                        if usize::from(pos[0]) + (offset * usize::from(col)) <= user_input.len() + 1 {
                            user_input.insert(usize::from(pos[0])+(offset * usize::from(col)) - 2, c);
                        }else {
                            user_input.push(c);
                        }

                        if offset == 0 {
                            (pos, offset) = render_text(&user_input[usize::from(pos[0] - 2)..].to_string(), pos, offset, false);
                        }else {
                            (pos, offset) = render_text(&user_input[usize::from(offset as u16 * col + pos[0]) - 2..].to_string(), pos, offset, false);
                        }

                        execute!(std::io::stdout(), MoveTo(pos[0]+1, pos[1])).expect("Problem with moving cursor");

                        let pos = get_cursor_position();
                        if user_input.len() + 1 >= usize::from(pos[0]) + (offset * usize::from(col)) && pos[0] + 1 == col {
                             execute!(std::io::stdout(), MoveTo(0, pos[1] + 1)).expect("problem with moving cursor");
                             offset += 1;
                        }
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
        print!("\n> ");
        let user_input = get_line([2, begin_pos[1]], cmd_history, completion, '>', 0);

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

    let mut user_input = get_line(pos, &mut cmd_history, completion, '$', 0);

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
