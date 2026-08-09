pub mod solutions {
    use is_executable::IsExecutable;
    use std::env;
    use std::io::{Error, ErrorKind, Write, stdout};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    pub mod cmd;
    use glob::glob;
    use regex::Regex;
    use shellwords;
    use std::fs;
    use std::fs::File;
    use std::fs::metadata;
    use std::sync::{Arc, Mutex};

    pub fn type_impl(input: &str) {
        let args = input.split(" ").collect::<Vec<&str>>();
        match args.as_slice() {
            [] => (),
            ["echo"] => println!(),
            ["echo", ..] => println!("{}", &input[5..]),
            ["type", args @ ("echo" | "exit" | "type")] => println!("{args} is a shell builtin"),
            ["type", args @ ..] => println!("{}: not found", args[0]),
            _ => println!("{}: command not found", input),
        }
    }

    pub fn get_env_path() -> Option<Vec<String>> {
        let env_args: Vec<String> = env::args().collect();
        //println!("Programme name: {}", env_args[0]);
        match env::var("PATH") {
            Ok(path_val) => {
                //println!("Path found {path_val}");
                let indv_paths: Vec<String> = path_val.split(":").map(|s| s.to_string()).collect();
                // for path in &indv_paths {
                //     println!("the path is {path}")
                // }

                Some(indv_paths)
            }

            Err(e) => {
                println!("error {e}");
                None
            }
        }
    }

    pub fn check_valid_path(args: &[String]) {
        let first_arg = &args[1];
        let mut defaults = vec!["/usr/bin", "/usr/local/bin"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();

        let paths = match get_env_path() {
            Some(mut p) => {
                p.append(&mut defaults);
                p
            }
            None => defaults,
        };

        // If you specifically wanted to append defaults to an existing path list:
        // let combined = paths + defaults;
        let mut found = false;
        for path in &paths {
            let full_path = Path::new(path).join(first_arg);
            if full_path.is_executable() {
                println!("{} is {}", first_arg, full_path.display());
                found = true;
                break;
            }
        }
        if !found {
            println!("{first_arg}: not found");
        }
    }

    pub fn find_exe_path_partial(prefix: &str) -> Result<Vec<String>, Error> {
        let mut results = Vec::new();
        let mut is_path = false;
        let mut cmd: Vec<&str> = prefix.split(" ").collect();
        let last_arg = cmd.last().unwrap();

        if let Some((first, sec)) = last_arg.split_once("/") {
            //println!("first is {first}, second is {sec}");

            if first == "." {
                //println!("exit exe search");
                return Ok(results);
            }
            is_path = true
        }

        if let Some(all_path) = get_env_path() {
            for path in all_path {
                let search = format!("{path}/{prefix}*");
                for exe in glob(search.as_str()).unwrap() {
                    match exe {
                        Ok(p) => {
                            let p_clone = p.clone();
                            let p_to_string = p.file_name().and_then(|s| s.to_str());
                            let string_spilt: Vec<&str> = p_to_string.unwrap().split("/").collect();
                            let last_part = string_spilt.last().unwrap().to_string();
                            // println!("last part is {last_part}");
                            if !is_path {
                                results.push(last_part);
                            } else {
                                let full_path = p.into_os_string().into_string().unwrap();
                                //println!("full path: {full_path} and pwd is {pwd_path}");
                                let mut modified_string =
                                    update_found_file_path(&full_path, p_clone);
                                results.push(modified_string.to_string());
                            }
                        }
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
            }
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Environment path not found",
            ));
        }
        //println!("results are: {:?}", results);
        Ok(results)
    }

    //find files in current path
    pub fn find_files_in_pwd(prefix: &str) -> Result<Vec<String>, Error> {
        //println!("prefix is{prefix}");
        let mut is_path = false;
        let mut results = Vec::new();
        let mut cmd: Vec<&str> = prefix.split(" ").collect();
        let last_arg = cmd.last().unwrap();

        if let Some((first, sec)) = last_arg.split_once("/") {
            //println!("pwd first is {first}, second is {sec}");
            is_path = true
        }

        if let pwd = env::current_dir().unwrap() {
            let pwd_path = &pwd.into_os_string().into_string().unwrap();
            let search = format!("{pwd_path}/{last_arg}*",);
            //println!("Searching {search}");
            for exe in glob(search.as_str()).unwrap() {
                match exe {
                    Ok(p) => {
                        let p_to_string = p.file_name().and_then(|s| s.to_str());
                        let string_spilt: Vec<&str> = p_to_string.unwrap().split("/").collect();
                        let last_part = string_spilt.last().unwrap().to_string();
                        let modified_string = update_found_file_path(pwd_path, p);
                        results.push(modified_string.to_string());
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
        } else {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Environment path not found",
            ));
        }

        //println!("pwd results are: {:?}", results);

        Ok(results)
    }

    fn update_found_file_path(input_path: &String, p: PathBuf) -> String {
        let full_path = p.into_os_string().into_string().unwrap();
        //println!("full path: {full_path}");
        let mut modified_string = full_path.replace(input_path, "");
        modified_string = modified_string.trim().to_string();
        if modified_string.chars().next() == Some('/') {
            modified_string = modified_string[1..].to_string();
        }
        //println!("updated input {modified_string}");
        let md: fs::Metadata = metadata(&full_path).unwrap();
        if md.is_dir() {
            modified_string = format!("{modified_string}/");
        }

        modified_string
    }

    pub fn check_file_extension(matches: Vec<String>) -> bool {
        if matches.len() <= 1 {
            return true;
        }

        let pwd_path = env::current_dir().unwrap();
        let first_file = matches[0].clone();
        let first_path_buf = pwd_path.join(&first_file);
        let first_path = Path::new(&first_path_buf);
        //println!("first path: {}", first_path.to_str().unwrap());
        matches.iter().skip(1).any(|filename| {
            let current_path_buf = pwd_path.join(filename);
            let current_path = Path::new(&current_path_buf);
            //println!("sec: {}, {}",filename, current_path.extension() == first_path.extension());
            let mut initial = first_file.to_string().clone();
            initial.pop();
            //println!("first file is {initial}");
            if filename.contains(&initial) {
                //println!("{initial} in {filename}");
                return true;
            }
            current_path.extension() == first_path.extension()
        })
    }
    pub fn run_external_program(cmd: &str) -> bool {
        let args: Vec<String> = cmd.split(" ").map(|x| x.to_string()).collect();
        //println!("$ {cmd}");
        // println!(
        //     "Program was passed {} args (including program name).",
        //     args.len()
        // );
        // for (index, arg) in args.iter().enumerate() {
        //     if (index == 0) {
        //         println!("Arg #{} (program name): {}", index, arg)
        //     } else {
        //         println!("Arg #{}: {}", index, arg)
        //     }
        // }

        let paths = match get_env_path() {
            Some(p) => p,
            None => vec!["/usr/bin", "/usr/local/bin"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        };

        for path in &paths {
            let full_path = Path::new(path).join(&args[0]);
            if full_path.is_executable() {
                // execute command etc git push
                let output = Command::new(&args[0])
                    .args(&args[1..])
                    .output()
                    .expect("Program  not found");
                let stdout = String::from_utf8_lossy(&output.stdout);
                print!("{}", stdout);
                return true;
            }
        }
        return false;
    }

    pub fn get_pwd() {
        if let Ok(cwd) = env::current_dir() {
            println!("{}", cwd.display());
        }
    }

    pub fn cd_path(args: &[String]) {
        match args.len() == 1 {
            true => {
                //println!("the path is: {}", args[0]);
                // handle tlide
                let mut first_arg = &args[0];
                let path: PathBuf = if args[0].trim() == "~" {
                    std::env::home_dir().unwrap_or_else(|| PathBuf::from("/home"))
                } else {
                    PathBuf::from(&first_arg)
                };

                if path.exists() {
                    env::set_current_dir(path);
                    //println!("Current dir: {:?}", env::current_dir());
                } else {
                    println!("cd: {}: No such file or directory", first_arg);
                }
            }

            false => {
                println!("cd: {}: No such file or directory", args.join(" "))
            }
        }
    }

    pub fn is_in_quotes(args: &str) -> bool {
        let start = args.chars().next().unwrap();
        let end: char = args.chars().last().unwrap();
        if start == end && ['\'', '\"'].contains(&start) {
            return true;
        }

        let re_single = Regex::new(r#"'.*'"#).unwrap();
        let re_double = Regex::new(r#"".*"$"#).unwrap();
        if re_single.is_match(args) || re_double.is_match(args) {
            return true;
        }

        return false;
    }

    // pub fn check_cat (args:  &str) -> String {
    //     let re_single = Regex::new(r#"'.*'"#).unwrap();
    //     let re_double = Regex::new(r#"".*"$"#).unwrap();
    //     let mut result = String::new();
    //     for cap in re_single.captures_iter(args) {
    //         println!("captured is: {:#?}", cap);
    //     }
    //     args.to_string()

    // }
    pub fn clean_input(args: &str) -> String {
        let mut result: String = args.to_string();
        let is_cat = args.len() >= 3 && &args[..3] == "cat";
        let re_single = Regex::new(r#"'.*'"#).unwrap();
        let re_double = Regex::new(r#""([^"]*)""#).unwrap();
        if is_cat {
            return result;
        }

        if !is_in_quotes(&result) {
            result = result.split_whitespace().collect::<Vec<&str>>().join(" ")
        }

        //match single quotes
        if re_single.is_match(&result) {
            result = result.replace("''", "").replace("\"\"", "");
            let specials = ["\"", "'", "\\"];
            for i in specials {
                result = result.replace(i, "");
            }
        }

        //match single quotes
        if re_double.is_match(&result) {
            let quoted_result = shellwords::split(&result).unwrap();
            // for cap in re_double.captures_iter(&result) {
            //     if let Some(m) = cap.get(1) {
            //         quoted_result.push(m.as_str());
            //     }
            // }

            result = quoted_result.join(" ");
            // let mut cmd_start = args.split_whitespace().next().unwrap_or("");
            // result = format!("{} {}", cmd_start, first_arg);
            //println!("formated reuslt is {}", result);
        }

        result
    }

    pub fn get_cat(args: &str) {
        let re_single = Regex::new(r#"'([^']*)'"#).unwrap();
        let re_double = Regex::new(r#""([^"]*)""#).unwrap();
        let mut result = Vec::new();

        // Collect matches from single quotes
        for cap in re_single.captures_iter(args) {
            if let Some(m) = cap.get(1) {
                result.push(m.as_str().to_string());
            }
        }

        // Collect matches from double quotes
        for cap in re_double.captures_iter(args) {
            if let Some(m) = cap.get(1) {
                result.push(m.as_str().to_string());
            }
        }

        // If we found no quoted arguments, run as a raw external command
        if result.is_empty() {
            run_external_program(&args.to_string());
            return;
        }

        let mut fixed_result = Vec::new();
        match re_double.is_match(args) {
            true => fixed_result = result,

            false => {
                fixed_result = result
                    .into_iter()
                    .map(|item| item.replace("'", ""))
                    .collect::<Vec<String>>()
            }
        }
        //println!("updated result is now {:?}", fixed_result);

        let output = Command::new("cat")
            .args(fixed_result)
            .output()
            .expect("Program  not found");
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{}", stdout);
        if !output.status.success() {
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
    }

    pub fn run_cmd(args: &[String]) -> bool {
        let output = Command::new(&args[0]).args(&args[1..]).output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                print!("{}", stdout);
                return true;
            }
            Err(_) => {
                return false;
            }
        }
    }

    pub fn run_cmd_with_output(cmd: Vec<String>, dir: String, operator: Option<String>) -> bool {
        match Command::new(&cmd[0]).args(&cmd[1..]).output() {
            Ok(output) => {
                let path = Path::new(&dir);
                if output.status.success() {
                    let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    if let Some(op) = operator {
                        if ["2>", "2>>"].contains(&op.as_str()) {
                            print!("{stdout}");
                            stdout = "".to_string();
                        }

                        //append to existing files as a new line
                        if [">>", "1>>", "2>>"].contains(&op.as_str()) {
                            // if a file not exist then create it
                            create_path(path);

                            stdout = format_file_content(path, &stdout);
                        }
                    };

                    if let Some(parent) = path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            eprintln!("Failed to create directory {:?}: {}", parent, e);
                            return false;
                        }
                    }

                    if let Err(e) = fs::write(&path, stdout) {
                        eprintln!("Failed to write to file {:?}: {}", path, e);
                        return false;
                    }

                    return true;
                } else {
                    let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    //force even bad output to write!!!
                    match operator {
                        Some(op) => {
                            if op == "2>" {
                                let path = Path::new(&dir);
                                fs::write(&path, stderr);
                                print!("{stdout}");
                            } else if op == "2>>" {
                                let path = Path::new(&dir);
                                create_path(path);
                                let content = format_file_content(path, &stderr);
                                fs::write(&path, content);
                            } else {
                                let path = Path::new(&dir);
                                fs::write(&path, stdout);
                                print!("{stderr}");
                            }

                            return true;
                        }

                        None => return false, // let error_args: Vec<String> = cmd.iter().map(|x| format!("{}: ", x)).collect();
                                              // let error_args_result = error_args.join("");
                                              // println!("{error_args_result}No such file or directory",);
                    }
                    return false;
                }
            }

            Err(e) => {
                println!("{e}");
                let error_args: Vec<String> = cmd.iter().map(|x| format!("{}: ", x)).collect();
                let error_args_result = error_args.join("");
                println!("{error_args_result}No such file or directory",);
                return false;
            }
        }
    }

    fn format_file_content(path: &Path, output: &str) -> String {
        let mut contents =
            fs::read_to_string(path).expect("Should have been able to read the file");

        contents.push_str(output);

        contents
    }

    fn create_path(path: &Path) {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create directory {:?}: {}", parent, e);
                }

                File::create(path).unwrap();
            }
        }
    }

    pub fn strip_trailing_nl(input: &mut String) -> String {
        let new_len = input
            .char_indices()
            .rev()
            .find(|(_, c)| !matches!(c, '\n' | '\r'))
            .map_or(0, |(i, _)| i + 1);

        if new_len != input.len() {
            input.truncate(new_len);
        }

        return input.to_string();
    }

    pub fn handle_jobs(
        args: Vec<String>,
        job_id: i32,
        finished_job: Arc<Mutex<String>>
    ) -> (i32, u32, Arc<Mutex<String>>, Arc<Mutex<bool>>) {
        let job = &args[0];
        let param = &args[1];

        let status = Arc::new(Mutex::new("Running".to_string()));
        let status_clone = Arc::clone(&status);
        let finished = Arc::new(Mutex::new(false));
        let finished_clone = Arc::clone(&finished);

        // store finished job
        let finished_job_clone = Arc::clone(&finished_job);

        let mut child = Command::new(job)
            .arg(param)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("error running cmd");

        let pid = child.id();
        println!("[{job_id}] {pid}");

        // Spawn thread to update status when done
        std::thread::spawn(move || match child.wait() {
            Ok(_exit_status) => {
                if let Ok(mut temp) = status_clone.lock() {
                    *temp = "Done".to_string();
                }
                if let Ok(mut finish_temp) = finished_clone.lock() {
                    *finish_temp = true;
                }

                if let Ok(mut finished_job_temp) = finished_job_clone.lock() {
                      let output = child.wait_with_output().unwrap();
                      //println!("{}", output.status);
                    if output.status.success() {
                        let mut arg_str = args.clone().join(" ");
                        arg_str.pop();
                        let done_output =
                            format!("[{}]{}  {}                 {}", job_id, "+", "Done", arg_str.trim());
                        
                        let out_str = str::from_utf8(&output.stdout).unwrap();
                         *finished_job_temp = done_output;
                        //println!("inside exit: {done_output}");
                        // no output just return str
                        // if (out_str.len() == 0) {
                        //     *finished_job_temp = done_output;
                        // } else {
                        //     *finished_job_temp = format!("{out_str}\n{done_output}");
                        // }
                        
                       
                    }
                }
            }
            Err(e) => {
                if let Ok(mut temp) = status_clone.lock() {
                    *temp = format!("Error: {}", e);
                }
            }
        });

        // Return the lock immediately so the shell doesn't freeze
        (job_id + 1, pid, status, finished)
    }

    // fn handle_complete_func(args: Vec<String> ){
    //     match args.as_slice() {
    //         ["complete", "-p" @..] => {
    //             if args.len() < 3 {return };
    //             let path = args[]
    //             println!("complete -C '{path}' {cmd}");
    //         }
    //     }
    // }
}
