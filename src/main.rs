use codecrafters_shell::solutions::handle_piped_cmds;
#[allow(unused_imports)]
use codecrafters_shell::solutions::{
    cd_path, check_file_extension, check_valid_path, find_exe_path_partial, find_files_in_pwd,
    get_pwd, handle_dual_command, handle_jobs, run_cmd, run_cmd_with_output, strip_trailing_nl,
};

use codecrafters_shell::solutions::cmd::{CompleteCmd, Job, JobList};
use codecrafters_shell::solutions::history::HistHelper;
use rustyline::completion::{Candidate, Completer};
use rustyline::config::Configurer;
use rustyline::hint::{Hint, Hinter, HistoryHinter};
use rustyline::history::{DefaultHistory, History, SearchDirection};
use rustyline::{
    Changeset, Cmd, Completer, CompletionType, ConditionalEventHandler, Context, DefaultEditor,
    Editor, Event, EventContext, EventHandler, Helper, Highlighter, KeyCode, KeyEvent, Modifiers,
    RepeatCount, Result, Validator,
};
use shellwords;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::{Write, pipe, stdout};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{fs, vec};
use tokio;

#[derive(Clone, Helper, Validator, Highlighter)]
struct MyHelper {
    tab_pressed: Cell<u8>,
    past_cursor_pos: Cell<usize>,
    complete_history: Rc<RefCell<Vec<CompleteCmd>>>,
    cached_complete: RefCell<String>,
}

impl MyHelper {
    fn add_history(&mut self, cmd: CompleteCmd) {
        self.complete_history.borrow_mut().push(cmd);
    }
}

