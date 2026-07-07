#[allow(unused_imports)]
use std::io::{self, Write};
use codecrafters_shell::solutions::type_impl;
use std::env;
fn main() {
    //TODO: Uncomment the code below to pass the first stage
    loop {
        let env_args:Vec<String> = env::args().collect();
        println!("Programme name: {}", env_args[0]);
        for (i, arg) in env_args.iter().enumerate() {
            println!("the {i} arg is {arg} ")
        }
        

        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        let args: Vec<&str> = input.split(" ").collect();
        match args.as_slice() {
            [] => (),
            ["exit"] => break,
            ["echo"] => println!(),
            ["echo", ..] => println!("{}", &input[5..]),
            ["type", args @ ("echo" | "exit" | "type")] => println!("{args} is a shell builtin"),
            ["type", args@("grep"| "ls")] => println!("{args} is /usr/bin/{args}"),
            ["type", args @ ..] => println!("{}: not found", args[0]),
            _ => println!("{}: command not found", input),
        }
    }
}
// match input.trim() {
//     "xyz" =>println!("xyz: command not found"),
//     _ => ()
// }
