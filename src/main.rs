use codecrafters_shell::solutions::{cd_path, check_valid_path, run_cmd, get_pwd};
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
        let args = shellwords::split(&input).unwrap();
        //println!("teh agrs: {:?}", args);
        if args.len() >= 1 {
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
