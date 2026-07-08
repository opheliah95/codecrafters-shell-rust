pub mod solutions {
    use is_executable::IsExecutable;
    use std::env;
    use std::io::stdout;
use std::path::Path;
    use std::process::Command;

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
                println!("{stdout}");
                break;
            }
        }
        return true;
    }
}
