use codecrafters_shell::solutions::{cd_path, check_valid_path, run_cmd, get_pwd, run_cmd_with_output};
use shellwords;
#[allow(unused_imports)]
use std::io::{self, Write};
fn main() {
    //TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        //input = clean_input(&input);
        let mut args = shellwords::split(&input).unwrap();
        //println!("teh agrs: {:?}", args);
        if args.len() >= 1 {
            let output = ["1>", ">"];
            let mut split_index: Option<usize> = None;
            for (index, val) in args.iter().enumerate() {
                if output.contains(&val.as_str()){
                    split_index = Some(index);
                    break;
                }
            }

            if let Some(index) = split_index {
                let file_dir = args.split_off(index+1);
                args.pop();
                // println!("now args is {:?}", args);
                // println!("now file dir is: {:?}", file_dir);
                run_cmd_with_output(args.clone(), file_dir[0].to_string());
                continue;

            }
            match &args[0] {
            cmd if cmd.len() == 0 => continue,
            cmd if *cmd == "exit"=> break,
            cmd if *cmd == "cd" => cd_path(&args[1..]),
            cmd if *cmd =="pwd" => get_pwd(),
            cmd if *cmd == "type" => {
                if ["echo", "exit", "type", "pwd"].contains(&args[1].as_str()) {
                    println!("{} is a shell builtin", &args[1])
                } else {
                    check_valid_path(&args);
                }
            },
            _ => {
                if !run_cmd(args) {
                    println!("{}: command not found", input)
                }
            }
        }
        }
    }
}
// match input.trim() {
//     "xyz" =>println!("xyz: command not found"),
//     _ => ()
// }
