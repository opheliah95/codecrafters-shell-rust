#[allow(unused_imports)]
use codecrafters_shell::solutions::{
    cd_path, check_valid_path, get_pwd, run_cmd, run_cmd_with_output,
};

use rustyline::hint::{Hint, Hinter};
use rustyline::history::FileHistory;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Context, Editor, Helper, Highlighter, Result, Validator, KeyCode, KeyEvent,
    Modifiers, EventHandler, Cmd, DefaultEditor};

use shellwords;
use std::collections::HashSet;

#[derive(Completer, Helper, Validator, Highlighter)]
struct DIYHinter {
    // It's simple example of rustyline, for more efficient, please use ** radix trie **
    hints: HashSet<CommandHint>,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl CommandHint {
    fn new(text: &str, complete_up_to: &str) -> Self {
        //assert!(text.starts_with(complete_up_to));
        Self {
            display: text.into(),
            complete_up_to: complete_up_to.len(),
        }
    }

    fn suffix(&self, strip_chars: usize) -> Self {
        Self {
            display: self.display[strip_chars..].to_owned(),
            complete_up_to: self.complete_up_to.saturating_sub(strip_chars),
        }
    }
}

impl Hinter for DIYHinter {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        self.hints
            .iter()
            .filter_map(|hint| {
                // expect hint after word complete, like redis cli, add condition:
                // line.ends_with(" ")
                if hint.display.starts_with(line) {
                    Some(hint.suffix(pos))
                } else {
                    None
                }
            })
            .next()
    }
}

fn diy_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();
    set.insert(CommandHint::new("echo", "echo"));
    set.insert(CommandHint::new("exit", "exit"));
    set.insert(CommandHint::new("set key value", "set "));
    set.insert(CommandHint::new("hget key field", "hget "));
    set.insert(CommandHint::new("hset key field value", "hset "));
    set
}

fn main() -> Result<()> {
    //TODO: Uncomment the code below to pass the first stage
    let h = DIYHinter { hints: diy_hints() };
    let mut rl = Editor::new()?;

    //let mut rl= Editor::new()?;
     rl.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        EventHandler::Simple(Cmd::CompleteHint),
    );
    rl.set_helper(Some(h));
    // let stdin = io::stdin();
    // let mut buffer = [0u8; 1];

    // #[cfg(feature = "with-file-history")]
    // if rl.load_history("history.txt").is_err() {
    //     println!("No previous history.");
    // }
    loop {
        // let mut handle = stdin.lock();
        // let single_read = handle.read(&mut buffer);
        // if single_read.as_ref().unwrap() == &1 {
        //     let b = buffer[0];
        //     if let Ok(s) = std::str::from_utf8(&buffer) {
        //         print!("{s}");
        //     } else {
        //         print!("\\x{:02x}", b);
        //     }
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                let mut input = String::new();

                input = line.trim().to_string();
                let mut args = shellwords::split(&input).unwrap();

                if args.len() >= 1 {
                    let output = ["1>", ">", "2>", ">>", "1>>", "2>>"];
                    let mut split_index: Option<usize> = None;
                    for (index, val) in args.iter().enumerate() {
                        if output.contains(&val.as_str()) {
                            split_index = Some(index);
                            break;
                        }
                    }

                    if let Some(index) = split_index {
                        if index + 1 >= args.len() {
                            eprintln!("Error: Missing file path after redirection");
                            // Don't break, just continue to next prompt
                            continue;
                        }

                        let file_dir = args.split_off(index + 1);
                        let operator = args.pop();
                        // println!("now args is {:?}", args);
                        // println!("now file dir is: {:?}", file_dir);
                        run_cmd_with_output(args.clone(), file_dir[0].to_string(), operator);
                        continue;
                    }
                    match &args[0] {
                        cmd if cmd.len() == 0 => continue,
                        cmd if *cmd == "exit" => std::process::exit(0),
                        cmd if *cmd == "cd" => cd_path(&args[1..]),
                        cmd if *cmd == "pwd" => get_pwd(),
                        cmd if *cmd == "type" => {
                            if ["echo", "exit", "type", "pwd"].contains(&args[1].as_str()) {
                                println!("{} is a shell builtin", &args[1])
                            } else {
                                check_valid_path(&args);
                            }
                        }
                        _ => {
                            if !run_cmd(&args) {
                                println!("{}: command not found", input)
                            }
                        }
                    }
                }
            }
            Err(_) => println!("No input"),
        }
    }
}

// match input.trim() {
//     "xyz" =>println!("xyz: command not found"),
//     _ => ()
// }
