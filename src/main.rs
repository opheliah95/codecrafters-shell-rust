#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    //TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut input =String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut input).unwrap();
    println!("{}: command not found", input)
    // match input.trim() {
    //     "xyz" =>println!("xyz: command not found"),
    //     _ => ()
    // }
}
