#[allow(unused_imports)]
use std::io::{self, Write};
use codecrafters_shell::solutions::{cd_path, check_valid_path, clean_input, get_env_path, get_pwd, run_external_program, type_impl};

fn main() {
    //TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();
        input = clean_input(&input);
        let args: Vec<&str> = input.split(" ").collect();
        match args.as_slice() {
            [] => (),
            ["exit"] => break,
            ["echo"] => println!(),
            ["echo", ..] => println!("{}", &input[5..]),
            ["pwd"] => get_pwd(),
            ["cd", args @..] => cd_path(args),
            ["type", args @ ("echo" | "exit" | "type" | "pwd" | "cd")] => println!("{args} is a shell builtin"),
            ["type", args @ ..] => check_valid_path(args),
            _ => {
                if !run_external_program(&input) {
                    println!("{}: command not found", input)
                }
            } 
        }
    }
}
// match input.trim() {
//     "xyz" =>println!("xyz: command not found"),
//     _ => ()
// }
