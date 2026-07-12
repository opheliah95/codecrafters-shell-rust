pub mod solutions {
    use is_executable::IsExecutable;
    use std::env;
    use std::io::stdout;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    mod cmd;
    use regex::Regex;
    use shellwords;

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

    pub fn check_valid_path(args: &[&str]) {
        let first_arg = args[0];
        let env_paths = get_env_path();
        let mut found = false;
        match env_paths {
            Some(paths) => {
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

            _ => {
                println!("{first_arg}: not found")
            }
        }
    }

    pub fn run_external_program(cmd: &String) -> bool {
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

    pub fn cd_path(args: &[&str]) {
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
            let specials = ["\"", "'"];
            for i in specials {
                result = result.replace(i, "");
            }
        }

        //match single quotes
        if re_double.is_match(&result) {
            let quoted_result =shellwords::split(&result).unwrap();
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

        let mut fixed_result =Vec::new();
        match re_double.is_match(args) {
            true => fixed_result = result,

            false =>fixed_result = result
            .into_iter()
            .map(|item| item.replace("'", ""))
            .collect::<Vec<String>>()
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
}