impl Completer for MyHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        let mut start: usize;
        start = line.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let partial = &line[start..pos];
        let history = self.complete_history.borrow();
        let input: Vec<&str> = line.split(' ').collect();
        let COMP_LINE = line.trim();
        let COMP_POINT = COMP_LINE.len().to_string();

        // Check if there's a -C completer registered for the command
        if input.len() > 0 {
            let command_name = input[0];
            if let Some(completer_cmd) = history
                .iter()
                .find(|cmd| cmd.cmd == command_name && cmd.flag == "-C")
            {
                // Found a -C completer, invoke it with the proper arguments
                let word_being_completed = partial;
                let previous_word = if input.len() > 1 {
                    input[input.len() - 2]
                } else {
                    ""
                };

                // Execute the completer script with: command_name, word_being_completed, previous_word
                let output = Command::new(&completer_cmd.path)
                    .arg(command_name)
                    .arg(word_being_completed)
                    .arg(previous_word)
                    .env("COMP_LINE", COMP_LINE)
                    .env("COMP_POINT", COMP_POINT)
                    .output()?;

                if output.status.success() {
                    let completions = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>();

                    if !completions.is_empty() {
                        // Add trailing space if there's only one completion
                        if completions.len() == 1 {
                            let final_result = format!("{} ", completions[0]);
                            return Ok((start, vec![final_result]));
                        }
                        return Ok((start, completions));
                    }
                }

                // If completer failed or returned nothing, fall through to other completion logic
            }
        }

        // Original completion logic for builtins and file completion
        if partial.is_empty() {
            if history.len() > 0 {
                for cmd in history.iter() {
                    let cmd_input = line.trim();
                    if cmd_input == cmd.cmd {
                        *self.cached_complete.borrow_mut() = cmd_input.to_string();
                        let mut output = strip_trailing_nl(&mut cmd.read_stdout());
                        output = format!("{output} ");
                        return Ok((start, vec![output]));
                    }
                }
            }

            // if &*self.cached_complete.borrow() == input[0] {
            //     println!("input is complete: {}", input[0])
            // }

            *self.cached_complete.borrow_mut() = String::new();
            let mut entries: Vec<_> = fs::read_dir(".")?.filter_map(|e| e.ok()).collect();
            if entries.len() == 1 {
                let entry = &entries[0];
                let entry_str = entry.file_name().to_string_lossy().to_string();
                let fixed_entry = format!("{}/", entry_str);
                return Ok((start, vec![fixed_entry]));
            }

            return Ok((start, Vec::new()));
        }

        *self.cached_complete.borrow_mut() = String::new();

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
        let cloned_match_pwd = matched_pwd_files.clone();
        let mut pwd_all_matched: bool = true;
        if !check_file_extension(cloned_match_pwd) {
            pwd_all_matched = false;
            if self.tab_pressed.get() == 0 {
                self.tab_pressed.set(1);
                print!("\x07");
                stdout().flush().unwrap();
                return Ok((start, Vec::new()));
            } else {
                println!();
                println!("{}", matched_pwd_files.join("  "));
                print!("$ {}", line);
                let _ = stdout().flush();
                self.tab_pressed.set(0);
                return Ok((start, Vec::new()));
            }
        }
        if matched_pwd_files.len() > 1 {
            let mut initial = matched_pwd_files[0].clone();
            initial.pop();
            return Ok((start, vec![initial]));
        }

        if matched_pwd_files.len() == 1 {
            let mut initial = matched_pwd_files[0].clone();
            if initial.chars().last() == Some('/') {
                return Ok((start, vec![initial]));
            }
            initial = format!("{} ", initial);
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

        let first_match = unique_matches.first().cloned().unwrap_or_default();
        let sub = &unique_matches[1..];

        if sub.iter().all(|content| content.starts_with(&first_match)) {
            if unique_matches.len() > 1 {
                return Ok((start, unique_matches));
            } else {
                let last_match = unique_matches.last().cloned().unwrap_or_default();

                if last_match.chars().last() == Some('/') {
                    return Ok((start, vec![last_match]));
                }

                let last_formatted = format!("{} ", last_match);
                return Ok((start, vec![last_formatted]));
            }
        } else {
            if self.tab_pressed.get() == 0 {
                self.tab_pressed.set(1);
                print!("\x07");
                stdout().flush().unwrap();
                return Ok((start, Vec::new()));
            } else {
                println!();
                println!("{}", unique_matches.join("  "));
                print!("$ {}", line);
                let _ = stdout().flush();
                self.tab_pressed.set(0);
                Ok((start, Vec::new()))
            }
        }
    }
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
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let last_finished_job = Arc::new(Mutex::new("".to_string()));
    let last_finished_job_id = Arc::new(Mutex::new(0));
    //let last_finished_job_clone = Arc::clone(&last_finished_job);
    let mut job_list: JobList = JobList {
        jobs: Vec::new(),
        last_finished_job: last_finished_job,
        last_job_pid: last_finished_job_id,
        discarded_pid: HashSet::new(),
    };
    let mut job_id = 1;
    let mut complete_cmd_history: Vec<CompleteCmd> = Vec::new();
    let complete_history = Rc::new(RefCell::new(Vec::<CompleteCmd>::new()));
    let cached_complete_cmd = RefCell::new(String::new());

    let mut h = MyHelper {
        tab_pressed: Cell::new(0),
        past_cursor_pos: Cell::new(0),
        complete_history: complete_history.clone(),
        cached_complete: cached_complete_cmd,
    };

    // handle file history
    let mut history_handler = HistHelper::new();
    let mut editor: Editor<MyHelper, _> = Editor::new()?;
    editor.set_helper(Some(h));
    editor.set_completion_type(CompletionType::List);

    // hist handler setup
    let mut hist_handler_shared = history_handler.clone();
    let mut editor_handler = history_handler.clone();
    // //let key_vec: Vec<KeyEvent> = [KeyEvent(KeyCode::Down, Modifiers::NONE), KeyEvent(KeyCode::Up, Modifiers::NONE)].to_vec();
    // editor.bind_sequence(
    //     Event::Any,
    //     EventHandler::Conditional(Box::new(editor_handler)),
    // );

    editor.bind_sequence(KeyEvent(KeyCode::Down, Modifiers::NONE), Cmd::NextHistory);
    editor.bind_sequence(KeyEvent(KeyCode::Up, Modifiers::NONE), Cmd::PreviousHistory);

    // history path

    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("history.txt")
        .unwrap();

    if editor.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = editor.readline("$ ");

        match readline {
            Ok(line) => {
                // reset index
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
                            continue;
                        }

                        let file_dir = args.split_off(index + 1);
                        let operator = args.pop();
                        run_cmd_with_output(args.clone(), file_dir[0].to_string(), operator);
                        continue;
                    }

                    // clear job
                    let unused_id = job_list.return_smallest_id();
                    //let mut temp_id = job_id_temp;
                    //println!("smallest id: {unused_id}");
                    if unused_id != 0 {
                        job_id = unused_id;
                    }

                    // handle pipes
                    if args.contains(&"|".to_string()) {
                        let pipe_args: Vec<&[String]> = args.split(|c| c == "|").collect();
                        handle_piped_cmds(pipe_args);
                        //println!("{dual_pipe_out}");
                        stdout().flush().unwrap();
                        continue;
                    }

                    // add cmd to history
                    //editor.add_history_entry(line.as_str())?;
                    writeln!(file, "{line}");
                    hist_handler_shared.add_history(&input);
                    hist_handler_shared.set_index_to_end();

                    match &args[0] {
                        cmd if cmd.len() == 0 => continue,
                        cmd if *cmd == "exit" => std::process::exit(0),
                        cmd if *cmd == "cd" => cd_path(&args[1..]),
                        cmd if *cmd == "pwd" => get_pwd(),
                        cmd if *cmd == "history" => {
                            if args.len() > 1 {
                                let hist_limit = args[1].parse::<usize>().unwrap();
                                hist_handler_shared.show_n_lines_history(hist_limit);
                                continue;
                            }
                            hist_handler_shared.show_all_history();
                        }
                        cmd if *cmd == "jobs" => {
                            // check if joblist is empty
                            if job_list.jobs.len() == 0 {
                                stdout().flush().unwrap();
                                continue;
                            }
                            //remove finished jobs if shown then list all jobs
                            job_list.remove_job();
                            job_list.list_jobs();
                            continue;
                        }
                        cmd if *cmd == "type" => {
                            if ["echo", "exit", "type", "pwd", "complete", "jobs", "history"]
                                .contains(&args[1].as_str())
                            {
                                println!("{} is a shell builtin", &args[1]);
                                job_list.display_last_job();
                            } else {
                                check_valid_path(&args);
                            }
                        }
                        cmd if *cmd == "complete" => {
                            if args.len() < 3 {
                                continue;
                            };

                            if ["-p", "-C", "-r"].contains(&args[1].as_str()) {
                                let first_arg = args[1].as_str();
                                let last_arg = &args[args.len() - 1];
                                let mut complete_cmd = CompleteCmd {
                                    flag: first_arg.to_string(),
                                    cmd: last_arg.to_string(),
                                    path: String::new(),
                                };
                                let mut temp_hist = complete_history.borrow_mut();

                                match first_arg {
                                    "-p" => {
                                        if complete_cmd_history.len() == 0 {
                                            complete_cmd.run_complete();
                                            continue;
                                        } else {
                                            let found_cmd = complete_cmd_history
                                                .iter()
                                                .find(|cmd| cmd.cmd == complete_cmd.cmd)
                                                .unwrap();

                                            found_cmd.run_complete();
                                        }
                                    }
                                    "-r" => {
                                        //println!("-r cmd");
                                        if complete_cmd_history.len() > 0 {
                                            // remove registered cmd
                                            //println!("start filtered cmd: {}", complete_cmd_history.len());
                                            let filtered_cmd: Vec<CompleteCmd> =
                                                complete_cmd_history
                                                    .iter()
                                                    .filter(|cmd| cmd.cmd != complete_cmd.cmd)
                                                    .cloned()
                                                    .collect();
                                            //println!("filtered cmd: {}", filtered_cmd.len());
                                            *temp_hist = filtered_cmd.clone();
                                            complete_cmd_history = filtered_cmd;
                                            //complete_cmd.set_path(String::new());
                                            continue;
                                        } else {
                                            println!(
                                                "complete: {last_arg}: no completion specification"
                                            )
                                        }
                                    }
                                    "-C" => {
                                        let path = args[2].as_str();
                                        complete_cmd.set_cmd(last_arg.to_string());
                                        complete_cmd.set_path(path.to_string());
                                        if complete_cmd_history.len() == 0 {
                                            complete_cmd_history.push(complete_cmd.clone());
                                            *temp_hist = vec![complete_cmd.clone()];
                                            continue;
                                        }
                                        let mut new_history: Vec<CompleteCmd> =
                                            complete_cmd_history
                                                .into_iter()
                                                .filter(|cmd| cmd.cmd != complete_cmd.cmd)
                                                .collect();

                                        new_history.push(complete_cmd);

                                        complete_cmd_history = new_history.clone();
                                        *temp_hist = new_history;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {
                            if (args.len() >= 3 && args.clone().last().unwrap() == "&") {
                                //println!("args: {:?}", args);
                                //check if any job has finished
                                //job_list.remove_job();

                                let arc_for_thread = Arc::clone(&job_list.last_finished_job);
                                let arc_for_pid = Arc::clone(&job_list.last_job_pid);
                                let (job_id_temp, p_id, status, finished) =
                                    handle_jobs(args.clone(), job_id, arc_for_thread, arc_for_pid);
                                stdout().flush().unwrap();

                                //println!("temp is {temp_id}");

                                let job = Job {
                                    num: job_id,
                                    pid: p_id,
                                    status: status,
                                    cmd: args.join(" "),
                                    marker: "+".to_string(),
                                    finished: finished,
                                    delete_counter: 0,
                                };

                                job_list.add_job(job);
                                job_list.sequence();
                                job_id = job_id_temp;
                                continue;
                            }
                            if !run_cmd(&args) {
                                println!("{}: command not found", input);
                                continue;
                            }
                            // print last known job
                            job_list.display_last_job();
                        }
                    }
                }
            }
            Err(_) => println!("No input"),
        }
    }
}
