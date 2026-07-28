#[allow(unused_imports)]
use codecrafters_shell::solutions::{
    cd_path, check_valid_path, find_exe_path_partial, find_files_in_pwd, get_pwd, run_cmd,
    run_cmd_with_output,
};

use rustyline::completion::{Candidate, Completer};
use rustyline::config::Configurer;
use rustyline::hint::{Hint, Hinter, HistoryHinter};
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{
    Changeset, Cmd, Completer, CompletionType, ConditionalEventHandler, Context, DefaultEditor,
    Editor, Event, EventContext, EventHandler, Helper, Highlighter, KeyCode, KeyEvent, Modifiers,
    RepeatCount, Result, Validator,
};
use shellwords;
use std::cell::Cell;
use std::collections::HashSet;
use std::fmt::format;
use std::fs;
use std::io::{Write, stdout};

#[derive(Helper, Validator, Highlighter)]
struct MyHelper {
    hints: HashSet<CommandHint>,
    tab_pressed: Cell<u8>,
    past_cursor_pos: Cell<usize>,
}

impl Completer for MyHelper {
    type Candidate = String;
    // TODO: let the implementers choose/find word boundaries ??? => Lexer

    /// Takes the currently edited `line` with the cursor `pos`ition and
    /// returns the start position and the completion candidates for the
    /// partial word to be completed.
    ///
    /// `("ls /usr/loc", 11)` => `Ok((3, vec!["/usr/local/"]))`
    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let mut start: usize;
        start = line.rfind(' ').map(|i| i + 1).unwrap_or(0);

        let partial = &line[start..pos]; // The part the user typed
        if partial.is_empty() {
            // handle one file in current dir
            let mut entries: Vec<_> = fs::read_dir(".")?.filter_map(|e| e.ok()).collect();
            if entries.len() == 1 {
                let mut entry = &entries[0];
                let mut entry_str = entry.file_name().to_string_lossy().to_string();
                let fixed_entry = format!("{}/", entry_str);
                return Ok((start, vec![fixed_entry]))
            }

            return Ok((start, Vec::new()));
        }

        let mut searches: Vec<String> = Vec::new();
        let mut matches: Vec<String> = Vec::new();
        searches = find_exe_path_partial(partial).unwrap();
        searches.push("exit".to_string());

        matches = searches
            .iter()
            .filter(|hint| hint.starts_with(partial))
            .map(|s| format!("{}", s))
            .collect();

        let mut matched_pwd_files = find_files_in_pwd(partial).unwrap();
        
        matched_pwd_files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        //println!("pwd matches {:?}", matched_pwd_files);
        if matched_pwd_files.len() >= 1 {
            //matches.append(&mut matched_pwd_files);
            let mut initial = matched_pwd_files[0].clone();
            initial.pop();
            return Ok((start, vec![initial]));
 
        } 
        if matches.is_empty() {
            return Ok((start, Vec::new()));
        }

        let mut seen = HashSet::new();
        let mut unique_matches: Vec<String> = matches
            .clone()
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect();

        unique_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        //println!("unique match {:?}", unique_matches);

        let first_match = unique_matches.first().clone().unwrap();
        let sub = &unique_matches[1..];

        if sub.iter().all(|content| content.starts_with(first_match)) {
            if unique_matches.len() > 1 {
                return Ok((start, unique_matches));
            } else {
                let last_match = unique_matches.last().clone().unwrap();
                if last_match.chars().last() == Some('/') {
                    return Ok((start, vec![last_match.to_string()]));
                }

                let last_formatted = format!("{} ", last_match);
                return Ok((start, vec![last_formatted]));
            }
        } else {
            if self.tab_pressed.get() == 0 {
                self.tab_pressed.set(1);
                print!("\x07");
                stdout().flush().unwrap();
                //println!("$ {}", partial);

                return Ok((start, Vec::new()));
            } else {
                println!(); // Print a newline after current typed input
                println!("{}", unique_matches.join("  ")); // Print matches separated by 2 spaces
                print!("$ {}", line); // Print prompt and re-display user's current typed command
                let _ = stdout().flush();
                self.tab_pressed.set(0);
                Ok((start, Vec::new()))
            }
        }
    }
}

//create hinter
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
        assert!(text.starts_with(complete_up_to));
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

impl Hinter for MyHelper {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<CommandHint> {
        None
        //     if line.is_empty() || pos < line.len() {
        //         return None;
        //     }

        //     self.hints
        //         .iter()
        //         .filter_map(|hint| {
        //             // expect hint after word complete, like redis cli, add condition:
        //             // line.ends_with(" ")
        //             if hint.display.starts_with(line) && line.len() >=3 {
        //                 Some(hint.suffix(pos))

        //             } else {
        //                 None
        //             }
        //         })
        //         .next()
    }
}

fn diy_hints() -> HashSet<CommandHint> {
    let mut set = HashSet::new();
    set.insert(CommandHint::new("echo ", "echo "));
    set.insert(CommandHint::new("exit ", "exit "));
    set.insert(CommandHint::new("set key value", "set "));
    set.insert(CommandHint::new("hget key field", "hget "));
    set.insert(CommandHint::new("hset key field value", "hset "));
    set
}

#[derive(Clone)]
struct TabHandler;
impl ConditionalEventHandler for TabHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        if let Some(k) = evt.get(0) {
            if let Some(txt) = ctx.hint_text() {
                //println!("enter tab");
                // println!("tab activated, text is {txt}");
                if txt.len() == 1 {
                    return Some(Cmd::Insert(1, txt.to_string()));
                }
            } else {
                //println!("error handling event");
                return Some(Cmd::Noop);
            }
        }
        //println!("error receiving input");
        None
    }
}

fn main() -> Result<()> {
    //TODO: Uncomment the code below to pass the first stage
    let h = MyHelper {
        hints: diy_hints(),
        tab_pressed: Cell::new(0),
        past_cursor_pos: Cell::new(0),
    };
    let mut editor: Editor<MyHelper, _> = Editor::new()?;
    editor.set_helper(Some(h));
    editor.set_completion_type(CompletionType::List);

    // //let mut rl= Editor::new()?;
    // editor.bind_sequence(
    //     KeyEvent::from('\t'),
    //     EventHandler::Conditional(Box::new(TabHandler)),
    // );

    loop {
        let readline = editor.readline("$ ");
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
